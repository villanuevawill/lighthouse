use crate::helpers::*;
use crate::response_builder::ResponseBuilder;
use crate::{ApiError, ApiResult, BoxFut};
use beacon_chain::BeaconChainTypes;
use futures::future::Future;
use futures::stream::Stream;
use hex;
use hyper::{Body, Request};
use serde::Deserialize;
use shard_chain::ShardChainTypes;
use slog::info;

pub fn get_state<T: ShardChainTypes + 'static, L: BeaconChainTypes + 'static>(
    req: Request<Body>,
) -> ApiResult {
    let log = get_logger_from_request(&req);
    info!(log, "Latest state requested");

    let shard_chain = get_shard_chain_from_request::<T, L>(&req)?;
    let current_state = shard_chain.current_state();

    ResponseBuilder::new(&req)?.body(&current_state.clone())
}

pub fn get_block<T: ShardChainTypes + 'static, L: BeaconChainTypes + 'static>(
    req: Request<Body>,
) -> ApiResult {
    let log = get_logger_from_request(&req);
    info!(log, "Latest block requested");

    let shard_chain = get_shard_chain_from_request::<T, L>(&req)?;
    let current_block = &shard_chain.head().shard_block;

    ResponseBuilder::new(&req)?.body(&current_block.clone())
}

#[derive(Deserialize, Debug)]
struct BlockBodyRequest {
    block_body: String,
}

pub fn process_block_body<T: ShardChainTypes + 'static, L: BeaconChainTypes + 'static>(
    req: Request<Body>,
) -> BoxFut {
    let log = get_logger_from_request(&req);
    info!(
        log,
        "A block body has been submitted, adding it to current pool."
    );

    let _ = try_future!(check_content_type_for_json(&req));
    let shard_chain = try_future!(get_shard_chain_from_request::<T, L>(&req));
    let response_builder = ResponseBuilder::new(&req);
    let body = req.into_body();

    Box::new(
        body.concat2()
            .map_err(|e| ApiError::ServerError(format!("Unable to get request body: {:?}", e)))
            .map(|chunk| chunk.iter().cloned().collect::<Vec<u8>>())
            .and_then(move |chunks| {
                serde_json::from_slice(&chunks.as_slice()).map_err(|e| {
                    ApiError::BadRequest(format!(
                        "Unable to deserialize JSON into a BeaconBlock: {:?}",
                        e
                    ))
                })
            })
            .and_then(move |block_body_request: BlockBodyRequest| {
                let body = hex::decode(block_body_request.block_body)?;
                shard_chain.process_body(body);
                Ok(())
            })
            .and_then(|_| response_builder?.body_text("success".to_string())),
    )
}
