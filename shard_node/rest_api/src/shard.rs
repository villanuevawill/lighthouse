use crate::helpers::*;
use crate::response_builder::ResponseBuilder;
use crate::{ApiError, ApiResult, UrlQuery};
use hyper::{Body, Request};

/// HTTP handler to return a `BeaconState` at the genesis block.
pub fn hello(req: Request<Body>) -> ApiResult {
    ResponseBuilder::new(&req)?.body_text("hello".to_string())
}
