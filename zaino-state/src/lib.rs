//! A mempool and chain-fetching service built on top of zebra's ReadStateService and TrustedChainSync.

#![warn(missing_docs)]
#![forbid(unsafe_code)]

pub mod error;
pub mod state;
pub mod status;
