use arq::Repository;
use log::{error, info};

pub async fn list_computers(repo: &Repository) -> Result<(), ()> {
    let computers = repo.list_computers().await.map_err(|e| {
        error!("Listing failed with error: {:?}", e);
    })?;

    info!("!!!");

    for c in computers.iter() {
        info!("id: {}, user: {}, name: {}", c.id, c.user, c.computer)
    }

    Ok(())
}
