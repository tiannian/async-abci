use tm_abci::{request, Request, RequestBeginBlock, RequestDeliverTx, RequestFinalizedBlock};

use crate::{Error, Result};

use std::mem;

#[derive(Debug)]
pub enum ConsensusState {
    Begin,
    ConsensusBegin,
    ConsensusBeginFlush,
    BlockBegin,
    BlockBeginFlush,
    TxReceived,
    BlockEnd,
    DeliverBlock,
    BlockCommit,
    BlockCommitFlush,
}

impl Default for ConsensusState {
    fn default() -> Self {
        Self::Begin
    }
}

#[derive(Default)]
pub struct ConsensusQueue {
    pub block: RequestBeginBlock,
    pub txs: Vec<RequestDeliverTx>,
    pub state: ConsensusState,
    pub packet: Option<Request>,
}

impl ConsensusQueue {
    pub fn new(pkt: Request) -> Result<Self> {
        match pkt.value.ok_or(Error::ABCIFormatError)? {
            request::Value::InitChain(_) => Ok(Default::default()),
            request::Value::BeginBlock(block) => {
                let cq = ConsensusQueue {
                    block,
                    txs: Vec::new(),
                    state: ConsensusState::Begin,
                    packet: None,
                };
                Ok(cq)
            }
            _ => Err(Error::ABCIPacketError),
        }
    }

    pub fn is_consensus(pkt: &Request) -> bool {
        matches!(pkt.value, Some(request::Value::InitChain(_)))
    }

    pub fn is_deliver_block(&self) -> bool {
        matches!(self.state, ConsensusState::DeliverBlock)
    }

    pub fn is_sendable(&self) -> bool {
        matches!(self.state, ConsensusState::ConsensusBeginFlush)
    }

    pub fn is_begin_block_flush(&self) -> bool {
        matches!(self.state, ConsensusState::BlockBeginFlush)
    }

    pub fn is_commit(&self) -> bool {
        matches!(self.state, ConsensusState::BlockCommit)
    }

    pub fn is_commit_flush(&self) -> bool {
        matches!(self.state, ConsensusState::BlockCommitFlush)
    }

    pub fn add_pkt(&mut self, pkt: Request) -> Result<()> {
        match (&self.state, pkt.value.ok_or(Error::ABCIFormatError)?) {
            // Begin -> ConsensusBegin
            (ConsensusState::Begin, request::Value::InitChain(p)) => {
                self.packet = Some(Request {
                    value: Some(request::Value::InitChain(p)),
                });
                self.state = ConsensusState::ConsensusBegin;
            }
            // ConsensusBegin -> ConsensusBeginFlush
            (ConsensusState::ConsensusBegin, request::Value::Flush(_)) => {
                self.state = ConsensusState::ConsensusBeginFlush;
            }
            // ConsensusBeginFlush -> BlockBegin
            (ConsensusState::ConsensusBeginFlush, request::Value::BeginBlock(p)) => {
                self.block = p;
                self.state = ConsensusState::BlockBegin;
            }
            // Begin -> BlockBegin
            (ConsensusState::Begin, request::Value::BeginBlock(p)) => {
                self.block = p;
                self.state = ConsensusState::BlockBegin;
            }
            // BlockBegin -> BlockBeginFlush
            (ConsensusState::BlockBegin, request::Value::Flush(_)) => {
                self.state = ConsensusState::BlockBeginFlush;
            }
            // BlockBegin -> TxReceived
            (ConsensusState::BlockBeginFlush, request::Value::DeliverTx(p)) => {
                self.txs.push(p);
                self.state = ConsensusState::TxReceived;
            }
            // TxReceived -> TxReceived
            (ConsensusState::TxReceived, request::Value::DeliverTx(p)) => {
                self.txs.push(p);
            }
            // TxReceived -> BlockEnd
            (ConsensusState::TxReceived, request::Value::EndBlock(_)) => {
                self.state = ConsensusState::BlockEnd;
            }
            // BlockBeginFlush -> BlockEnd
            (ConsensusState::BlockBeginFlush, request::Value::EndBlock(_)) => {
                self.state = ConsensusState::BlockEnd;
            }
            // BlockEnd -> DeliverBlock
            (ConsensusState::BlockEnd, request::Value::Flush(_)) => {
                self.state = ConsensusState::DeliverBlock;
            }
            // DeliverBlock -> BlockCommit
            (ConsensusState::DeliverBlock, request::Value::Commit(_)) => {
                self.state = ConsensusState::BlockCommit;
            }
            // BlockCommit -> BlockCommitFlush
            (ConsensusState::BlockCommit, request::Value::Flush(_)) => {
                self.state = ConsensusState::BlockCommitFlush;
            }
            // BlockCommitFlush -> BlockBegin
            (ConsensusState::BlockCommitFlush, request::Value::BeginBlock(p)) => {
                self.block = p;
                self.state = ConsensusState::BlockBegin;
            }
            _ => return Err(Error::ABCIPacketError),
        }

        Ok(())
    }

    pub fn to_block(&mut self) -> Result<RequestFinalizedBlock> {
        if let ConsensusState::DeliverBlock = self.state {
            let block = mem::take(&mut self.block);
            let txs = mem::take(&mut self.txs);

            Ok(RequestFinalizedBlock::new(block, txs))
        } else {
            Err(Error::StateError)
        }
    }

    pub fn to_packet(&mut self) -> Result<Request> {
        let request = mem::replace(&mut self.packet, None);
        request.ok_or(Error::ABCIPacketError)
    }
}
