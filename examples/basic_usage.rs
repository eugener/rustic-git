//! Basic Usage Example
//!
//! This example demonstrates the fundamental workflow of the rustic-git library:
//! - Initialize a new repository
//! - Create some files
//! - Check repository status
//! - Stage files
//! - Create a commit
//!
//! Run with: cargo run --example basic_usage

use rustic_git::{Repository, Result};
use std::fs;
use std::path::Path;

fn main() -> Result<()> {
    println!("Rustic Git - Basic Usage Example\n");

    // Use a temporary directory for this example
    let repo_path = "/tmp/rustic_git_basic_example";

    // Clean up any previous run
    if Path::new(repo_path).exists() {
        fs::remove_dir_all(repo_path).expect("Failed to clean up previous example");
    }

    println!("Initializing new repository at: {}", repo_path);

    // Initialize a new repository
    let repo = Repository::init(repo_path, false)?;
    println!("Repository initialized successfully\n");

    // Create some example files
    println!("Creating example files...");
    fs::create_dir_all(format!("{}/src", repo_path))?;

    fs::write(
        format!("{}/README.md", repo_path),
        "# My Awesome Project\n\nThis is a demo project for rustic-git!\n",
    )?;

    fs::write(
        format!("{}/src/main.rs", repo_path),
        r#"fn main() {
    println!("Hello from rustic-git example!");
}
"#,
    )?;

    fs::write(
        format!("{}/src/lib.rs", repo_path),
        "// Library code goes here\npub fn hello() -> &'static str {\n    \"Hello, World!\"\n}\n",
    )?;

    println!("Created 3 files: README.md, src/main.rs, src/lib.rs\n");

    // Check repository status
    println!("Checking repository status...");
    let status = repo.status()?;

    if status.is_clean() {
        println!("   Repository is clean (no changes)");
    } else {
        println!("   Repository has changes:");
        println!("   Modified files: {}", status.modified_files().len());
        println!("   Untracked files: {}", status.untracked_files().len());

        // Show untracked files
        for filename in status.untracked_files() {
            println!("      - {}", filename);
        }
    }
    println!();

    // Stage specific files first
    println!("Staging files...");

    // Stage README.md first
    repo.add(&["README.md"])?;
    println!("Staged README.md");

    // Stage all remaining files
    repo.add_all()?;
    println!("Staged all remaining files");

    // Check status after staging
    let status_after_staging = repo.status()?;
    println!("\nStatus after staging:");
    if status_after_staging.is_clean() {
        println!("   Repository is clean (all changes staged)");
    } else {
        println!(
            "   Files staged for commit: {}",
            status_after_staging.files.len()
        );
        for (file_status, filename) in &status_after_staging.files {
            println!("      {:?}: {}", file_status, filename);
        }
    }
    println!();

    // Create a commit
    println!("Creating commit...");
    let hash = repo.commit("Initial commit: Add project structure and basic files")?;

    println!("Commit created successfully!");
    println!("   Full hash: {}", hash);
    println!("   Short hash: {}", hash.short());
    println!();

    // Verify final status
    println!("Final repository status:");
    let final_status = repo.status()?;
    if final_status.is_clean() {
        println!("   Repository is clean - all changes committed!");
    } else {
        println!("   Repository still has uncommitted changes");
    }
    println!();

    // Clean up
    println!("Cleaning up example repository...");
    fs::remove_dir_all(repo_path)?;
    println!("Example completed successfully!");

    Ok(())
}
