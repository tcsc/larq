use crate::cli::ListFolderOpts;
use arq::Repository;
use log::{error, info};

pub async fn list_folders(repo: &Repository, args: ListFolderOpts) -> Result<(), ()> {
    let folders = repo.list_folders(&args.computer).await;
    for f in folders.iter() {
        info!("{:?}", f);
    }

    Ok(())
}
