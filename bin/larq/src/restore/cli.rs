use gumdrop::Options;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Options)]
pub enum Command {
    ListComputers(ListComputerOpts),
    ListFolders(ListFolderOpts),
    Restore(RestoreOpts),
}

#[derive(Debug, Options)]
pub struct ListComputerOpts {}

#[derive(Debug, Options)]
#[options(required)]
pub struct ListFolderOpts {
    #[options(help = "The computer to operate on", meta = "UUID")]
    pub computer: Uuid,
}

#[derive(Debug, Options)]
pub struct RestoreOpts {
    #[options(help = "The computer to operate on", meta = "UUID")]
    pub computer: Uuid,

    #[options(help = "A regex describing the path of the file(s) to restore")]
    pub path: String,
}

#[derive(Debug, Options)]
pub struct Args {
    #[options(help = "Use config file")]
    pub config_file: PathBuf,

    #[options(help = "Be more verbose")]
    pub verbose: bool,

    #[options(help = "Print help message and exit")]
    help: bool,

    #[options(help = "Encryption password", meta = "PWD")]
    pub password: String,

    #[options(command)]
    pub cmd: Option<Command>,
}

#[cfg(test)]
mod test {}
