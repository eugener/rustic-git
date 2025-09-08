//! Staging Operations Example
//!
//! This example demonstrates all available staging methods:
//! - add(): Stage specific files
//! - add_all(): Stage all changes (like `git add .`)
//! - add_update(): Stage all tracked file changes (like `git add -u`)
//! - Show before/after status for each operation
//!
//! Run with: cargo run --example staging_operations

use rustic_git::{Repository, FileStatus, Result};
use std::fs;
use std::path::Path;

fn main() -> Result<()> {
    println!("Rustic Git - Staging Operations Example\n");

    let repo_path = "/tmp/rustic_git_staging_example";
    
    // Clean up any previous run
    if Path::new(repo_path).exists() {
        fs::remove_dir_all(repo_path).expect("Failed to clean up previous example");
    }

    // Initialize repository and create initial commit
    println!("Setting up repository with initial files...");
    let repo = Repository::init(repo_path, false)?;
    
    // Create initial files
    fs::create_dir_all(format!("{}/src", repo_path))?;
    fs::create_dir_all(format!("{}/docs", repo_path))?;
    
    fs::write(format!("{}/README.md", repo_path), "# Staging Demo\nOriginal content")?;
    fs::write(format!("{}/src/main.rs", repo_path), "fn main() { println!(\"v1\"); }")?;
    fs::write(format!("{}/src/lib.rs", repo_path), "pub fn version() -> &'static str { \"1.0\" }")?;
    
    // Create initial commit so we can demonstrate staging tracked file changes
    repo.add_all()?;
    let _initial_hash = repo.commit("Initial commit with basic files")?;
    println!("Created initial repository with 3 files\n");

    println!("=== Staging Specific Files with add() ===\n");

    // Create some new files and modify existing ones
    println!("Creating new files and modifying existing ones...");
    fs::write(format!("{}/new_file1.txt", repo_path), "New file 1 content")?;
    fs::write(format!("{}/new_file2.txt", repo_path), "New file 2 content")?;
    fs::write(format!("{}/docs/guide.md", repo_path), "# User Guide")?;
    
    // Modify existing files
    fs::write(format!("{}/README.md", repo_path), "# Staging Demo\nUpdated content!")?;
    fs::write(format!("{}/src/main.rs", repo_path), "fn main() { println!(\"v2 - updated!\"); }")?;
    
    println!("Created 3 new files and modified 2 existing files");

    // Show status before staging
    println!("\nStatus before staging:");
    let status_before = repo.status()?;
    display_status_breakdown(&status_before);

    // Stage specific files using add()
    println!("\nUsing add() to stage specific files:");
    
    // Stage just the README.md
    repo.add(&["README.md"])?;
    println!("   Staged README.md");
    
    let status_after_readme = repo.status()?;
    display_status_changes(&status_before, &status_after_readme, "after staging README.md");

    // Stage multiple specific files
    repo.add(&["new_file1.txt", "src/main.rs"])?;
    println!("   Staged new_file1.txt and src/main.rs");
    
    let status_after_multiple = repo.status()?;
    display_status_changes(&status_after_readme, &status_after_multiple, "after staging multiple files");

    // Stage using Path objects (alternative syntax)
    use std::path::Path as StdPath;
    repo.add(&[StdPath::new("docs/guide.md")])?;
    println!("   Staged docs/guide.md using Path object");

    let status_after_path = repo.status()?;
    display_status_changes(&status_after_multiple, &status_after_path, "after staging with Path object");

    println!();

    println!("=== Staging All Changes with add_all() ===\n");

    // Create more files to demonstrate add_all()
    println!("Creating additional files for add_all() demo...");
    fs::write(format!("{}/config.toml", repo_path), "[package]\nname = \"example\"")?;
    fs::write(format!("{}/src/utils.rs", repo_path), "pub fn helper() {}")?;
    fs::create_dir_all(format!("{}/tests", repo_path))?;
    fs::write(format!("{}/tests/integration.rs", repo_path), "#[test]\nfn test_basic() {}")?;
    
    println!("Created 3 more files");

    let status_before_add_all = repo.status()?;
    println!("\nStatus before add_all():");
    display_status_breakdown(&status_before_add_all);

    // Use add_all() to stage everything remaining
    println!("\nUsing add_all() to stage all remaining changes:");
    repo.add_all()?;
    println!("   Staged all changes with add_all()");

    let status_after_add_all = repo.status()?;
    display_status_changes(&status_before_add_all, &status_after_add_all, "after add_all()");

    // Create a commit to set up for add_update() demo
    let _commit_hash = repo.commit("Add all new files and modifications")?;
    println!("   Committed all staged changes\n");

    println!("=== Staging Tracked Changes with add_update() ===\n");

    // Create new untracked files and modify existing tracked files
    println!("Setting up files for add_update() demonstration...");
    
    // Create new untracked files (these should NOT be staged by add_update)
    fs::write(format!("{}/untracked1.txt", repo_path), "This is untracked")?;
    fs::write(format!("{}/untracked2.txt", repo_path), "Another untracked file")?;
    
    // Modify existing tracked files (these SHOULD be staged by add_update)
    fs::write(format!("{}/README.md", repo_path), "# Staging Demo\nContent updated again for add_update demo!")?;
    fs::write(format!("{}/src/lib.rs", repo_path), "pub fn version() -> &'static str { \"2.0\" }")?;
    fs::write(format!("{}/config.toml", repo_path), "[package]\nname = \"example\"\nversion = \"0.2.0\"")?;
    
    println!("Created 2 untracked files and modified 3 tracked files");

    let status_before_add_update = repo.status()?;
    println!("\nStatus before add_update():");
    display_status_breakdown(&status_before_add_update);

    // Use add_update() to stage only tracked file changes
    println!("\nUsing add_update() to stage only tracked file modifications:");
    repo.add_update()?;
    println!("   Used add_update() - should stage modified tracked files only");

    let status_after_add_update = repo.status()?;
    display_status_changes(&status_before_add_update, &status_after_add_update, "after add_update()");
    
    // Verify that untracked files are still untracked
    let remaining_untracked = status_after_add_update.untracked_files();
    if !remaining_untracked.is_empty() {
        println!("   Untracked files remain untracked (as expected):");
        for filename in remaining_untracked {
            println!("      - {}", filename);
        }
    }

    println!();

    println!("=== Error Handling in Staging Operations ===\n");

    // Demonstrate error handling
    println!("Testing error conditions:");

    // Try to add non-existent files
    match repo.add(&["nonexistent_file.txt"]) {
        Ok(_) => println!("   Unexpectedly succeeded adding non-existent file"),
        Err(e) => println!("   Expected error for non-existent file: {:?}", e),
    }

    // Try to add empty array (should succeed but do nothing)
    match repo.add(&[] as &[&str]) {
        Ok(_) => println!("   Empty add() succeeded (no-op)"),
        Err(e) => println!("   Empty add() failed: {:?}", e),
    }

    println!();

    println!("=== Final Repository State ===\n");

    let final_status = repo.status()?;
    println!("Final repository summary:");
    display_status_breakdown(&final_status);
    
    if final_status.has_changes() {
        let staged_count = final_status.files.iter()
            .filter(|(status, _)| matches!(status, FileStatus::Added | FileStatus::Modified))
            .count();
        let untracked_count = final_status.untracked_files().len();
        
        println!("\nRepository state:");
        println!("   {} files staged and ready to commit", staged_count);
        println!("   {} untracked files not yet added", untracked_count);
        
        if staged_count > 0 {
            println!("\n   You could now commit with: repo.commit(\"Your message\")?");
        }
    }

    // Clean up
    println!("\nCleaning up example repository...");
    fs::remove_dir_all(repo_path)?;
    println!("Staging operations example completed!");

    Ok(())
}

