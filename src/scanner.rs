use ignore::WalkBuilder;
use std::collections::HashSet;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub fn scan(root: &str, cache_file: Option<&PathBuf>) -> Vec<PathBuf> {
    if let Some(path) = cache_file
        && path.exists()
        && let Ok(file) = File::open(path)
    {
        let reader = BufReader::new(file);
        if let Ok(repos) = serde_json::from_reader::<_, Vec<PathBuf>>(reader) {
            return repos;
        }
    }

    let repos = Arc::new(Mutex::new(HashSet::new()));
    let repos_clone = repos.clone();

    WalkBuilder::new(root)
        .threads(num_cpus::get())
        .follow_links(true)
        .standard_filters(false) // Don't respect gitignore for the repo search itself, we want to find repos!
        .hidden(false) // We need to see .git
        .filter_entry(|e| !is_ignored(e.file_name().to_str().unwrap_or("")))
        .build_parallel()
        .run(move || {
            let repos = repos_clone.clone();
            Box::new(move |entry| {
                if let Ok(entry) = entry
                    && is_repo_marker(entry.file_name().to_str().unwrap_or(""), entry.file_type())
                    && let Some(parent) = entry.path().parent()
                {
                    repos.lock().unwrap().insert(parent.to_path_buf());
                }
                ignore::WalkState::Continue
            })
        });

    let set = Arc::try_unwrap(repos).unwrap().into_inner().unwrap();
    let result: Vec<PathBuf> = set.into_iter().collect();

    if let Some(path) = cache_file {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(file) = File::create(path) {
            let _ = serde_json::to_writer(file, &result);
        }
    }

    result
}

fn is_ignored(name: &str) -> bool {
    name == "node_modules" || name == "vendor"
}

fn is_repo_marker(name: &str, ft: Option<std::fs::FileType>) -> bool {
    (name == ".git" || name == ".jj") && ft.is_some_and(|t| t.is_dir())
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

        let found = scan(dir.path().to_str().unwrap(), None);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0], repo_dir);
    }

    #[test]
    fn test_scan_jj_repo() {
        let dir = tempdir().unwrap();
        let repo_dir = dir.path().join("jj-repo");
        fs::create_dir(&repo_dir).unwrap();
        fs::create_dir(repo_dir.join(".jj")).unwrap();

        let found = scan(dir.path().to_str().unwrap(), None);
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

        let found = scan(dir.path().to_str().unwrap(), None);
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

        let found = scan(dir.path().to_str().unwrap(), None);
        assert_eq!(found.len(), 0);
    }

    #[test]
    fn test_ignores_vendor() {
        let dir = tempdir().unwrap();
        let vendor_dir = dir.path().join("vendor");
        fs::create_dir(&vendor_dir).unwrap();
        let sub_git = vendor_dir.join("dep").join(".git");
        fs::create_dir_all(&sub_git).unwrap();

        let found = scan(dir.path().to_str().unwrap(), None);
        assert_eq!(found.len(), 0);
    }
}
