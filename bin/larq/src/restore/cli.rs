use std::io::Write;
use std::fs::canonicalize;
use std::path::PathBuf;
use argparse::{ArgumentParser, Parse, StoreTrue};

pub struct Args {
    pub config_file: PathBuf,
    pub verbose: bool,
}

impl Args {
    pub fn parse(args: Vec<String>,
                 stdout: &mut Write,
                 stderr: &mut Write) -> Result<Args, i32> {

        let mut result = Args {
            config_file: PathBuf::new(),
            verbose: false,
        };

        let rval = {
            let mut ap = ArgumentParser::new();
            ap.set_description("Restore Arq backups");
            ap.refer(&mut result.config_file)
                .add_option(&["-c", "--config-file"],
                            Parse,
                            "Use config file");

            ap.refer(&mut result.verbose)
                .add_option(&["-v", "--verbose"],
                            StoreTrue,
                            "Say more");

            ap.parse(args, stdout, stderr)
        };

        rval.map(|_| result)
    }
}

#[cfg(test)]
mod test {

}
