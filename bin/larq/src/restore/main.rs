mod cli;
mod cmd;
mod config;

use gumdrop::Options;
use log::{debug, error, LevelFilter};
use std::{process::exit, sync::Arc};

use arq::s3;
use cli::{Args, Command};
use config::Config;
use simple_logger::SimpleLogger;

fn main() {
    let args = Args::parse_args_default_or_exit();


    let log_level = match args.verbose {
                        0 => LevelFilter::Warn,
                        1 => LevelFilter::Info,
                        2 => LevelFilter::Debug,
                        _ => LevelFilter::Trace,
                    };
    SimpleLogger::new().with_level(log_level).init().unwrap();

    debug!("Loading config from {:?}...", args.config_file);
    let cfg = match config::load(&args.config_file) {
        Ok(cfg) => cfg,
        Err(msg) => {
            error!("Config load failed: {:?}", msg);
            exit(1)
        }
    };

    if let Some(cmd) = args.cmd {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let result = runtime.block_on(dispatch_cmd(&cfg, &args.password, cmd));

        drop(runtime);
        exit(result);
    }
}

async fn dispatch_cmd(cfg: &Config, secret: &str, cmd: Command) -> i32 {
    let transport = s3::Store::new(
        &cfg.bucket_name,
        &cfg.access_key_id,
        &cfg.secret_key,
        rusoto_core::Region::ApSoutheast2,
        Some(std::path::PathBuf::from("./cache"))
    )
    .expect("Transport construction");
    let repo = arq::Repository::new(secret, Arc::new(transport));

    let result = match cmd {
        Command::ListComputers(_) => cmd::list_computers(&repo).await,
        Command::ListFolders(opts) => cmd::list_folders(&repo, opts).await,
        Command::ListFiles(opts) => cmd::list_files(&repo, opts).await.map_err(|e| {
            log::error!("Failed: {:?}", e);
        }),
    };

    result.map(|_| 0).unwrap_or(1)
}
