//! Staging Operations Example
//!
//! This example demonstrates all available staging methods:
//! - add(): Stage specific files
//! - add_all(): Stage all changes (like `git add .`)
//! - add_update(): Stage all tracked file changes (like `git add -u`)
//! - Show before/after status for each operation
//!
//! Run with: cargo run --example staging_operations

use rustic_git::{IndexStatus, Repository, Result, WorktreeStatus};
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

    fs::write(
        format!("{}/README.md", repo_path),
        "# Staging Demo\nOriginal content",
    )?;
    fs::write(
        format!("{}/src/main.rs", repo_path),
        "fn main() { println!(\"v1\"); }",
    )?;
    fs::write(
        format!("{}/src/lib.rs", repo_path),
        "pub fn version() -> &'static str { \"1.0\" }",
    )?;

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
    fs::write(
        format!("{}/README.md", repo_path),
        "# Staging Demo\nUpdated content!",
    )?;
    fs::write(
        format!("{}/src/main.rs", repo_path),
        "fn main() { println!(\"v2 - updated!\"); }",
    )?;

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
    display_status_changes(
        &status_before,
        &status_after_readme,
        "after staging README.md",
    );

    // Stage multiple specific files
    repo.add(&["new_file1.txt", "src/main.rs"])?;
    println!("   Staged new_file1.txt and src/main.rs");

    let status_after_multiple = repo.status()?;
    display_status_changes(
        &status_after_readme,
        &status_after_multiple,
        "after staging multiple files",
    );

    // Stage using Path objects (alternative syntax)
    use std::path::Path as StdPath;
    repo.add(&[StdPath::new("docs/guide.md")])?;
    println!("   Staged docs/guide.md using Path object");

    let status_after_path = repo.status()?;
    display_status_changes(
        &status_after_multiple,
        &status_after_path,
        "after staging with Path object",
    );

    println!();

    println!("=== Staging All Changes with add_all() ===\n");

    // Create more files to demonstrate add_all()
    println!("Creating additional files for add_all() demo...");
    fs::write(
        format!("{}/config.toml", repo_path),
        "[package]\nname = \"example\"",
    )?;
    fs::write(format!("{}/src/utils.rs", repo_path), "pub fn helper() {}")?;
    fs::create_dir_all(format!("{}/tests", repo_path))?;
    fs::write(
        format!("{}/tests/integration.rs", repo_path),
        "#[test]\nfn test_basic() {}",
    )?;

    println!("Created 3 more files");

    let status_before_add_all = repo.status()?;
    println!("\nStatus before add_all():");
    display_status_breakdown(&status_before_add_all);

    // Use add_all() to stage everything remaining
    println!("\nUsing add_all() to stage all remaining changes:");
    repo.add_all()?;
    println!("   Staged all changes with add_all()");

    let status_after_add_all = repo.status()?;
    display_status_changes(
        &status_before_add_all,
        &status_after_add_all,
        "after add_all()",
    );

    // Create a commit to set up for add_update() demo
    let _commit_hash = repo.commit("Add all new files and modifications")?;
    println!("   Committed all staged changes\n");

    println!("=== Staging Tracked Changes with add_update() ===\n");

    // Create new untracked files and modify existing tracked files
    println!("Setting up files for add_update() demonstration...");

    // Create new untracked files (these should NOT be staged by add_update)
    fs::write(format!("{}/untracked1.txt", repo_path), "This is untracked")?;
    fs::write(
        format!("{}/untracked2.txt", repo_path),
        "Another untracked file",
    )?;

    // Modify existing tracked files (these SHOULD be staged by add_update)
    fs::write(
        format!("{}/README.md", repo_path),
        "# Staging Demo\nContent updated again for add_update demo!",
    )?;
    fs::write(
        format!("{}/src/lib.rs", repo_path),
        "pub fn version() -> &'static str { \"2.0\" }",
    )?;
    fs::write(
        format!("{}/config.toml", repo_path),
        "[package]\nname = \"example\"\nversion = \"0.2.0\"",
    )?;

    println!("Created 2 untracked files and modified 3 tracked files");

    let status_before_add_update = repo.status()?;
    println!("\nStatus before add_update():");
    display_status_breakdown(&status_before_add_update);

    // Use add_update() to stage only tracked file changes
    println!("\nUsing add_update() to stage only tracked file modifications:");
    repo.add_update()?;
    println!("   Used add_update() - should stage modified tracked files only");

    let status_after_add_update = repo.status()?;
    display_status_changes(
        &status_before_add_update,
        &status_after_add_update,
        "after add_update()",
    );

    // Verify that untracked files are still untracked
    let remaining_untracked: Vec<_> = status_after_add_update.untracked_entries().collect();
    if !remaining_untracked.is_empty() {
        println!("   Untracked files remain untracked (as expected):");
        for entry in remaining_untracked {
            println!("      - {}", entry.path.display());
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
        let staged_count = final_status.staged_files().count();
        let untracked_count = final_status.untracked_entries().count();

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

    let mut index_counts = std::collections::HashMap::new();
    let mut worktree_counts = std::collections::HashMap::new();

    for entry in &status.entries {
        if !matches!(entry.index_status, IndexStatus::Clean) {
            *index_counts.entry(&entry.index_status).or_insert(0) += 1;
        }
        if !matches!(entry.worktree_status, WorktreeStatus::Clean) {
            *worktree_counts.entry(&entry.worktree_status).or_insert(0) += 1;
        }
    }

    println!("   Index status:");
    for (index_status, count) in &index_counts {
        let marker = match index_status {
            IndexStatus::Modified => "[M]",
            IndexStatus::Added => "[A]",
            IndexStatus::Deleted => "[D]",
            IndexStatus::Renamed => "[R]",
            IndexStatus::Copied => "[C]",
            IndexStatus::Clean => "[ ]",
        };
        println!("      {} {:?}: {} files", marker, index_status, count);
    }

    println!("   Worktree status:");
    for (worktree_status, count) in &worktree_counts {
        let marker = match worktree_status {
            WorktreeStatus::Modified => "[M]",
            WorktreeStatus::Deleted => "[D]",
            WorktreeStatus::Untracked => "[?]",
            WorktreeStatus::Ignored => "[I]",
            WorktreeStatus::Clean => "[ ]",
        };
        println!("      {} {:?}: {} files", marker, worktree_status, count);
    }
}

/// Display changes between two status states
fn display_status_changes(
    before: &rustic_git::GitStatus,
    after: &rustic_git::GitStatus,
    description: &str,
) {
    println!("\n   Status changes {}:", description);

    let before_count = before.entries.len();
    let after_count = after.entries.len();

    if before_count == after_count {
        println!("      Total files unchanged ({} files)", after_count);
    } else {
        println!(
            "      Total files: {} → {} ({:+})",
            before_count,
            after_count,
            after_count as i32 - before_count as i32
        );
    }

    // Show status summary
    let before_staged = before.staged_files().count();
    let after_staged = after.staged_files().count();
    let before_untracked = before.untracked_entries().count();
    let after_untracked = after.untracked_entries().count();

    if before_staged != after_staged {
        println!(
            "      Staged files: {} → {} ({:+})",
            before_staged,
            after_staged,
            after_staged as i32 - before_staged as i32
        );
    }

    if before_untracked != after_untracked {
        println!(
            "      Untracked files: {} → {} ({:+})",
            before_untracked,
            after_untracked,
            after_untracked as i32 - before_untracked as i32
        );
    }
}
