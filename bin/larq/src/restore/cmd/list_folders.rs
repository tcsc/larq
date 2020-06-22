use crate::cli::ListFolderOpts;
use arq::{format_uuid, Repository};
use log::info;

pub async fn list_folders(repo: &Repository, args: ListFolderOpts) -> Result<(), ()> {
    let computer_id = format_uuid(&args.computer);

    let computer = repo
        .get_computer(computer_id.clone())
        .await
        .map_err(|_| ())?;

    let folders = computer.list_folders().await.map_err(|_| ())?;

    for f in folders.iter() {
        info!("F: {:?}", f);
    }

    Ok(())
}
