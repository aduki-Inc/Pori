use include_dir::{include_dir, Dir};
use mime_guess::from_path;
use std::collections::HashMap;

/// Static files embedded at compile time
static STATIC_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/static");

/// Static file handler
pub struct StaticFileHandler {
    cache: HashMap<String, StaticFile>,
}

/// Cached static file
#[derive(Clone)]
pub struct StaticFile {
    pub content: Vec<u8>,
    pub mime_type: String,
    pub etag: String,
}

impl StaticFileHandler {
    /// Create new static file handler
    pub fn new() -> Self {
        let mut handler = Self {
            cache: HashMap::new(),
        };

        handler.load_static_files();
        handler
    }

    /// Load and cache all static files
    fn load_static_files(&mut self) {
        self.load_files_recursive(&STATIC_DIR, "");
    }

    /// Recursively load files from directory
    fn load_files_recursive(&mut self, dir: &Dir, prefix: &str) {
        for entry in dir.entries() {
            match entry {
                include_dir::DirEntry::Dir(subdir) => {
                    let new_prefix = if prefix.is_empty() {
                        subdir
                            .path()
                            .file_name()
                            .unwrap()
                            .to_str()
                            .unwrap()
                            .to_string()
                    } else {
                        format!(
                            "{}/{}",
                            prefix,
                            subdir.path().file_name().unwrap().to_str().unwrap()
                        )
                    };
                    self.load_files_recursive(subdir, &new_prefix);
                }
                include_dir::DirEntry::File(file) => {
                    let file_path = if prefix.is_empty() {
                        file.path()
                            .file_name()
                            .unwrap()
                            .to_str()
                            .unwrap()
                            .to_string()
                    } else {
                        format!(
                            "{}/{}",
                            prefix,
                            file.path().file_name().unwrap().to_str().unwrap()
                        )
                    };

                    let content = file.contents().to_vec();
                    let mime_type = from_path(&file_path).first_or_octet_stream().to_string();

                    // Generate ETag from content hash
                    let etag = format!("\"{}\"", self.generate_etag(&content));

                    self.cache.insert(
                        file_path,
                        StaticFile {
                            content,
                            mime_type,
                            etag,
                        },
                    );
                }
            }
        }
    }

    /// Get static file by path
    pub fn get_file(&self, path: &str) -> Option<&StaticFile> {
        // Normalize path (remove leading slash)
        let normalized_path = path.trim_start_matches('/');

        // Handle root path
        if normalized_path.is_empty() || normalized_path == "index.html" {
            return self.cache.get("index.html");
        }

        self.cache.get(normalized_path)
    }

    /// Check if path is a static file
    pub fn is_static_file(&self, path: &str) -> bool {
        self.get_file(path).is_some()
    }

    /// Get all available files (for debugging)
    pub fn list_files(&self) -> Vec<String> {
        self.cache.keys().cloned().collect()
    }

    /// Generate ETag from content
    fn generate_etag(&self, content: &[u8]) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Get cache statistics
    pub fn get_stats(&self) -> StaticFileStats {
        let total_files = self.cache.len();
        let total_size: usize = self.cache.values().map(|f| f.content.len()).sum();

        StaticFileStats {
            total_files,
            total_size,
        }
    }
}

impl Default for StaticFileHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Static file statistics
#[derive(Debug, Clone)]
pub struct StaticFileStats {
    pub total_files: usize,
    pub total_size: usize,
}

/// Create default static files if directory doesn't exist
pub fn create_default_static_files() -> HashMap<String, StaticFile> {
    let mut files = HashMap::new();

    // Default index.html
    let index_html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Tunnel Client Dashboard</title>
    <link rel="stylesheet" href="/css/main.css">
</head>
<body>
    <div class="container">
        <header>
            <h1>Tunnel Client Dashboard</h1>
            <div class="status-indicator" id="status">
                <span class="status-dot"></span>
                <span class="status-text">Connecting...</span>
            </div>
        </header>
        
        <main>
            <section class="stats-grid">
                <div class="stat-card">
                    <h3>Requests Processed</h3>
                    <div class="stat-value" id="requests-processed">0</div>
                </div>
                <div class="stat-card">
                    <h3>Successful</h3>
                    <div class="stat-value" id="requests-successful">0</div>
                </div>
                <div class="stat-card">
                    <h3>Failed</h3>
                    <div class="stat-value" id="requests-failed">0</div>
                </div>
                <div class="stat-card">
                    <h3>Bytes Transferred</h3>
                    <div class="stat-value" id="bytes-transferred">0 B</div>
                </div>
            </section>
            
            <section class="logs">
                <h2>Recent Activity</h2>
                <div class="log-container" id="log-container">
                    <div class="log-entry">Dashboard started</div>
                </div>
            </section>
            
            <section class="controls">
                <button id="refresh-btn" class="btn btn-primary">Refresh</button>
                <button id="clear-logs-btn" class="btn btn-secondary">Clear Logs</button>
            </section>
        </main>
    </div>
    
    <script src="/js/main.js"></script>
</body>
</html>"#;

    files.insert(
        "index.html".to_string(),
        StaticFile {
            content: index_html.as_bytes().to_vec(),
            mime_type: "text/html".to_string(),
            etag: "\"default-index\"".to_string(),
        },
    );

    files
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_static_file_handler() {
        let handler = StaticFileHandler::new();

        // Test path normalization
        assert_eq!(
            handler.get_file("/test.html").is_some(),
            handler.get_file("test.html").is_some()
        );
    }

    #[test]
    fn test_default_static_files() {
        let files = create_default_static_files();

        assert!(files.contains_key("index.html"));

        let index = &files["index.html"];
        assert_eq!(index.mime_type, "text/html");
        assert!(!index.content.is_empty());
    }
}
