use crate::cli::ListFileOpts;
use arq::{format_uuid, RepoError, Repository};
use log::info;

pub async fn list_files(repo: &Repository, args: ListFileOpts) -> Result<(), RepoError> {
    // fetch computer info
    let computer = repo.get_computer(format_uuid(&args.computer)).await?;
    let mut folder = computer.get_folder(&format_uuid(&args.folder)).await?;

    info!("Folder: {:?}", folder.info);

    let latest_commit = folder.get_latest_commit().await?;

    let tree_index = folder.load_tree_index().await?;

    info!("Loaded tree index!");
    tree_index.load(&latest_commit).await?;

    // let tree = tree_index.load(latest_commit)?;
    unimplemented!()
}
