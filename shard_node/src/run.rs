use tokio::prelude::*;
use tokio::runtime::Builder;
use tokio_timer::clock::Clock;

pub fn run_simulation(log: &slog::Logger) -> () {
    // handle tokio result or error
    let runtime = Builder::new()
        .name_prefix("shard-")
        .clock(Clock::system())
        .build()
        .map_err(|e| format!("{:?}", e))
        .unwrap();

    let executor = runtime.executor();

    shard_client::run_shard_chain(log, &executor);

    runtime.shutdown_on_idle().wait().unwrap();
}
