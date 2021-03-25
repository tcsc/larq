use std::path::PathBuf;

use arq_storage::Key;

pub struct Cache {
    root: Option<PathBuf>,
}

impl Cache {
    pub fn new(path: Option<PathBuf>) -> Cache {
        Cache { root: path }
    }

    pub fn read(&self, key: &Key) -> Option<Vec<u8>> {
        use std::io::Read;

        if let Some(p) = self.root.as_ref() {
            let path = p.join(key.as_str());

            if let Ok(mut f) = std::fs::File::open(&path) {
                let mut content = Vec::new();
                if f.read_to_end(&mut content).is_ok() {
                    log::debug!("Found in cache");
                    return Some(content);
                }
            }
        }

        None
    }

    pub fn write(&self, key: &Key, data: &[u8]) {
        use std::io::Write;

        if let Some(p) = self.root.as_ref() {
            let path = p.join(key.as_str());
            let tmp = path.with_extension(".tmp");

            if let Some(parent_dir) = path.parent() {
                std::fs::create_dir_all(parent_dir);
            }

            if let Ok(mut f) = std::fs::OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&tmp)
            {
                f.write_all(data);
                drop(f);

                std::fs::rename(&tmp, &path);
            }
        }
    }
}
