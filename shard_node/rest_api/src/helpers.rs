use crate::{ApiError, ApiResult};
use beacon_chain::BeaconChainTypes;
use shard_chain::{ShardChain, ShardChainTypes};
use http::header;
use hyper::{Body, Request};
use std::sync::Arc;

/// Checks the provided request to ensure that the `content-type` header.
///
/// The content-type header should either be omitted, in which case JSON is assumed, or it should
/// explicity specify `application/json`. If anything else is provided, an error is returned.
pub fn check_content_type_for_json(req: &Request<Body>) -> Result<(), ApiError> {
    match req.headers().get(header::CONTENT_TYPE) {
        Some(h) if h == "application/json" => Ok(()),
        Some(h) => Err(ApiError::BadRequest(format!(
            "The provided content-type {:?} is not available, this endpoint only supports json.",
            h
        ))),
        _ => Ok(()),
    }
}

pub fn get_shard_chain_from_request<T: ShardChainTypes + 'static, L: BeaconChainTypes + 'static> (
    req: &Request<Body>,
) -> Result<(Arc<ShardChain<T, L>>), ApiError> {
    // Get shard chain
    let shard_chain = req
        .extensions()
        .get::<Arc<ShardChain<T, L>>>()
        .ok_or_else(|| ApiError::ServerError("Beacon chain extension missing".into()))?;

    Ok(shard_chain.clone())
}

pub fn get_logger_from_request(req: &Request<Body>) -> slog::Logger {
    let log = req
        .extensions()
        .get::<slog::Logger>()
        .expect("Should always get the logger from the request, since we put it in there.");
    log.to_owned()
}
