//! Lightwallet service RPC implementations.

use std::sync::{atomic::AtomicBool, Arc};

use zaino_state::{fetch::FetchServiceSubscriber, indexer::ChainStateInterface};

pub mod service;

#[derive(Clone)]
/// Zaino gRPC service.
pub struct GrpcClient {
    /// Chain fetch service subscriber.
    pub chainstate_interface: ChainStateInterface<FetchServiceSubscriber>,
    /// Represents the Online status of the gRPC server.
    pub online: Arc<AtomicBool>,
}
