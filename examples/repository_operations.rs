//! Repository Operations Example
//!
//! This example demonstrates various repository lifecycle operations:
//! - Initialize regular and bare repositories
//! - Open existing repositories
//! - Handle errors when opening non-existent repositories
//! - Display repository information
//!
//! Run with: cargo run --example repository_operations

use rustic_git::{GitError, Repository, Result};
use std::fs;
use std::path::Path;

fn main() -> Result<()> {
    println!("Rustic Git - Repository Operations Example\n");

    let base_path = "/tmp/rustic_git_repo_example";
    let regular_repo_path = format!("{}/regular", base_path);
    let bare_repo_path = format!("{}/bare", base_path);
    let nonexistent_path = format!("{}/nonexistent", base_path);

    // Clean up any previous runs
    if Path::new(base_path).exists() {
        fs::remove_dir_all(base_path).expect("Failed to clean up previous example");
    }
    fs::create_dir_all(base_path)?;

    println!("=== Repository Initialization ===\n");

    // 1. Initialize a regular repository
    println!("Initializing regular repository...");
    let regular_repo = Repository::init(&regular_repo_path, false)?;
    println!("Regular repository created at: {}", regular_repo_path);
    println!("   Repository path: {:?}", regular_repo.repo_path());

    // Verify it's a git repo by checking for .git directory
    if Path::new(&format!("{}/.git", regular_repo_path)).exists() {
        println!("   .git directory found");
    }
    println!();

    // 2. Initialize a bare repository
    println!("Initializing bare repository...");
    let bare_repo = Repository::init(&bare_repo_path, true)?;
    println!("Bare repository created at: {}", bare_repo_path);
    println!("   Repository path: {:?}", bare_repo.repo_path());

    // Verify bare repo structure (has HEAD, objects, etc. directly)
    if Path::new(&format!("{}/HEAD", bare_repo_path)).exists() {
        println!("   HEAD file found (bare repository structure)");
    }
    if Path::new(&format!("{}/objects", bare_repo_path)).exists() {
        println!("   objects directory found");
    }
    println!();

    println!("=== Repository Opening ===\n");

    // 3. Open the existing regular repository
    println!("Opening existing regular repository...");
    match Repository::open(&regular_repo_path) {
        Ok(opened_repo) => {
            println!("Successfully opened regular repository");
            println!("   Repository path: {:?}", opened_repo.repo_path());

            // Test that we can perform operations on the opened repo
            let status = opened_repo.status()?;
            println!("   Repository status: {} files", status.files.len());
        }
        Err(e) => {
            println!("Failed to open regular repository: {:?}", e);
        }
    }
    println!();

    // 4. Open the existing bare repository
    println!("Opening existing bare repository...");
    match Repository::open(&bare_repo_path) {
        Ok(opened_bare) => {
            println!("Successfully opened bare repository");
            println!("   Repository path: {:?}", opened_bare.repo_path());

            // Note: status operations might behave differently on bare repos
            match opened_bare.status() {
                Ok(status) => println!("   Bare repository status: {} files", status.files.len()),
                Err(e) => println!(
                    "   Note: Status check on bare repo failed (expected): {:?}",
                    e
                ),
            }
        }
        Err(e) => {
            println!("Failed to open bare repository: {:?}", e);
        }
    }
    println!();

    println!("=== Error Handling ===\n");

    // 5. Try to open a non-existent repository
    println!("Attempting to open non-existent repository...");
    match Repository::open(&nonexistent_path) {
        Ok(_repo) => {
            println!("Unexpectedly succeeded opening non-existent repo");
        }
        Err(GitError::CommandFailed(msg)) => {
            println!("Expected error caught: CommandFailed");
            println!("   Error message: {}", msg);
        }
        Err(GitError::IoError(msg)) => {
            println!("Expected error caught: IoError");
            println!("   Error message: {}", msg);
        }
    }
    println!();

    // 6. Try to open a regular file as a repository
    let fake_repo_path = format!("{}/fake.txt", base_path);
    fs::write(&fake_repo_path, "This is not a git repository")?;

    println!("Attempting to open regular file as repository...");
    match Repository::open(&fake_repo_path) {
        Ok(_repo) => {
            println!("Unexpectedly succeeded opening regular file as repo");
        }
        Err(GitError::CommandFailed(msg)) => {
            println!("Expected error caught: CommandFailed");
            println!("   Error message: {}", msg);
        }
        Err(GitError::IoError(msg)) => {
            println!("Expected error caught: IoError");
            println!("   Error message: {}", msg);
        }
    }
    println!();

    println!("=== Repository Information ===\n");

    // 7. Compare regular vs bare repository information
    println!("Comparing repository types:");

    let regular_path = regular_repo.repo_path();
    let bare_path = bare_repo.repo_path();

    println!("   Regular repo path: {:?}", regular_path);
    println!("   Bare repo path: {:?}", bare_path);

    // Show directory contents
    if let Ok(entries) = fs::read_dir(regular_path) {
        let mut files: Vec<_> = entries.filter_map(|e| e.ok()).collect();
        files.sort_by_key(|e| e.file_name());

        println!("   Regular repo contents:");
        for entry in files {
            if let Some(name) = entry.file_name().to_str() {
                let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
                let marker = if is_dir { "[DIR]" } else { "[FILE]" };
                println!("     {} {}", marker, name);
            }
        }
    }

    if let Ok(entries) = fs::read_dir(bare_path) {
        let mut files: Vec<_> = entries.filter_map(|e| e.ok()).collect();
        files.sort_by_key(|e| e.file_name());

        println!("   Bare repo contents:");
        for entry in files {
            if let Some(name) = entry.file_name().to_str() {
                let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
                let marker = if is_dir { "[DIR]" } else { "[FILE]" };
                println!("     {} {}", marker, name);
            }
        }
    }
    println!();

    // Clean up
    println!("Cleaning up example repositories...");
    fs::remove_dir_all(base_path)?;
    println!("Repository operations example completed!");

    Ok(())
}
