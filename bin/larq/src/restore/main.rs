mod cli;
mod config;

use log::{debug, error, info};
use rusoto_core::Region;
use std::env;
use std::io;
use std::process::exit;
use tokio::prelude::*;

use std::sync::Arc;

fn main() {
    use cli::Args;

    let args = match Args::parse(env::args().collect(), &mut io::stdout(), &mut io::stderr()) {
        Ok(a) => a,
        Err(n) => exit(n),
    };

    simple_logger::init_with_level(log::Level::Debug).unwrap();

    debug!("Loading config from {:?}...", args.config_file);
    let cfg = match config::load(&args.config_file) {
        Ok(cfg) => cfg,
        Err(msg) => {
            error!("Config load failed: {:?}", msg);
            exit(1)
        }
    };

    let transport = arq::s3::Transport::new(
        &cfg.bucket_name,
        &cfg.access_key_id,
        &cfg.secret_key,
        rusoto_core::Region::ApSoutheast2,
    )
    .expect("Transport construction");
    let repo = arq::Repository::new(&args.computer_id, Arc::new(transport));

    let mut rt = tokio::runtime::Runtime::new().expect("Runtime");

    debug!("listing computers/...");
    match rt.block_on(repo.list_computers()) {
        Ok(v) => {
            info!("Yay! - list is {} long", v.len());
            for c in v.iter() {
                info!("user: {}, name: {}", c.user, c.computer)
            }
        }
        Err(e) => error!("Listing failed with error: {:?}", e),
    }
}
