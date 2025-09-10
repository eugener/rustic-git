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
use std::env;
use std::fs;

fn main() -> Result<()> {
    println!("Rustic Git - Basic Usage Example\n");

    // Use a temporary directory for this example
    let repo_path = env::temp_dir().join("rustic_git_basic_example");

    // Clean up any previous run
    if repo_path.exists() {
        fs::remove_dir_all(&repo_path).expect("Failed to clean up previous example");
    }

    println!("Initializing new repository at: {}", repo_path.display());

    // Initialize a new repository
    let repo = Repository::init(&repo_path, false)?;
    println!("Repository initialized successfully\n");

    // Create some example files
    println!("Creating example files...");
    fs::create_dir_all(repo_path.join("src"))?;

    fs::write(
        repo_path.join("README.md"),
        "# My Awesome Project\n\nThis is a demo project for rustic-git!\n",
    )?;

    fs::write(
        repo_path.join("src/main.rs"),
        r#"fn main() {
    println!("Hello from rustic-git example!");
}
"#,
    )?;

    fs::write(
        repo_path.join("src/lib.rs"),
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
        println!("   Unstaged files: {}", status.unstaged_files().count());
        println!("   Untracked files: {}", status.untracked_entries().count());

        // Show untracked files
        for entry in status.untracked_entries() {
            println!("      - {}", entry.path.display());
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
            status_after_staging.entries.len()
        );
        for entry in &status_after_staging.entries {
            println!(
                "      Index {:?}, Worktree {:?}: {}",
                entry.index_status,
                entry.worktree_status,
                entry.path.display()
            );
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
    fs::remove_dir_all(&repo_path)?;
    println!("Example completed successfully!");

    Ok(())
}
