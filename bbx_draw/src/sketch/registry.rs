//! Sketch discovery and registry.

use std::{fs, io, path::PathBuf, time::SystemTime};

use serde::{Deserialize, Serialize};

/// Metadata about a discovered sketch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SketchMetadata {
    /// The display name of the sketch.
    pub name: String,
    /// A brief description.
    pub description: String,
    /// Path to the sketch source file.
    pub source_path: PathBuf,
    /// Last modification time.
    #[serde(with = "system_time_serde")]
    pub last_modified: SystemTime,
}

/// Registry for discovering and caching sketch metadata.
pub struct SketchRegistry {
    cache_dir: PathBuf,
    entries: Vec<SketchMetadata>,
}

impl SketchRegistry {
    /// Create a new sketch registry.
    ///
    /// Uses the platform's cache directory (~/.cache/bbx_draw/sketches on Linux,
    /// ~/Library/Caches/bbx_draw/sketches on macOS, etc.).
    pub fn new() -> io::Result<Self> {
        let cache_dir = dirs::cache_dir()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "No cache directory found"))?
            .join("bbx_draw")
            .join("sketches");

        fs::create_dir_all(&cache_dir)?;

        let mut registry = Self {
            cache_dir,
            entries: Vec::new(),
        };

        registry.load_cache()?;
        Ok(registry)
    }

    /// Create a registry with a custom cache directory.
    pub fn with_cache_dir(cache_dir: PathBuf) -> io::Result<Self> {
        fs::create_dir_all(&cache_dir)?;

        let mut registry = Self {
            cache_dir,
            entries: Vec::new(),
        };

        registry.load_cache()?;
        Ok(registry)
    }

    /// Get the cache directory path.
    pub fn cache_dir(&self) -> &PathBuf {
        &self.cache_dir
    }

    /// Discover sketches in a directory and update the registry.
    pub fn discover(&mut self, search_dir: &PathBuf) -> io::Result<usize> {
        let mut discovered = 0;

        if !search_dir.exists() {
            return Ok(0);
        }

        for entry in fs::read_dir(search_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().is_some_and(|ext| ext == "rs")
                && let Ok(metadata) = Self::extract_metadata(&path)
                && !self.entries.iter().any(|e| e.source_path == metadata.source_path)
            {
                self.entries.push(metadata);
                discovered += 1;
            }
        }

        self.save_cache()?;
        Ok(discovered)
    }

    /// List all registered sketches.
    pub fn list(&self) -> &[SketchMetadata] {
        &self.entries
    }

    /// Get a sketch by name.
    pub fn get(&self, name: &str) -> Option<&SketchMetadata> {
        self.entries.iter().find(|e| e.name == name)
    }

    /// Remove a sketch from the registry.
    pub fn remove(&mut self, name: &str) -> Option<SketchMetadata> {
        if let Some(pos) = self.entries.iter().position(|e| e.name == name) {
            let removed = self.entries.remove(pos);
            let _ = self.save_cache();
            Some(removed)
        } else {
            None
        }
    }

    /// Add or update a sketch in the registry.
    pub fn register(&mut self, metadata: SketchMetadata) -> io::Result<()> {
        if let Some(existing) = self.entries.iter_mut().find(|e| e.name == metadata.name) {
            *existing = metadata;
        } else {
            self.entries.push(metadata);
        }
        self.save_cache()
    }

    fn cache_file_path(&self) -> PathBuf {
        self.cache_dir.join("registry.json")
    }

    fn load_cache(&mut self) -> io::Result<()> {
        let cache_file = self.cache_file_path();
        if cache_file.exists() {
            let content = fs::read_to_string(&cache_file)?;
            self.entries = serde_json::from_str(&content).unwrap_or_default();
        }
        Ok(())
    }

    fn save_cache(&self) -> io::Result<()> {
        let cache_file = self.cache_file_path();
        let content = serde_json::to_string_pretty(&self.entries)?;
        fs::write(cache_file, content)
    }

    fn extract_metadata(path: &PathBuf) -> io::Result<SketchMetadata> {
        let content = fs::read_to_string(path)?;
        let file_meta = fs::metadata(path)?;

        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let description = Self::extract_doc_comment(&content).unwrap_or_else(|| format!("Sketch: {name}"));

        Ok(SketchMetadata {
            name,
            description,
            source_path: path.clone(),
            last_modified: file_meta.modified()?,
        })
    }

    fn extract_doc_comment(content: &str) -> Option<String> {
        let mut doc_lines = Vec::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("//!") {
                doc_lines.push(trimmed.trim_start_matches("//!").trim());
            } else if !trimmed.is_empty() && !trimmed.starts_with("//") {
                break;
            }
        }

        if doc_lines.is_empty() {
            None
        } else {
            Some(doc_lines.join(" "))
        }
    }
}

mod system_time_serde {
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(time: &SystemTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let duration = time.duration_since(UNIX_EPOCH).unwrap_or(Duration::ZERO);
        duration.as_secs().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<SystemTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(UNIX_EPOCH + Duration::from_secs(secs))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_doc_comment() {
        let content = r#"//! This is a test sketch.
//! It does something cool.

use something;
"#;
        let desc = SketchRegistry::extract_doc_comment(content);
        assert_eq!(desc, Some("This is a test sketch. It does something cool.".to_string()));
    }

    #[test]
    fn test_extract_no_doc_comment() {
        let content = "use something;\nfn main() {}";
        let desc = SketchRegistry::extract_doc_comment(content);
        assert_eq!(desc, None);
    }
}