/// Display a breakdown of repository status
fn display_status_breakdown(status: &rustic_git::GitStatus) {
    if status.is_clean() {
        println!("   Repository is clean");
        return;
    }

    let mut counts = std::collections::HashMap::new();
    for (file_status, _) in &status.files {
        *counts.entry(file_status).or_insert(0) += 1;
    }

    println!("   Files by status:");
    for (file_status, count) in &counts {
        let marker = match file_status {
            FileStatus::Modified => "[M]",
            FileStatus::Added => "[A]",
            FileStatus::Deleted => "[D]", 
            FileStatus::Renamed => "[R]",
            FileStatus::Copied => "[C]",
            FileStatus::Untracked => "[?]",
            FileStatus::Ignored => "[I]",
        };
        println!("      {} {:?}: {} files", marker, file_status, count);
    }
}

/// Display changes between two status states
fn display_status_changes(before: &rustic_git::GitStatus, after: &rustic_git::GitStatus, description: &str) {
    println!("\n   Status changes {}:", description);
    
    let before_count = before.files.len();
    let after_count = after.files.len();
    
    if before_count == after_count {
        println!("      Total files unchanged ({} files)", after_count);
    } else {
        println!("      Total files: {} → {} ({:+})", before_count, after_count, after_count as i32 - before_count as i32);
    }
    
    // Count status types in both states
    let mut before_counts = std::collections::HashMap::new();
    let mut after_counts = std::collections::HashMap::new();
    
    for (status, _) in &before.files {
        *before_counts.entry(format!("{:?}", status)).or_insert(0) += 1;
    }
    
    for (status, _) in &after.files {
        *after_counts.entry(format!("{:?}", status)).or_insert(0) += 1;
    }
    
    // Show changes for each status type
    let all_statuses: std::collections::HashSet<_> = before_counts.keys().chain(after_counts.keys()).collect();
    
    for status in all_statuses {
        let before_val = before_counts.get(status).unwrap_or(&0);
        let after_val = after_counts.get(status).unwrap_or(&0);
        
        if before_val != after_val {
            println!("      {}: {} → {} ({:+})", status, before_val, after_val, *after_val as i32 - *before_val as i32);
        }
    }
}