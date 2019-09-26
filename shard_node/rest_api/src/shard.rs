use crate::helpers::*;
use crate::response_builder::ResponseBuilder;
use crate::{ApiError, ApiResult, UrlQuery};
use beacon_chain::BeaconChainTypes;
use shard_chain::{ShardChain, ShardChainTypes};
use hyper::{Body, Request};

/// HTTP handler to return a `BeaconState` at the genesis block.
pub fn hello(req: Request<Body>) -> ApiResult {
    ResponseBuilder::new(&req)?.body_text("hello".to_string())
}

pub fn get_state<T: ShardChainTypes + 'static, L: BeaconChainTypes + 'static>(req: Request<Body>) -> ApiResult {
    let shard_chain = get_shard_chain_from_request::<T, L>(&req)?;
    let current_state = shard_chain.current_state();

    ResponseBuilder::new(&req)?.body(&current_state.clone())
}

pub fn get_block<T: ShardChainTypes + 'static, L: BeaconChainTypes + 'static>(req: Request<Body>) -> ApiResult {
    let shard_chain = get_shard_chain_from_request::<T, L>(&req)?;
    let current_block = &shard_chain.head().shard_block;

    ResponseBuilder::new(&req)?.body(&current_block.clone())
}
