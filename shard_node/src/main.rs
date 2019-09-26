mod run;

extern crate clap;
use clap::{App, Arg, SubCommand};
use shard_chain::ShardChainHarness;
use slog::{crit, o, Drain, Level};

fn main() {
    let matches = App::new("My Super Program")
        .version("0.1.0")
        .author("Will Villanueva")
        .about("Simulates Shard Chains")
        .arg(
            Arg::with_name("shards")
                .short("s")
                .long("shards")
                .value_name("FILE")
                .help("Sets a custom config file")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("verbosity")
                .short("v")
                .multiple(true)
                .help("Sets the verbosity level")
                .takes_value(true),
        )
        .get_matches();

    // Matches number of shards to run
    let shards = matches.value_of("shards").unwrap_or("1");

    // build the initial logger
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::CompactFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build();

    let drain = match matches.occurrences_of("verbosity") {
        0 => drain.filter_level(Level::Info),
        1 => drain.filter_level(Level::Debug),
        2 => drain.filter_level(Level::Trace),
        _ => drain.filter_level(Level::Trace),
    };

    let mut log = slog::Logger::root(drain.fuse(), o!());

    run::run_harness(&log);
}
