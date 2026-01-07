use anyhow::{Result, anyhow};
use chrono::{DateTime, Duration, NaiveDate, Utc};
use rayon::prelude::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
// use gix::bstr::ByteSlice;

// JJ imports - tentative based on common API patterns
// If these fail, we might need to adjust or fallback to CLI
// We will comment them out for the first pass and verify dependencies
// use jj_lib::workspace::Workspace;
// use jj_lib::settings::UserSettings;

pub type CommitCounts = HashMap<NaiveDate, i32>;

const DAYS_IN_LAST_SIX_MONTHS: i64 = 183;

pub fn process_repositories(repos: Vec<PathBuf>, email: &str) -> CommitCounts {
    // Convert email to bytes for gix comparison
    let email_bytes = email.as_bytes().to_vec();

    repos
        .par_iter()
        .fold(HashMap::new, |mut acc: CommitCounts, path| {
            let mut repo_commits = HashMap::new();

            // Detect repo type
            let git_dir = path.join(".git");
            let jj_dir = path.join(".jj");

            let six_months_ago = std::time::SystemTime::now()
                .checked_sub(std::time::Duration::from_secs(
                    DAYS_IN_LAST_SIX_MONTHS as u64 * 24 * 60 * 60,
                ))
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH);

            if git_dir.exists() {
                // Optimization: Check if HEAD has been modified recently
                if let Ok(metadata) = std::fs::metadata(git_dir.join("HEAD"))
                    && let Ok(mtime) = metadata.modified()
                    && mtime < six_months_ago
                {
                    return acc; // Skip stale repo
                }

                if let Err(_e) = process_git(path, &email_bytes, &mut repo_commits) {
                    // Silently ignore errors
                }
            } else if jj_dir.exists() {
                // Optimization for JJ
                if let Ok(metadata) = std::fs::metadata(&jj_dir)
                    && let Ok(mtime) = metadata.modified()
                    && mtime < six_months_ago
                {
                    return acc;
                }

                if process_jj(path, email, &mut repo_commits).is_err() {
                    // Silently ignore errors
                }
            }

            // Merge local repo stats into the fold accumulator
            for (date, count) in repo_commits {
                *acc.entry(date).or_insert(0) += count;
            }
            acc
        })
        .reduce(HashMap::new, |mut a, b| {
            for (date, count) in b {
                *a.entry(date).or_insert(0) += count;
            }
            a
        })
}

fn process_git(path: &Path, email: &[u8], commits: &mut CommitCounts) -> Result<()> {
    // Open repo
    let repo = gix::open(path)?;

    // HEAD
    let head = repo.head()?;
    // gix::Head usually has id() in recent versions
    let head_id = head.id().ok_or_else(|| anyhow!("No HEAD"))?;

    // Revwalk
    let commit_graph = repo.rev_walk(Some(head_id.detach())).all()?;

    let cutoff_date = Utc::now() - Duration::days(DAYS_IN_LAST_SIX_MONTHS);

    for info in commit_graph {
        let info = info?;
        let commit = info.object()?;
        let author = commit.author()?;

        // Compiler indicates author.time is &str in this context/version
        let time_str: &str = author.time;
        let seconds = time_str
            .split_whitespace()
            .next()
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(0);

        // gix time is seconds since epoch
        let datetime =
            DateTime::from_timestamp(seconds, 0).ok_or_else(|| anyhow!("Invalid timestamp"))?;

        if datetime < cutoff_date {
            // Optimization: Stop traversing if we are too far back.
            // Git history is topologically sorted (parents come after children),
            // so once we hit a date older than our window, we can safely stop.
            break;
        }

        if author.email != email {
            continue;
        }

        let utc_date = datetime.date_naive();
        *commits.entry(utc_date).or_insert(0) += 1;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_process_git_repo() -> Result<()> {
        let dir = tempdir()?;
        let repo_path = dir.path().join("my-repo");
        // Use gix init just to make sure dir exists or standard fs
        std::fs::create_dir_all(&repo_path)?;

        let email = "test@example.com";

        // Setup git repo using CLI for simplicity and control over timestamps if needed

        let status = std::process::Command::new("git")
            .arg("init")
            .current_dir(&repo_path)
            .status()?;

        assert!(status.success(), "git init failed");

        let status = std::process::Command::new("git")
            .args(&["config", "user.email", email])
            .current_dir(&repo_path)
            .status()?;

        assert!(status.success(), "git config email failed");

        let status = std::process::Command::new("git")
            .args(&["config", "user.name", "Test User"])
            .current_dir(&repo_path)
            .status()?;

        assert!(status.success(), "git config name failed");

        let status = std::process::Command::new("git")
            .args(&["config", "commit.gpgsign", "false"])
            .current_dir(&repo_path)
            .status()?;

        assert!(status.success(), "git config gpg failed");

        // Commit "today"

        std::fs::write(repo_path.join("file"), "content")?;

        let status = std::process::Command::new("git")
            .args(&["add", "file"])
            .current_dir(&repo_path)
            .status()?;

        assert!(status.success(), "git add failed");

        let output = std::process::Command::new("git")
            .args(&["commit", "-m", "msg"])
            .current_dir(&repo_path)
            .output()?;

        if !output.status.success() {
            panic!(
                "Commit failed: {:?}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        let repos = vec![repo_path];
        let stats = process_repositories(repos, email);

        let today = Utc::now().date_naive();
        assert_eq!(stats.get(&today), Some(&1));

        Ok(())
    }
}

fn process_jj(path: &Path, email: &str, commits: &mut CommitCounts) -> Result<()> {
    use std::process::Command;

    // Use a specific date format: YYYY-MM-DD

    let output = Command::new("jj")
        .arg("log")
        .arg("--no-graph")
        .arg("-r")
        .arg("::@") // Ancestors of HEAD
        .arg("-T")
        .arg(r#"author.email() ++ "|" ++ author.timestamp().format("%Y-%m-%d") ++ "\n""#)
        .current_dir(path)
        .output()?;

    if !output.status.success() {
        return Err(anyhow!("jj command failed"));
    }

    let stdout = String::from_utf8(output.stdout)?;

    let cutoff_date = (Utc::now() - Duration::days(DAYS_IN_LAST_SIX_MONTHS)).date_naive();

    for line in stdout.lines() {
        let parts: Vec<&str> = line.split('|').collect();

        if parts.len() < 2 {
            continue;
        }

        let commit_email = parts[0].trim();

        let date_str = parts[1].trim();

        if commit_email != email {
            continue;
        }

        match NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
            Ok(date) if date >= cutoff_date => {
                *commits.entry(date).or_insert(0) += 1;
            }

            _ => {}
        }
    }

    Ok(())
}
