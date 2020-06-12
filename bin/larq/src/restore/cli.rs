use gumdrop::Options;
use std::fs::canonicalize;
use std::io::Write;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Options)]
pub enum Command {
    ListComputers (ListComputerOpts),
    ListFolders (ListFolderOpts)
}

#[derive(Debug, Options)]
pub struct ListComputerOpts {}

#[derive(Debug, Options)]
pub struct ListFolderOpts {
    #[options(help = "The computer to operate on", meta = "UUID")]
    pub computer: Uuid
}

#[derive(Debug, Options)]
pub struct Args {
    #[options(help = "Use config file")]
    pub config_file: PathBuf,

    #[options(help = "Be more verbose")]
    pub verbose: bool,

    #[options(help = "Print help message and exit")]
    help: bool,

    #[options(command)]
    pub cmd: Option<Command>
}

#[cfg(test)]
mod test {

}
