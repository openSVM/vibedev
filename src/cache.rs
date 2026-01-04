// Binary cache module using serde_json for serialization
use anyhow::Result;
use serde::{de::DeserializeOwned, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

/// Get the cache directory
pub fn get_cache_dir() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("vibedev")
        .join("cache")
}

/// Cache key from a string
fn cache_path(key: &str) -> PathBuf {
    get_cache_dir().join(format!("{}.json", key))
}

/// Save data to cache
pub fn cache_save<T: Serialize>(key: &str, data: &T) -> Result<()> {
    let cache_dir = get_cache_dir();
    fs::create_dir_all(&cache_dir)?;

    let bytes = serde_json::to_vec(data)?;
    let path = cache_path(key);
    fs::write(&path, &bytes)?;

    Ok(())
}

/// Load data from cache
pub fn cache_load<T: DeserializeOwned>(key: &str) -> Result<Option<T>> {
    let path = cache_path(key);

    if !path.exists() {
        return Ok(None);
    }

    let bytes = fs::read(&path)?;
    let data: T = serde_json::from_slice(&bytes)?;

    Ok(Some(data))
}

/// Load from cache if fresh, otherwise return None
pub fn cache_load_fresh<T: DeserializeOwned>(key: &str, max_age: Duration) -> Result<Option<T>> {
    let path = cache_path(key);

    if !path.exists() {
        return Ok(None);
    }

    // Check age
    let metadata = fs::metadata(&path)?;
    let modified = metadata.modified()?;
    let age = SystemTime::now()
        .duration_since(modified)
        .unwrap_or(Duration::MAX);

    if age > max_age {
        return Ok(None);
    }

    let bytes = fs::read(&path)?;
    let data: T = serde_json::from_slice(&bytes)?;

    Ok(Some(data))
}

/// Clear a specific cache entry
pub fn cache_clear(key: &str) -> Result<()> {
    let path = cache_path(key);
    if path.exists() {
        fs::remove_file(&path)?;
    }
    Ok(())
}

/// Clear all cache
pub fn cache_clear_all() -> Result<()> {
    let cache_dir = get_cache_dir();
    if cache_dir.exists() {
        fs::remove_dir_all(&cache_dir)?;
    }
    Ok(())
}

/// Get cache stats
pub fn cache_stats() -> Result<CacheStats> {
    let cache_dir = get_cache_dir();

    if !cache_dir.exists() {
        return Ok(CacheStats::default());
    }

    let mut total_size = 0u64;
    let mut file_count = 0usize;

    for entry in fs::read_dir(&cache_dir)? {
        let entry = entry?;
        if entry.path().extension().is_some_and(|e| e == "json") {
            total_size += entry.metadata()?.len();
            file_count += 1;
        }
    }

    Ok(CacheStats {
        total_size,
        file_count,
        cache_dir,
    })
}

#[derive(Default)]
pub struct CacheStats {
    pub total_size: u64,
    pub file_count: usize,
    pub cache_dir: PathBuf,
}

impl CacheStats {
    pub fn format_size(&self) -> String {
        if self.total_size >= 1024 * 1024 {
            format!("{:.2} MB", self.total_size as f64 / (1024.0 * 1024.0))
        } else if self.total_size >= 1024 {
            format!("{:.2} KB", self.total_size as f64 / 1024.0)
        } else {
            format!("{} B", self.total_size)
        }
    }
}
