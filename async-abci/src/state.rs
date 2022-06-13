use tm_abci::{request, Request, RequestBeginBlock, RequestDeliverTx, RequestFinalizedBlock};

use crate::{Error, Result};

use std::mem;

#[derive(Debug)]
pub enum ConsensusState {
    Begin,
    ConsensusBegin,
    BlockBegin,
    TxReceived,
    BlockEnd,
    DeliverBlock,
}

impl Default for ConsensusState {
    fn default() -> Self {
        Self::Begin
    }
}

pub enum State {
    Consensus(ConsensusQueue),
    Other,
}

#[derive(Default)]
pub struct ConsensusQueue {
    pub block: RequestBeginBlock,
    pub txs: Vec<RequestDeliverTx>,
    pub state: ConsensusState,
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
                };
                Ok(cq)
            }
            _ => Err(Error::ABCIPacketError),
        }
    }

    pub fn is_consensus(pkt: &Request) -> bool {
        matches!(pkt.value, Some(request::Value::BeginBlock(_)))
    }

    pub fn is_deliver_block(&self) -> bool {
        matches!(self.state, ConsensusState::DeliverBlock)
    }

    pub fn add_pkt(&mut self, pkt: Request) -> Result<()> {
        match (&self.state, pkt.value.ok_or(Error::ABCIFormatError)?) {
            // Begin -> ConsensusBegin
            (ConsensusState::Begin, request::Value::InitChain(_)) => {
                self.state = ConsensusState::ConsensusBegin;
            }
            // ConsensusBegin -> BlockBegin
            (ConsensusState::ConsensusBegin, request::Value::BeginBlock(p)) => {
                self.block = p;
                self.state = ConsensusState::BlockBegin;
            }
            // Begin -> BlockBegin
            (ConsensusState::Begin, request::Value::BeginBlock(p)) => {
                self.block = p;
                self.state = ConsensusState::BlockBegin;
            }
            // BlockBegin -> TxReceived
            (ConsensusState::BlockBegin, request::Value::DeliverTx(p)) => {
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
            // BlockEnd -> DeliverBlock
            (ConsensusState::BlockEnd, request::Value::Flush(_)) => {
                self.state = ConsensusState::DeliverBlock;
            }
            // DeliverBlock -> BlockBegin
            (ConsensusState::DeliverBlock, request::Value::BeginBlock(p)) => {
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
}
