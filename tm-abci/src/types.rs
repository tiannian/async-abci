use alloc::vec::Vec;
use tm_protos::{
    abci::{
        Evidence, LastCommitInfo, RequestBeginBlock, RequestDeliverTx, ResponseDeliverTx,
        ResponseEndBlock,
    },
    types::Header,
};

#[derive(Debug, Default)]
pub struct RequestFinalizedBlock {
    pub hash: Vec<u8>,
    pub header: Header,
    pub last_commit_info: LastCommitInfo,
    pub byzantine_validators: Vec<Evidence>,
    pub transactions: Vec<Vec<u8>>,
}

impl RequestFinalizedBlock {
    pub fn new(begin: RequestBeginBlock, txs: Vec<RequestDeliverTx>) -> Self {
        let mut transactions = Vec::new();

        for tx in txs {
            transactions.push(tx.tx);
        }

        Self {
            hash: begin.hash,
            header: begin.header.unwrap(),
            last_commit_info: begin.last_commit_info.unwrap(),
            byzantine_validators: begin.byzantine_validators,
            transactions,
        }
    }
}

#[derive(Debug, Default)]
pub struct ResponseFinalizedBlock {
    pub tx_receipt: Vec<ResponseDeliverTx>,
    pub end_recepit: ResponseEndBlock,
}
