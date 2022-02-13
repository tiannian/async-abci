/// Consensus captures the consensus rules for processing a block in the blockchain,
/// including all blockchain data structures and the rules of the application's
/// state transition machine.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Consensus {
    #[prost(uint64, tag="1")]
    pub block: u64,
    #[prost(uint64, tag="2")]
    pub app: u64,
}
