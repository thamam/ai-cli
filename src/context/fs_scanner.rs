use anyhow::Result;
use ignore::WalkBuilder;
use std::path::Path;

/// Scans the file system respecting .gitignore
pub struct FileScanner;

impl FileScanner {
    /// Scan directory and return relevant files (respecting .gitignore)
    /// Limited to text files under 100KB
    pub fn scan_directory(path: &Path, max_files: usize) -> Result<Vec<(String, String)>> {
        let mut files = Vec::new();

        let walker = WalkBuilder::new(path)
            .hidden(false)
            .git_ignore(true)
            .git_global(true)
            .git_exclude(true)
            .build();

        for entry in walker.take(max_files) {
            match entry {
                Ok(entry) => {
                    let path = entry.path();

                    // Skip directories
                    if !path.is_file() {
                        continue;
                    }

                    // Check file size (max 100KB)
                    if let Ok(metadata) = path.metadata() {
                        if metadata.len() > 100_000 {
                            continue;
                        }
                    }

                    // Try to read as text
                    if let Ok(content) = std::fs::read_to_string(path) {
                        if let Some(filename) = path.file_name() {
                            files.push((
                                filename.to_string_lossy().to_string(),
                                content,
                            ));
                        }
                    }
                }
                Err(_) => continue,
            }
        }

        Ok(files)
    }

    /// Get recently modified files using git (if available)
    pub fn get_modified_files(path: &Path) -> Result<Vec<String>> {
        // TODO: Implement git status integration
        // For now, return empty
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_scan_directory() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "Hello, World!").unwrap();

        let files = FileScanner::scan_directory(temp_dir.path(), 10).unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].0, "test.txt");
        assert_eq!(files[0].1, "Hello, World!");
    }
}
