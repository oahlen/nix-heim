#[cfg(test)]
pub mod tests {
    use std::{fs, io::Write, path::PathBuf};

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
}
