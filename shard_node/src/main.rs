mod run;

extern crate clap;
use shard_chain::ShardChainHarness;
use clap::{Arg, App, SubCommand};

fn main() {
    let matches = App::new("My Super Program")
                          .version("0.1.0")
                          .author("Will Villanueva")
                          .about("Simulates Shard Chains")
                          .arg(Arg::with_name("shards")
                               .short("s")
                               .long("shards")
                               .value_name("FILE")
                               .help("Sets a custom config file")
                               .takes_value(true))
                          .get_matches();

    // Matches number of shards to run
    let shards = matches.value_of("shards").unwrap_or("1");
    run::run_harness();
}


