use std::path::PathBuf;
use std::collections::HashSet;
use walkdir::{DirEntry, WalkDir};

pub fn scan(root: &str) -> Vec<PathBuf> {
    let mut repos = HashSet::new();

    let walker = WalkDir::new(root).follow_links(true).into_iter();

    for entry in walker.filter_entry(|e| !is_ignored(e)).flatten() {
        if !is_repo_marker(&entry) {
            continue;
        }
        if let Some(parent) = entry.path().parent() {
            repos.insert(parent.to_path_buf());
        }
    }

    repos.into_iter().collect()
}

fn is_ignored(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s == "node_modules" || s == "vendor")
        .unwrap_or(false)
}

fn is_repo_marker(entry: &DirEntry) -> bool {
    let name = entry.file_name().to_str().unwrap_or("");
    // We look for the directory itself, e.g. /path/to/repo/.git
    (name == ".git" || name == ".jj") && entry.file_type().is_dir()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_scan_git_repo() {
        let dir = tempdir().unwrap();
        let repo_dir = dir.path().join("my-repo");
        fs::create_dir(&repo_dir).unwrap();
        fs::create_dir(repo_dir.join(".git")).unwrap();

        let found = scan(dir.path().to_str().unwrap());
        assert_eq!(found.len(), 1);
        assert_eq!(found[0], repo_dir);
    }

    #[test]
    fn test_scan_jj_repo() {
        let dir = tempdir().unwrap();
        let repo_dir = dir.path().join("jj-repo");
        fs::create_dir(&repo_dir).unwrap();
        fs::create_dir(repo_dir.join(".jj")).unwrap();

        let found = scan(dir.path().to_str().unwrap());
        assert_eq!(found.len(), 1);
        assert_eq!(found[0], repo_dir);
    }

    #[test]
    fn test_deduplication() {
        let dir = tempdir().unwrap();
        let repo_dir = dir.path().join("dual-repo");
        fs::create_dir(&repo_dir).unwrap();
        fs::create_dir(repo_dir.join(".git")).unwrap();
        fs::create_dir(repo_dir.join(".jj")).unwrap();

        let found = scan(dir.path().to_str().unwrap());
        assert_eq!(found.len(), 1);
        assert_eq!(found[0], repo_dir);
    }

    #[test]
    fn test_ignores_node_modules() {
        let dir = tempdir().unwrap();
        let node_dir = dir.path().join("node_modules");
        fs::create_dir(&node_dir).unwrap();
        // Even if there is a .git inside node_modules (e.g. from a dependency), it should be ignored by the walker filter
        let sub_git = node_dir.join("dep").join(".git");
        fs::create_dir_all(&sub_git).unwrap();

        let found = scan(dir.path().to_str().unwrap());
        assert_eq!(found.len(), 0);
    }

    #[test]
    fn test_ignores_vendor() {
        let dir = tempdir().unwrap();
        let vendor_dir = dir.path().join("vendor");
        fs::create_dir(&vendor_dir).unwrap();
        let sub_git = vendor_dir.join("dep").join(".git");
        fs::create_dir_all(&sub_git).unwrap();

        let found = scan(dir.path().to_str().unwrap());
        assert_eq!(found.len(), 0);
    }
}
