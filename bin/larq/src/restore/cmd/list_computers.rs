use log::{error, info};
use arq::Repository;
use futures::{Future, future};

pub fn list_computers(repo: &Repository) -> Box<dyn Future<Item = (), Error = ()> + Send>
{
    let f = repo.list_computers()
        .and_then(|computers| {
            for c in computers.iter() {
                info!("id: {}, user: {}, name: {}", c.id, c.user, c.computer)
            }
            future::ok(())
        })
        .or_else(|e| {
            error!("Listing failed with error: {:?}", e);
            future::err(())
        });
    Box::new(f)
}