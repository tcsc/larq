mod cli;
mod config;
mod cmd;

use std::{process::exit, sync::Arc};

use log::{debug, error, Level};
use futures::future;
use gumdrop::Options;
use cli::{Args, Command};


fn main() {

    let args = Args::parse_args_default_or_exit();

    let log_level = if args.verbose { Level::Debug } else { log::Level::Info };
    simple_logger::init_with_level(log_level).unwrap();

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
    let repo = arq::Repository::new(Arc::new(transport));

    let mut rt = tokio::runtime::Runtime::new().expect("Runtime");

    let main_fn = match args.cmd {
        Some(Command::ListComputers(_)) => cmd::list_computers(&repo),
        Some(Command::ListFolders(opts)) => cmd::list_folders(&repo, opts),
        None => Box::new(future::ok(()))
    };

    rt.block_on(main_fn);
}
