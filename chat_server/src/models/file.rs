use sha1::{Digest, Sha1};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct ChatFile {
    pub ext: String,
    pub hash: String,
}

impl ChatFile {
    pub fn new(filename: &str, data: &[u8]) -> Self {
        let hash = Sha1::digest(data);
        Self {
            ext: filename.rsplit(".").next().unwrap_or("txt").to_string(),
            hash: hex::encode(hash),
        }
    }

    pub fn url(&self, ws_id: String) -> String {
        format!("/files/{}/{}", ws_id, self.hash_to_path())
    }

    pub fn path(&self, base_dir: &Path) -> PathBuf {
        base_dir.join(self.hash_to_path())
    }

    pub fn hash_to_path(&self) -> String {
        let (part1, part2) = self.hash.split_at(3);
        let (part2, part3) = part2.split_at(3);
        format!("{}/{}/{}.{}", part1, part2, part3, self.ext)
    }
}

#[cfg(test)]
mod tests {

    // cargo test --package chat_server --lib -- models::file::tests::test_find_ext --exact --show-output
    #[test]
    fn test_find_ext() {
        let name = "Capture-2024-10-06-210908.png";
        let ext = name.rsplit(".").next().unwrap_or("txt");
        println!("{:?}", ext);
        assert_eq!(ext, "png");
    }
}
