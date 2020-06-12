use log::{error, info};
use arq::Repository;
use futures::{Future, future};
use crate::cli::ListFolderOpts;

pub fn list_folders(repo: &Repository, args: ListFolderOpts) -> Box<dyn Future<Item = (), Error = ()> + Send>
{
    let f = repo.list_folders(&args.computer);
    Box::new(f)
}