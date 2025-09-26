use rustic_git::{Repository, ResetMode, Result};
use std::{env, fs};

/// Comprehensive demonstration of reset operations in rustic-git
///
/// This example showcases:
/// - Different reset modes (soft, mixed, hard)
/// - Reset to specific commits
/// - File-specific resets
/// - Error handling for invalid commits
///
/// Reset operations are essential for managing git history and staging area.
fn main() -> Result<()> {
    println!("=== Reset Operations Demo ===\n");

    // Create a temporary directory for our example
    let temp_dir = env::temp_dir().join("rustic_git_reset_demo");

    // Clean up if exists
    if temp_dir.exists() {
        fs::remove_dir_all(&temp_dir)?;
    }

    println!("Working in temporary directory: {:?}\n", temp_dir);

    // Initialize a new repository
    let repo = Repository::init(&temp_dir, false)?;

    // Configure user for commits
    repo.config().set_user("Example User", "example@test.com")?;

    demonstrate_reset_modes(&repo, &temp_dir)?;
    demonstrate_file_resets(&repo, &temp_dir)?;
    demonstrate_error_handling(&repo)?;

    println!("\n=== Reset Operations Demo Complete ===");

    // Clean up
    fs::remove_dir_all(&temp_dir)?;
    Ok(())
}

fn demonstrate_reset_modes(repo: &Repository, temp_dir: &std::path::Path) -> Result<()> {
    println!("--- Demonstrating Reset Modes ---\n");

    // Create initial commits
    println!("1. Creating initial commits...");

    // First commit
    let file1_path = temp_dir.join("file1.txt");
    fs::write(&file1_path, "Initial content")?;
    repo.add(&["file1.txt"])?;
    let first_commit = repo.commit("Initial commit")?;
    println!("   Created first commit: {}", first_commit);

    // Second commit
    let file2_path = temp_dir.join("file2.txt");
    fs::write(&file2_path, "Second file content")?;
    repo.add(&["file2.txt"])?;
    let second_commit = repo.commit("Add file2.txt")?;
    println!("   Created second commit: {}", second_commit);

    // Third commit
    fs::write(&file1_path, "Modified content")?;
    repo.add(&["file1.txt"])?;
    let third_commit = repo.commit("Modify file1.txt")?;
    println!("   Created third commit: {}", third_commit);

    // Show current status
    println!("\n2. Current repository state:");
    show_repo_state(repo)?;

    // Demonstrate soft reset
    println!("\n3. Performing soft reset to second commit...");
    repo.reset_soft(&second_commit.to_string())?;

    println!("   After soft reset:");
    show_repo_state(repo)?;
    println!("   Note: Changes are still staged, working directory unchanged");

    // Reset back to third commit for next demonstration
    repo.reset_hard(&third_commit.to_string())?;

    // Demonstrate mixed reset (default)
    println!("\n4. Performing mixed reset to second commit...");
    repo.reset_mixed(&second_commit.to_string())?;

    println!("   After mixed reset:");
    show_repo_state(repo)?;
    println!("   Note: Changes are unstaged but preserved in working directory");

    // Reset back to third commit for next demonstration
    repo.reset_hard(&third_commit.to_string())?;

    // Demonstrate hard reset
    println!("\n5. Performing hard reset to first commit...");
    repo.reset_hard(&first_commit.to_string())?;

    println!("   After hard reset:");
    show_repo_state(repo)?;
    println!("   Note: All changes discarded, working directory matches commit");

    // Demonstrate reset_with_mode for flexibility
    println!("\n6. Using reset_with_mode for explicit control...");

    // Recreate second commit for demo
    fs::write(&file2_path, "Recreated second file")?;
    repo.add(&["file2.txt"])?;
    let _new_commit = repo.commit("Recreate file2.txt")?;

    repo.reset_with_mode(&first_commit.to_string(), ResetMode::Mixed)?;
    println!("   Used ResetMode::Mixed explicitly");
    show_repo_state(repo)?;

    Ok(())
}

fn demonstrate_file_resets(repo: &Repository, temp_dir: &std::path::Path) -> Result<()> {
    println!("\n--- Demonstrating File-Specific Resets ---\n");

    // Create some files and stage them
    println!("1. Creating and staging multiple files...");

    let file_a = temp_dir.join("fileA.txt");
    let file_b = temp_dir.join("fileB.txt");

    fs::write(&file_a, "Content A")?;
    fs::write(&file_b, "Content B")?;

    repo.add(&["fileA.txt", "fileB.txt"])?;
    println!("   Staged fileA.txt and fileB.txt");

    show_repo_state(repo)?;

    // Reset a single file (using existing reset_file from files.rs)
    println!("\n2. Resetting single file (fileA.txt)...");
    repo.reset_file("fileA.txt")?;

    println!("   After resetting fileA.txt:");
    show_repo_state(repo)?;
    println!("   Note: fileA.txt is unstaged, fileB.txt remains staged");

    // Demonstrate HEAD reset (unstage all changes)
    println!("\n3. Performing mixed reset to HEAD (unstage all)...");
    repo.reset_mixed("HEAD")?;

    println!("   After reset HEAD:");
    show_repo_state(repo)?;
    println!("   Note: All staged changes are now unstaged");

    Ok(())
}

fn demonstrate_error_handling(repo: &Repository) -> Result<()> {
    println!("\n--- Demonstrating Error Handling ---\n");

    // Try to reset to invalid commit
    println!("1. Attempting reset to invalid commit hash...");
    match repo.reset_mixed("invalid_commit_hash") {
        Ok(_) => println!("   Unexpected success!"),
        Err(e) => println!("   Expected error: {}", e),
    }

    // Try to reset to non-existent reference
    println!("\n2. Attempting reset to non-existent reference...");
    match repo.reset_soft("nonexistent-branch") {
        Ok(_) => println!("   Unexpected success!"),
        Err(e) => println!("   Expected error: {}", e),
    }

    println!("\n   Error handling works correctly!");
    Ok(())
}

fn show_repo_state(repo: &Repository) -> Result<()> {
    let status = repo.status()?;

    let staged_count = status.staged_files().count();
    let unstaged_count = status.unstaged_files().count();
    let untracked_count = status.untracked_entries().count();

    println!("   Repository state:");
    println!("     - Staged files: {}", staged_count);
    println!("     - Modified files: {}", unstaged_count);
    println!("     - Untracked files: {}", untracked_count);

    if staged_count > 0 {
        println!("     - Staged:");
        for file in status.staged_files().take(5) {
            println!("       * {}", file.path.display());
        }
    }

    if unstaged_count > 0 {
        println!("     - Modified:");
        for file in status.unstaged_files().take(5) {
            println!("       * {}", file.path.display());
        }
    }

    if untracked_count > 0 {
        println!("     - Untracked:");
        for file in status.untracked_entries().take(5) {
            println!("       * {}", file.path.display());
        }
    }

    Ok(())
}
