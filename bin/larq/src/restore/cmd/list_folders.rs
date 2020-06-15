use log::{error, info};
use arq::Repository;
use crate::cli::ListFolderOpts;

pub async fn list_folders(repo: &Repository, args: ListFolderOpts) -> Result<(), ()>
{
    let folders = repo.list_folders(&args.computer).await;
    for f in folders.iter() {
        info!("{:?}", f);
    }

    Ok(())
}