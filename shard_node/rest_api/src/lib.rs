#[macro_use]
mod macros;
#[macro_use]
extern crate lazy_static;

mod config;
mod error;
mod helpers;
mod response_builder;
mod url_query;
mod shard;

use beacon_chain::BeaconChainTypes;
use shard_chain::{ShardChain, ShardChainTypes};
use error::{ApiError, ApiResult};
use futures::future::IntoFuture;
use hyper::rt::Future;
use hyper::service::Service;
use hyper::{Body, Method, Request, Response, Server};
use slog::{info, o, warn};
use std::sync::Arc;
use url_query::UrlQuery;
use tokio::runtime::TaskExecutor;

pub use config::Config as ApiConfig;

type BoxFut = Box<dyn Future<Item = Response<Body>, Error = ApiError> + Send>;

pub struct ApiService<T: ShardChainTypes + 'static, L: BeaconChainTypes + 'static> {
    log: slog::Logger,
    shard_chain: Arc<ShardChain<T, L>>,
}

fn into_boxfut<F: IntoFuture + 'static>(item: F) -> BoxFut
where
    F: IntoFuture<Item = Response<Body>, Error = ApiError>,
    F::Future: Send,
{
    Box::new(item.into_future())
}

impl<T: ShardChainTypes, L: BeaconChainTypes> Service for ApiService<T, L> {
    type ReqBody = Body;
    type ResBody = Body;
    type Error = ApiError;
    type Future = BoxFut;

    fn call(&mut self, mut req: Request<Body>) -> Self::Future {
        req.extensions_mut()
            .insert::<slog::Logger>(self.log.clone());
        req.extensions_mut()
            .insert::<Arc<ShardChain<T, L>>>(self.shard_chain.clone());

        let path = req.uri().path().to_string();

        let result = match (req.method(), path.as_ref()) {
            (&Method::GET, "/hello") => into_boxfut(shard::hello(req)),
            _ => Box::new(futures::future::err(ApiError::NotFound(
                "Request path and/or method not found.".to_owned(),
            ))),
        };

        let response = match result.wait() {
            // Return the `hyper::Response`.
            Ok(response) => {
                slog::debug!(self.log, "Request successful: {:?}", path);
                response
            }
            // Map the `ApiError` into `hyper::Response`.
            Err(e) => {
                slog::debug!(self.log, "Request failure: {:?}", path);
                e.into()
            }
        };

        Box::new(futures::future::ok(response))
    }
}

pub fn start_server<T: ShardChainTypes + 'static, L: BeaconChainTypes + 'static>(
    config: &ApiConfig,
    executor: &TaskExecutor,
    shard_chain: Arc<ShardChain<T, L>>,
    log: &slog::Logger,
) -> Result<(), hyper::Error> {
    let log = log.new(o!("Service" => "Api"));

    // Get the address to bind to
    let bind_addr = (config.listen_address, config.port).into();

    // Clone our stateful objects, for use in service closure.
    let server_log = log.clone();
    let server_bc = shard_chain.clone();

    let service = move || -> futures::future::FutureResult<ApiService<T, L>, String> {
        futures::future::ok(ApiService {
            log: server_log.clone(),
            shard_chain: server_bc.clone(),
        })
    };

    let log_clone = log.clone();
    let server = Server::bind(&bind_addr)
        .serve(service)
        .map_err(move |e| {
            warn!(
            log_clone,
            "API failed to start, Unable to bind"; "address" => format!("{:?}", e)
            )
        });

    info!(
    log,
    "REST API started";
    "address" => format!("{}", config.listen_address),
    "port" => config.port,
    );

    executor.spawn(server);

    Ok(())
}
