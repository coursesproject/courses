use anyhow::anyhow;
use blake3::Hash;
use cdoc::config::Format;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Cache {
    content_files: HashMap<String, FileInfo>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct FileInfo {
    hash: Hash,
    build_succeeded: BTreeMap<String, bool>,
}

impl FileInfo {
    pub fn build_status_all(&self) -> bool {
        self.build_succeeded.values().all(|v| *v)
    }

    pub fn build_status_format<F: Format>(&self, format: F) -> Option<&bool> {
        self.build_succeeded.get(format.name())
    }

    pub fn set_status(&mut self, format: &str, build_succeeded: bool) -> Option<bool> {
        self.build_succeeded
            .insert(format.to_string(), build_succeeded)
    }
}

impl Cache {
    pub fn exists(&self, path: &str) -> bool {
        self.content_files.contains_key(path)
    }

    pub fn matches(&self, path: &str, hash: Hash) -> bool {
        self.content_files
            .get(path)
            .map(|h| h.hash == hash)
            .unwrap_or_default()
    }

    pub fn should_rebuild_all(&self, path: &str, hash: Hash) -> bool {
        self.content_files
            .get(path)
            .map(|h| h.build_status_all() && h.hash == hash)
            .unwrap_or_default()
    }

    pub fn should_rebuild_format<F: Format>(&self, path: &str, format: F) -> bool {
        self.content_files
            .get(path)
            .map(|h| *h.build_status_format(format).unwrap_or(&false))
            .unwrap_or_default()
    }

    pub fn reset_entry(&mut self, path: String, hash: Hash) {
        self.content_files.insert(
            path,
            FileInfo {
                hash,
                build_succeeded: Default::default(),
            },
        );
    }

    pub fn update_build_status(
        &mut self,
        path: String,
        format: &str,
        build_succeeded: bool,
    ) -> anyhow::Result<()> {
        self.content_files
            .get_mut(&path)
            .ok_or_else(|| anyhow!("path not found in cache"))?
            .set_status(format, build_succeeded);
        Ok(())
    }
}
