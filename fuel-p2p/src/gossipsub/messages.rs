use fuel_core_interfaces::common::fuel_tx::Transaction;
use fuel_core_interfaces::model::{ConsensusVote, FuelBlock};
use std::sync::Arc;

use serde::{Deserialize, Serialize};

/// Used to inform `GossipsubCodec` to which GossipsubMessage decode to
/// GossipTopicTag is decided by checking received TopicHash from the peer
#[derive(Debug, Clone, Copy)]
pub enum GossipTopicTag {
    NewTx,
    NewBlock,
    ConensusVote,
}

/// Takes Arc<T> and wraps it in a matching GossipsubBroadcastRequest
/// The inner referenced value is serialized and broadcast to the network
/// It is deserialized as `GossipsubMessage`
#[derive(Debug, Clone)]
pub enum GossipsubBroadcastRequest {
    NewTx(Arc<Transaction>),
    NewBlock(Arc<FuelBlock>),
    ConensusVote(Arc<ConsensusVote>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum GossipsubMessage {
    NewTx(Transaction),
    NewBlock(FuelBlock),
    ConensusVote(ConsensusVote),
}
