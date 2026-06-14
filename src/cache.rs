use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

const CACHE_FILE: &str = ".reefmt-cache";
const CACHE_VERSION: u8 = 1;

/// Tracks file modification times to avoid re-processing unchanged files.
#[derive(Serialize, Deserialize)]
pub(crate) struct FormatCache {
    version: u8,
    files: HashMap<String, u128>,
}

impl FormatCache {
    /// Load the cache from disk. Returns an empty cache if the file doesn't
    /// exist, has an incompatible version, or is corrupt.
    pub(crate) fn load() -> Self {
        let path = Path::new(CACHE_FILE);
        if path.exists() {
            if let Ok(content) = fs::read_to_string(path) {
                if let Ok(cache) = serde_json::from_str::<FormatCache>(&content) {
                    if cache.version == CACHE_VERSION {
                        return cache;
                    }
                }
            }
        }
        FormatCache {
            version: CACHE_VERSION,
            files: HashMap::new(),
        }
    }

    /// Save the cache to disk atomically (write to temp file, then rename).
    pub(crate) fn save(&self) {
        if let Ok(content) = serde_json::to_string(self) {
            let tmp_path = format!("{}.tmp", CACHE_FILE);
            if fs::write(&tmp_path, &content).is_ok() {
                let _ = fs::rename(&tmp_path, CACHE_FILE);
            }
        }
    }

    /// Returns `true` if the file's current mtime matches the cached value,
    /// meaning it hasn't changed since the last format run.
    pub(crate) fn is_fresh(&self, path: &Path) -> bool {
        let Some(cached_mtime) = self.lookup(path) else {
            return false;
        };
        let Some(current_mtime) = get_mtime_ns(path) else {
            return false;
        };
        current_mtime == cached_mtime
    }

    /// Update the cache entry for a file with its current mtime.
    pub(crate) fn mark_fresh(&mut self, path: &Path) {
        let Some(key) = canonical_key(path) else {
            return;
        };
        if let Some(mtime) = get_mtime_ns(path) {
            self.files.insert(key, mtime);
        }
    }

    fn lookup(&self, path: &Path) -> Option<u128> {
        let key = canonical_key(path)?;
        self.files.get(&key).copied()
    }
}

/// Get the file's mtime in nanoseconds since UNIX_EPOCH.
fn get_mtime_ns(path: &Path) -> Option<u128> {
    fs::metadata(path)
        .ok()?
        .modified()
        .ok()?
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .map(|d| d.as_nanos())
}

/// Canonicalize a path to an absolute string key for the cache.
fn canonical_key(path: &Path) -> Option<String> {
    let abs = fs::canonicalize(path).ok()?;
    abs.to_str().map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;

    #[test]
    fn loads_empty_cache_when_no_file() {
        let cache = FormatCache::load();
        assert!(cache.files.is_empty());
        assert_eq!(cache.version, CACHE_VERSION);
    }

    #[test]
    fn mark_and_check_freshness() {
        let dir = env::temp_dir().join("reefmt_cache_test_mark");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.ree");
        fs::write(&path, "content").unwrap();

        let mut cache = FormatCache::load();
        assert!(!cache.is_fresh(&path), "file not cached yet");

        cache.mark_fresh(&path);
        assert!(cache.is_fresh(&path), "file should be fresh after marking");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn freshness_invalidated_after_modification() {
        let dir = env::temp_dir().join("reefmt_cache_test_invalidate");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.ree");
        fs::write(&path, "content v1").unwrap();

        let mut cache = FormatCache::load();
        cache.mark_fresh(&path);
        assert!(cache.is_fresh(&path));

        // Wait a tick to ensure different mtime
        std::thread::sleep(std::time::Duration::from_millis(10));
        fs::write(&path, "content v2").unwrap();

        assert!(!cache.is_fresh(&path), "should be stale after modification");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn save_and_load_roundtrip() {
        let dir = env::temp_dir().join("reefmt_cache_test_roundtrip");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.ree");
        fs::write(&path, "content").unwrap();

        // Mark and save
        let mut cache = FormatCache::load();
        cache.mark_fresh(&path);

        // Serialize and deserialize manually to test roundtrip
        let json = serde_json::to_string(&cache).unwrap();
        let loaded: FormatCache = serde_json::from_str(&json).unwrap();

        assert!(loaded.is_fresh(&path), "freshness should survive roundtrip");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn skips_corrupted_cache() {
        let cache_file = Path::new(CACHE_FILE);
        // Write garbage
        fs::write(cache_file, "this is not json").ok();
        let cache = FormatCache::load();
        assert!(cache.files.is_empty(), "should return empty cache on corruption");

        // Clean up
        let _ = fs::remove_file(cache_file);
    }

    #[test]
    fn skips_wrong_version_cache() {
        let cache_file = Path::new(CACHE_FILE);
        let bad = r#"{"version":99,"files":{}}"#;
        fs::write(cache_file, bad).ok();
        let cache = FormatCache::load();
        assert!(cache.files.is_empty(), "should return empty cache for wrong version");

        let _ = fs::remove_file(cache_file);
    }

    #[test]
    fn non_existent_file_is_not_fresh() {
        let cache = FormatCache::load();
        let path = Path::new("/nonexistent/path/to/file.ree");
        assert!(!cache.is_fresh(path), "non-existent file should not be fresh");
    }

    #[test]
    fn ignore_paths_outside_project_are_not_in_cache() {
        let mut cache = FormatCache::load();
        let outside = Path::new("/tmp/some_file.ree");
        assert!(!cache.is_fresh(outside), "outside paths should not be fresh");
        cache.mark_fresh(outside);
        // tmp might not have a canonicalizable path but shouldn't crash
    }
}
