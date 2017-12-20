extern crate arq;
extern crate argparse;
extern crate chrono;
extern crate toml;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate simple_logger;
extern crate rusoto_core;
extern crate rusoto_s3;

mod cli;
mod config;

use std::env;
use std::io;
use std::process::exit;
use rusoto_core::{Region};

fn main() {
    use cli::Args;

    let args = match Args::parse(env::args().collect(), &mut io::stdout(), &mut io::stderr()) {
        Ok(a) => a,
        Err(n) => exit(n),
    };

    simple_logger::init_with_level(log::LogLevel::Debug).unwrap();

    debug!("Loading config from {:?}...", args.config_file);
    let cfg = match config::load(&args.config_file) {
        Ok(cfg) => cfg,
        Err(msg) => {
            error!("Config load failed: {:?}", msg);
            exit(1)
        }
    };

    let transport = Box::new(arq::s3::Transport::new(
        &cfg.bucket_name,
        &cfg.access_key_id,
        &cfg.secret_key,
        rusoto_core::Region::ApSoutheast2));
    let repo = arq::Repository::new(&args.computer_id, transport);

    debug!("fetching repo salt...");
    match repo.salt() {
        Ok(s) => info!("Salt: {:?}", s),
        Err(e) =>  error!("Listing failed with error: {:?}", e)
    }


}
