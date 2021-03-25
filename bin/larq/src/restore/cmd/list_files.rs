use crate::cli::ListFileOpts;
use arq::{format_uuid, RepoError, Repository};
use log::info;

pub async fn list_files(repo: &Repository, args: ListFileOpts) -> Result<(), RepoError> {
    // fetch computer info
    let computer = repo.get_computer(format_uuid(&args.computer)).await?;
    let folder = computer.get_folder(&format_uuid(&args.folder)).await?;

    info!("Folder: {:?}", folder.local_path());

    let latest_commit = folder.get_latest_commit().await?;

    info!("Committed at: {:?}", latest_commit.timestamp());

    latest_commit.list_files("**/*").await
}
