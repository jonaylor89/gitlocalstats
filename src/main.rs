use clap::Parser;
use colored::Colorize;
use directories::UserDirs;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

mod scanner;
mod stats;
mod ui;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Folder to scan
    #[arg(short, long)]
    folder: Option<PathBuf>,

    /// Email to filter by
    #[arg(short, long)]
    email: Option<String>,

    /// Force a rescan of the filesystem (ignoring cache)
    #[arg(short, long)]
    rescan: bool,

    /// Enable verbose logging of timing performance
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> anyhow::Result<()> {
    let start_time = Instant::now();
    let cli = Cli::parse();

    // Config setup
    let user_dirs =
        UserDirs::new().ok_or_else(|| anyhow::anyhow!("Could not find user directories"))?;
    let home_dir = user_dirs.home_dir();
    let config_dir = home_dir.join(".config").join("gitlocalstats");
    let config_path = config_dir.join("config");
    
    // Cache setup
    let cache_dir = home_dir.join(".cache").join("gitlocalstats");
    let cache_path = cache_dir.join("repos.json");

    if !config_path.exists() {
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let default_scan_dir = home_dir.join("Repos");
        let default_config = format!(
            "folder={}\nemail=example@email.com\n",
            default_scan_dir.display()
        );
        fs::write(&config_path, default_config)?;
    }

    // Load config using dotenvy to parse the key=value format
    // We suppress errors if the file is malformed, but we tried creating it.
    let _ = dotenvy::from_path(&config_path);

    // Resolve arguments (CLI > Env/Config > Default)
    let folder_path = cli
        .folder
        .or_else(|| env::var("folder").ok().map(PathBuf::from))
        .unwrap_or_else(|| home_dir.join("Repos"));

    let email = cli
        .email
        .or_else(|| env::var("email").ok())
        .or_else(get_git_config_email)
        .unwrap_or_else(|| "example@email.com".to_string());

    println!(
        "Scanning {} for commits by {}...",
        folder_path.display().to_string().cyan(),
        email.cyan()
    );

    // Step 1: Scan
    let step_start = Instant::now();
    let cache_arg = if cli.rescan { None } else { Some(&cache_path) };
    let repos = scanner::scan(folder_path.to_str().unwrap(), cache_arg);
    if cli.verbose {
        println!("[Perf] Scan/Cache Load: {:.2?}", step_start.elapsed());
        println!("[Info] Processing {} repositories", repos.len());
    }

    // Step 2: Stats
    let step_start = Instant::now();
    let commit_counts = stats::process_repositories(repos, &email);
    if cli.verbose {
        println!("[Perf] Stats Processing: {:.2?}", step_start.elapsed());
    }

    // Step 3: UI
    let step_start = Instant::now();
    ui::print_stats(&commit_counts);
    if cli.verbose {
        println!("[Perf] UI Rendering: {:.2?}", step_start.elapsed());
    }

    let duration = start_time.elapsed();
    println!("\nDone in {:.2?}", duration);

    Ok(())
}

fn get_git_config_email() -> Option<String> {
    gix::config::File::from_globals()
        .ok()
        .and_then(|config| config.string("user.email").map(|s| s.to_string()))
}
