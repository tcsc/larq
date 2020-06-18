use crate::cli::ListFolderOpts;
use arq::Repository;
use log::info;
use uuid::Uuid;

pub async fn list_folders(repo: &Repository, args: ListFolderOpts) -> Result<(), ()> {
    let computer_id = args
        .computer
        .to_hyphenated_ref()
        .encode_upper(&mut Uuid::encode_buffer())
        .to_owned();

    let computer = repo.get_computer(&computer_id).await.map_err(|_| ())?;
    info!("{:?}", computer);

    let key = arq::encryption::CryptoKey::new(&args.password, "BucketPL".as_bytes()).map_err(|_| ())?;

    let folders = repo.list_folders(&computer_id, &key).await.map_err(|_| ())?;

    Ok(())
}
