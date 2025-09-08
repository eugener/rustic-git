//! Status Checking Example
//!
//! This example demonstrates comprehensive status checking capabilities:
//! - Check clean vs dirty repository states
//! - Explore all FileStatus variants
//! - Use different status query methods
//! - Handle repositories with various file states
//!
//! Run with: cargo run --example status_checking

use rustic_git::{FileStatus, Repository, Result};
use std::fs;
use std::path::Path;

fn main() -> Result<()> {
    println!("Rustic Git - Status Checking Example\n");

    let repo_path = "/tmp/rustic_git_status_example";

    // Clean up any previous run
    if Path::new(repo_path).exists() {
        fs::remove_dir_all(repo_path).expect("Failed to clean up previous example");
    }

    // Initialize repository
    println!("Setting up repository for status demonstration...");
    let repo = Repository::init(repo_path, false)?;

    println!("=== Clean Repository Status ===\n");

    // Check initial status (should be clean)
    let status = repo.status()?;
    println!("Initial repository status:");
    display_status_summary(&status);
    println!();

    println!("=== Creating Files with Different States ===\n");

    // Create various types of files to demonstrate different statuses
    println!("Creating test files...");

    // Create some files that will be untracked
    fs::write(
        format!("{}/untracked1.txt", repo_path),
        "This file is untracked",
    )?;
    fs::write(
        format!("{}/untracked2.txt", repo_path),
        "Another untracked file",
    )?;

    // Create a .gitignore to demonstrate ignored files
    fs::write(
        format!("{}/.gitignore", repo_path),
        "*.log\n*.tmp\n/temp/\n",
    )?;

    // Create files that will be ignored
    fs::write(format!("{}/debug.log", repo_path), "Log file content")?;
    fs::write(format!("{}/cache.tmp", repo_path), "Temporary file")?;
    fs::create_dir_all(format!("{}/temp", repo_path))?;
    fs::write(format!("{}/temp/data.txt", repo_path), "Temp data")?;

    println!("Created test files");

    // Check status after creating untracked files
    println!("\nStatus after creating untracked files:");
    let status_untracked = repo.status()?;
    display_status_summary(&status_untracked);
    display_detailed_status(&status_untracked);
    println!();

    println!("=== Staging Files to Show 'Added' Status ===\n");

    // Stage some files to show "Added" status
    repo.add(&["untracked1.txt", ".gitignore"])?;
    println!("Staged untracked1.txt and .gitignore");

    let status_added = repo.status()?;
    println!("\nStatus after staging files:");
    display_status_summary(&status_added);
    display_detailed_status(&status_added);
    println!();

    println!("=== Creating Initial Commit ===\n");

    // Commit the staged files so we can demonstrate modified/deleted states
    let _hash = repo.commit("Initial commit with basic files")?;
    println!("Created initial commit");

    let status_after_commit = repo.status()?;
    println!("\nStatus after commit:");
    display_status_summary(&status_after_commit);
    if !status_after_commit.is_clean() {
        display_detailed_status(&status_after_commit);
    }
    println!();

    println!("=== Modifying Files to Show 'Modified' Status ===\n");

    // Modify existing tracked files
    fs::write(
        format!("{}/untracked1.txt", repo_path),
        "This file has been MODIFIED!",
    )?;
    fs::write(
        format!("{}/.gitignore", repo_path),
        "*.log\n*.tmp\n/temp/\n# Added comment\n",
    )?;
    println!("Modified untracked1.txt and .gitignore");

    let status_modified = repo.status()?;
    println!("\nStatus after modifying files:");
    display_status_summary(&status_modified);
    display_detailed_status(&status_modified);
    println!();

    println!("=== Demonstrating All Status Query Methods ===\n");

    // Stage one of the modified files to show mixed states
    repo.add(&["untracked1.txt"])?;
    println!("Staged untracked1.txt (now shows as Added)");

    let status_mixed = repo.status()?;
    println!("\nMixed status demonstration:");
    display_status_summary(&status_mixed);

    // Demonstrate different query methods
    println!("\nUsing different status query methods:");

    println!("   All files ({} total):", status_mixed.files.len());
    for (file_status, filename) in &status_mixed.files {
        println!("      {:?}: {}", file_status, filename);
    }

    // Query by specific status
    let modified_files = status_mixed.modified_files();
    if !modified_files.is_empty() {
        println!("\n   Modified files ({}):", modified_files.len());
        for filename in &modified_files {
            println!("      - {}", filename);
        }
    }

    let untracked_files = status_mixed.untracked_files();
    if !untracked_files.is_empty() {
        println!("\n   Untracked files ({}):", untracked_files.len());
        for filename in &untracked_files {
            println!("      - {}", filename);
        }
    }

    // Query by FileStatus enum
    let added_files = status_mixed.files_with_status(FileStatus::Added);
    if !added_files.is_empty() {
        println!("\n   Added files ({}):", added_files.len());
        for filename in &added_files {
            println!("      - {}", filename);
        }
    }

    println!();

    println!("=== File Status Filtering Examples ===\n");

    // Demonstrate filtering capabilities
    println!("Filtering examples:");

    // Count files by status
    let mut status_counts = std::collections::HashMap::new();
    for (file_status, _) in &status_mixed.files {
        *status_counts
            .entry(format!("{:?}", file_status))
            .or_insert(0) += 1;
    }

    println!("   Files by status:");
    for (status, count) in &status_counts {
        println!("      {}: {} files", status, count);
    }

    // Filter for specific patterns
    let txt_files: Vec<_> = status_mixed
        .files
        .iter()
        .filter(|(_, filename)| filename.ends_with(".txt"))
        .collect();

    if !txt_files.is_empty() {
        println!("\n   .txt files:");
        for (file_status, filename) in txt_files {
            println!("      {:?}: {}", file_status, filename);
        }
    }

    println!();

    println!("=== Repository State Checking ===\n");

    println!("Repository state summary:");
    println!("   Total files tracked: {}", status_mixed.files.len());
    println!("   Is clean: {}", status_mixed.is_clean());
    println!("   Has changes: {}", status_mixed.has_changes());

    if status_mixed.has_changes() {
        println!("   Repository needs attention!");

        if !status_mixed.modified_files().is_empty() {
            println!(
                "      - {} files need to be staged",
                status_mixed.modified_files().len()
            );
        }

        if !status_mixed.untracked_files().is_empty() {
            println!(
                "      - {} untracked files to consider",
                status_mixed.untracked_files().len()
            );
        }

        let staged_count = status_mixed
            .files
            .iter()
            .filter(|(status, _)| matches!(status, FileStatus::Added))
            .count();

        if staged_count > 0 {
            println!("      - {} files ready to commit", staged_count);
        }
    }

    // Clean up
    println!("\nCleaning up example repository...");
    fs::remove_dir_all(repo_path)?;
    println!("Status checking example completed!");

    Ok(())
}

/// Display a summary of the repository status
fn display_status_summary(status: &rustic_git::GitStatus) {
    if status.is_clean() {
        println!("   Repository is clean (no changes)");
    } else {
        println!("   Repository has {} changes", status.files.len());
        println!("      Modified: {}", status.modified_files().len());
        println!("      Untracked: {}", status.untracked_files().len());
    }
}

/// Display detailed status information
fn display_detailed_status(status: &rustic_git::GitStatus) {
    if !status.files.is_empty() {
        println!("   Detailed file status:");
        for (file_status, filename) in &status.files {
            let marker = match file_status {
                FileStatus::Modified => "[M]",
                FileStatus::Added => "[A]",
                FileStatus::Deleted => "[D]",
                FileStatus::Renamed => "[R]",
                FileStatus::Copied => "[C]",
                FileStatus::Untracked => "[?]",
                FileStatus::Ignored => "[I]",
            };
            println!("      {} {:?}: {}", marker, file_status, filename);
        }
    }
}
