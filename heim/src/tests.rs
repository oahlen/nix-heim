#[cfg(test)]
pub mod tests {
    use std::{fs, io::Write, path::PathBuf};

    use crate::entry::{FileEntry, SourceEntry};

    pub fn test_dir() -> PathBuf {
        let id = uuid::Uuid::new_v4();
        let dir = std::env::temp_dir().join(format!("heim_test_{}", id));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    pub fn write_file(dir: &PathBuf, name: &str, content: &str) -> PathBuf {
        let path = dir.join(name);

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }

        let mut f = fs::File::create(&path).unwrap();
        f.write_all(content.as_bytes()).unwrap();
        path
    }

    pub fn make_symlink(dir: &PathBuf, name: &str, source: &PathBuf) -> PathBuf {
        let target = dir.join(name);

        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).unwrap();
        }

        std::os::unix::fs::symlink(&source, &target).unwrap();
        target
    }

    impl FileEntry {
        pub fn create(source: PathBuf, target: PathBuf, overwrite: bool) -> FileEntry {
            FileEntry::new(
                vec![SourceEntry {
                    source,
                    name: String::from("default"),
                    default: true,
                }],
                target,
                overwrite,
            )
        }
    }
}
