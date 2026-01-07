use assert_cmd::Command;
use predicates::prelude::*;
use std::process::Command as StdCommand;
use tempfile::tempdir;

#[test]
fn test_cli_integration() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let repo_path = dir.path().join("repo");
    std::fs::create_dir(&repo_path)?;

    // Setup a real git repo
    let email = "integration@test.com";
    StdCommand::new("git")
        .arg("init")
        .current_dir(&repo_path)
        .output()?;
    StdCommand::new("git")
        .args(&["config", "user.email", email])
        .current_dir(&repo_path)
        .output()?;
        StdCommand::new("git")
            .args(&["config", "user.name", "Test User"])
            .current_dir(&repo_path)
            .output()?;
            
        StdCommand::new("git")
            .args(&["config", "commit.gpgsign", "false"])
            .current_dir(&repo_path)
            .output()?;
        
        std::fs::write(repo_path.join("README.md"), "# Test")?;    StdCommand::new("git")
        .args(&["add", "."])
        .current_dir(&repo_path)
        .output()?;
    StdCommand::new("git")
        .args(&["commit", "-m", "Initial commit"])
        .current_dir(&repo_path)
        .output()?;

    let mut cmd = cargo::cargo_bin_cmd!("gitlocalstats");

    cmd.arg("--folder")
        .arg(dir.path())
        .arg("--email")
        .arg(email);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Scanning"));

    // We can also verify that the output contains the visual representation of the commit
    // Since we just made a commit today, the output should not be empty of "1"s or colored blocks?
    // The ANSI codes make it hard to grep for "1", but we can check if it finishes successfully.

    Ok(())
}
