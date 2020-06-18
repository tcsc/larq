mod cli;
mod cmd;
mod config;

use std::{process::exit, sync::Arc};

use cli::{Args, Command};
use gumdrop::Options;
use log::{debug, error, info, Level};

use arq::s3;

#[tokio::main]
async fn main() {
    let args = Args::parse_args_default_or_exit();

    let log_level = if args.verbose {
        Level::Trace
    } else {
        log::Level::Info
    };
    simple_logger::init_with_level(log_level).unwrap();

    debug!("Loading config from {:?}...", args.config_file);
    let cfg = match config::load(&args.config_file) {
        Ok(cfg) => cfg,
        Err(msg) => {
            error!("Config load failed: {:?}", msg);
            exit(1)
        }
    };

    let transport = s3::Store::new(
        &cfg.bucket_name,
        &cfg.access_key_id,
        &cfg.secret_key,
        rusoto_core::Region::ApSoutheast2,
    )
    .expect("Transport construction");
    let repo = arq::Repository::new(Arc::new(transport));

    let _ = match args.cmd {
        Some(Command::ListComputers(_)) => cmd::list_computers(&repo).await,
        Some(Command::ListFolders(opts)) => cmd::list_folders(&repo, opts).await,
        None => Ok(()),
    };

    info!("Exiting...");
}
