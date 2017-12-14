extern crate arq;
extern crate argparse;
extern crate chrono;
extern crate toml;
#[macro_use] extern crate log;
#[macro_use] extern crate serde_derive;
extern crate serde;
extern crate simple_logger;
extern crate rusoto_core;
extern crate rusoto_s3;

mod cli;
mod config;

use std::env;
use std::io;
use std::process::exit;
use rusoto_s3::{S3, S3Client};
use rusoto_core::{Region, default_tls_client};

fn main() {
    use cli::Args;

    let args = match Args::parse(env::args().collect(),
                                 &mut io::stdout(),
                                 &mut io::stderr()) {
        Ok(a) => a,
        Err(n) => exit(n)
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

    let dispatcher = default_tls_client().unwrap();
    let client = S3Client::new(
        dispatcher,
        cfg.clone(),
        Region::ApSoutheast2
    );

    let buckets = client.list_buckets().unwrap();
    match buckets.buckets {
        Some(bs) => {
            for b in bs.iter() {
                let bucket_name = match b.name {
                    Some(ref s) => s,
                    None => "unnamed"
                };
                println!("Bucket: {}", bucket_name);
            }
        },
        None => {
            println!("No buckets to be had");
        }
    }

}

