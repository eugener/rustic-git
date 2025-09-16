//! File Lifecycle Operations Example
//!
//! This example demonstrates comprehensive file management capabilities:
//! - Restoring files from different sources (checkout_file, restore)
//! - Unstaging files (reset_file)
//! - Removing files with various options (rm, rm_with_options)
//! - Moving and renaming files (mv, mv_with_options)
//! - Managing .gitignore patterns (ignore_add, ignore_check, ignore_list)
//! - Handle various file scenarios and edge cases
//!
//! Run with: cargo run --example file_lifecycle_operations

use rustic_git::{MoveOptions, RemoveOptions, Repository, RestoreOptions, Result};
use std::{env, fs};

fn main() -> Result<()> {
    println!("Rustic Git - File Lifecycle Operations Example\n");

    let base_path = env::temp_dir().join("rustic_git_files_example");
    let repo_path = base_path.join("main_repo");

    // Clean up any previous runs
    if base_path.exists() {
        fs::remove_dir_all(&base_path).expect("Failed to clean up previous example");
    }
    fs::create_dir_all(&base_path)?;

    println!("=== Repository Setup ===\n");

    // Initialize repository
    println!("Initializing repository for file lifecycle demonstrations...");
    let repo = Repository::init(&repo_path, false)?;
    println!("Repository initialized at: {}", repo_path.display());

    // Set up git configuration for commits
    repo.config().set_user("Demo User", "demo@example.com")?;

    // Create initial project structure
    fs::create_dir_all(repo_path.join("src"))?;
    fs::create_dir_all(repo_path.join("docs"))?;
    fs::create_dir_all(repo_path.join("tests"))?;

    let files = [
        (
            "README.md",
            "# File Lifecycle Demo\n\nDemonstrating rustic-git file management capabilities.",
        ),
        (
            "src/main.rs",
            "fn main() {\n    println!(\"Hello, world!\");\n}",
        ),
        (
            "src/lib.rs",
            "//! Library module\n\npub fn greet() {\n    println!(\"Hello from lib!\");\n}",
        ),
        ("docs/guide.md", "# User Guide\n\nThis is the user guide."),
        (
            "tests/integration.rs",
            "#[test]\nfn test_basic() {\n    assert_eq!(2 + 2, 4);\n}",
        ),
    ];

    for (path, content) in &files {
        fs::write(repo_path.join(path), content)?;
    }

    repo.add(&files.iter().map(|(path, _)| *path).collect::<Vec<_>>())?;
    let initial_commit = repo.commit("Initial project setup")?;
    println!("Created initial commit: {}\n", initial_commit.short());

    println!("=== File Restoration Operations ===\n");

    // Modify some files
    println!("Modifying files to demonstrate restoration...");
    fs::write(
        repo_path.join("README.md"),
        "# Modified README\n\nThis content has been changed.",
    )?;
    fs::write(
        repo_path.join("src/main.rs"),
        "fn main() {\n    println!(\"Modified main!\");\n    println!(\"Added new line!\");\n}",
    )?;

    println!("   Modified README.md and src/main.rs");

    // Show current status
    let status = repo.status()?;
    println!(
        "   Files with modifications: {}",
        status.unstaged_files().count()
    );
    for entry in status.unstaged_files() {
        println!("     - {}", entry.path.display());
    }
    println!();

    // Restore single file with checkout_file
    println!("Restoring README.md using checkout_file():");
    repo.checkout_file("README.md")?;
    let restored_content = fs::read_to_string(repo_path.join("README.md"))?;
    println!("   ✓ README.md restored to original state");
    println!(
        "   Content preview: {:?}",
        restored_content.lines().next().unwrap_or("")
    );
    println!();

    // Demonstrate advanced restore with options
    println!("Creating second commit for restore demonstration...");
    fs::write(
        repo_path.join("src/advanced.rs"),
        "//! Advanced module\n\npub fn advanced_function() {\n    println!(\"Advanced functionality\");\n}",
    )?;
    repo.add(&["src/advanced.rs"])?;
    let second_commit = repo.commit("Add advanced module")?;
    println!("   Second commit: {}", second_commit.short());

    // Modify the advanced file
    fs::write(
        repo_path.join("src/advanced.rs"),
        "//! HEAVILY MODIFIED\n\npub fn broken_function() {\n    panic!(\"This is broken!\");\n}",
    )?;
    println!("   Modified src/advanced.rs");

    // Restore from specific commit using restore with options
    println!("Restoring src/advanced.rs from specific commit using restore():");
    let restore_options = RestoreOptions::new()
        .with_source(format!("{}", second_commit))
        .with_worktree();
    repo.restore(&["src/advanced.rs"], restore_options)?;

    let restored_advanced = fs::read_to_string(repo_path.join("src/advanced.rs"))?;
    println!("   ✓ File restored from commit {}", second_commit.short());
    println!(
        "   Content preview: {:?}",
        restored_advanced.lines().next().unwrap_or("")
    );
    println!();

    println!("=== Staging Area Operations ===\n");

    // Modify and stage files
    println!("Demonstrating staging area manipulation...");
    fs::write(
        repo_path.join("src/lib.rs"),
        "//! STAGED CHANGES\n\npub fn new_function() {\n    println!(\"This will be staged\");\n}",
    )?;
    repo.add(&["src/lib.rs"])?;
    println!("   Modified and staged src/lib.rs");

    let status = repo.status()?;
    println!("   Staged files: {}", status.staged_files().count());
    for entry in status.staged_files() {
        println!("     - {}", entry.path.display());
    }

    // Unstage the file
    println!("Unstaging src/lib.rs using reset_file():");
    repo.reset_file("src/lib.rs")?;

    let status_after_reset = repo.status()?;
    println!("   ✓ File unstaged (now in modified files)");
    println!(
        "   Staged files: {}",
        status_after_reset.staged_files().count()
    );
    println!(
        "   Modified files: {}",
        status_after_reset.unstaged_files().count()
    );
    println!();

    println!("=== File Removal Operations ===\n");

    // Create files for removal demonstration
    println!("Creating files for removal demonstration...");
    fs::write(repo_path.join("temp_file.txt"), "This is a temporary file")?;
    fs::write(
        repo_path.join("docs/old_doc.md"),
        "# Old Documentation\n\nThis document is outdated.",
    )?;
    fs::create_dir_all(repo_path.join("old_directory"))?;
    fs::write(
        repo_path.join("old_directory/nested_file.txt"),
        "Nested content",
    )?;

    // Add and commit these files
    repo.add(&[
        "temp_file.txt",
        "docs/old_doc.md",
        "old_directory/nested_file.txt",
    ])?;
    repo.commit("Add files for removal demo")?;
    println!("   Created and committed files for removal");

    // Basic file removal
    println!("Removing temp_file.txt using rm():");
    repo.rm(&["temp_file.txt"])?;
    println!("   ✓ temp_file.txt removed from repository and working tree");
    assert!(!repo_path.join("temp_file.txt").exists());

    // Remove from index only (keep in working tree)
    println!("Removing docs/old_doc.md from index only using rm_with_options():");
    let cached_remove_options = RemoveOptions::new().with_cached();
    repo.rm_with_options(&["docs/old_doc.md"], cached_remove_options)?;

    println!("   ✓ File removed from index but kept in working tree");
    assert!(repo_path.join("docs/old_doc.md").exists());
    let content = fs::read_to_string(repo_path.join("docs/old_doc.md"))?;
    println!(
        "   Working tree content still available: {:?}",
        content.lines().next().unwrap_or("")
    );

    // Recursive removal
    println!("Removing old_directory/ recursively:");
    let recursive_options = RemoveOptions::new().with_recursive();
    repo.rm_with_options(&["old_directory/"], recursive_options)?;
    println!("   ✓ Directory and contents removed recursively");
    assert!(!repo_path.join("old_directory").exists());
    println!();

    println!("=== File Move/Rename Operations ===\n");

    // Create files for move demonstration
    println!("Creating files for move/rename demonstration...");
    fs::write(repo_path.join("old_name.txt"), "This file will be renamed")?;
    fs::create_dir_all(repo_path.join("source_dir"))?;
    fs::write(
        repo_path.join("source_dir/movable.txt"),
        "This file will be moved",
    )?;
    fs::create_dir_all(repo_path.join("target_dir"))?;

    repo.add(&["old_name.txt", "source_dir/movable.txt"])?;
    repo.commit("Add files for move demo")?;
    println!("   Created files for move demonstration");

    // Simple rename
    println!("Renaming old_name.txt to new_name.txt using mv():");
    repo.mv("old_name.txt", "new_name.txt")?;

    assert!(!repo_path.join("old_name.txt").exists());
    assert!(repo_path.join("new_name.txt").exists());
    let content = fs::read_to_string(repo_path.join("new_name.txt"))?;
    println!("   ✓ File renamed successfully");
    println!("   Content preserved: {:?}", content.trim());

    // Move file to different directory
    println!("Moving source_dir/movable.txt to target_dir/ using mv():");
    repo.mv("source_dir/movable.txt", "target_dir/movable.txt")?;

    assert!(!repo_path.join("source_dir/movable.txt").exists());
    assert!(repo_path.join("target_dir/movable.txt").exists());
    println!("   ✓ File moved to different directory");

    // Demonstrate move with options (dry run)
    fs::write(repo_path.join("test_move.txt"), "Test content for dry run")?;
    repo.add(&["test_move.txt"])?;
    repo.commit("Add test file for dry run demo")?;

    println!("Demonstrating dry run move (won't actually move):");
    let dry_run_options = MoveOptions::new().with_dry_run().with_verbose();
    repo.mv_with_options("test_move.txt", "would_be_moved.txt", dry_run_options)?;

    // File should still exist at original location
    assert!(repo_path.join("test_move.txt").exists());
    assert!(!repo_path.join("would_be_moved.txt").exists());
    println!("   ✓ Dry run completed - no actual move performed");
    println!();

    println!("=== .gitignore Management ===\n");

    // Initially no ignore patterns
    println!("Checking initial .gitignore state:");
    let initial_patterns = repo.ignore_list()?;
    println!("   Initial ignore patterns: {}", initial_patterns.len());

    // Add ignore patterns
    println!("Adding ignore patterns...");
    repo.ignore_add(&[
        "*.tmp",
        "*.log",
        "build/",
        "node_modules/",
        ".DS_Store",
        "*.secret",
    ])?;
    println!("   Added 6 ignore patterns to .gitignore");

    // List current patterns
    let patterns = repo.ignore_list()?;
    println!("   Current ignore patterns: {}", patterns.len());
    for (i, pattern) in patterns.iter().enumerate() {
        println!("     {}. {}", i + 1, pattern);
    }

    // Create test files to check ignore status
    println!("\nCreating test files to check ignore status...");
    let test_files = [
        ("regular_file.txt", false),
        ("temp_file.tmp", true),
        ("debug.log", true),
        ("important.secret", true),
        ("normal.md", false),
    ];

    for (filename, _) in &test_files {
        fs::write(repo_path.join(filename), "test content")?;
    }

    // Check ignore status for each file
    println!("Checking ignore status for test files:");
    for (filename, expected_ignored) in &test_files {
        let is_ignored = repo.ignore_check(filename)?;
        let status_symbol = if is_ignored { "🚫" } else { "✅" };
        println!(
            "   {} {} - {}",
            status_symbol,
            filename,
            if is_ignored { "IGNORED" } else { "TRACKED" }
        );

        // Verify expectation
        assert_eq!(
            is_ignored, *expected_ignored,
            "Ignore status mismatch for {}",
            filename
        );
    }
    println!();

    println!("=== Error Handling and Edge Cases ===\n");

    // Test error cases
    println!("Testing error conditions:");

    // Try to checkout non-existent file
    println!("   Attempting to checkout non-existent file:");
    match repo.checkout_file("nonexistent.txt") {
        Ok(_) => println!("     Unexpected success"),
        Err(e) => println!("     ✓ Expected error: {}", e),
    }

    // Try to reset non-existent file
    println!("   Attempting to reset non-staged file:");
    match repo.reset_file("new_name.txt") {
        Ok(_) => println!("     ✓ Reset succeeded (file not staged, no error)"),
        Err(e) => println!("     Error: {}", e),
    }

    // Try to remove non-existent file
    println!("   Attempting to remove non-existent file:");
    match repo.rm(&["definitely_not_here.txt"]) {
        Ok(_) => println!("     Unexpected success"),
        Err(e) => println!("     ✓ Expected error: {}", e),
    }

    // Try to remove with ignore-unmatch option
    println!("   Attempting to remove with ignore-unmatch option:");
    let ignore_unmatch_options = RemoveOptions::new().with_ignore_unmatch();
    match repo.rm_with_options(&["also_not_here.txt"], ignore_unmatch_options) {
        Ok(_) => println!("     ✓ Succeeded with ignore-unmatch (no error)"),
        Err(e) => println!("     Error: {}", e),
    }

    // Try to move to existing file without force
    fs::write(repo_path.join("existing_target.txt"), "existing content")?;
    repo.add(&["existing_target.txt"])?;
    repo.commit("Add existing target")?;

    println!("   Attempting to move to existing file without force:");
    match repo.mv("test_move.txt", "existing_target.txt") {
        Ok(_) => println!("     Unexpected success (git may have overwritten)"),
        Err(e) => println!("     ✓ Expected error: {}", e),
    }
    println!();

    println!("=== Advanced Restore Operations ===\n");

    // Demonstrate restore with staged and worktree options
    println!("Demonstrating advanced restore with staging area...");

    // Modify file and stage it
    fs::write(repo_path.join("new_name.txt"), "staged changes")?;
    repo.add(&["new_name.txt"])?;

    // Modify it again in working tree
    fs::write(repo_path.join("new_name.txt"), "working tree changes")?;

    println!("   File has both staged and working tree changes");

    // Restore only staged area
    println!("   Restoring staged changes only:");
    let staged_restore = RestoreOptions::new().with_staged();
    repo.restore(&["new_name.txt"], staged_restore)?;

    let content_after_staged_restore = fs::read_to_string(repo_path.join("new_name.txt"))?;
    println!("     ✓ Staged changes restored, working tree preserved");
    println!(
        "     Working tree content: {:?}",
        content_after_staged_restore.trim()
    );

    // Restore working tree
    println!("   Restoring working tree:");
    let worktree_restore = RestoreOptions::new().with_worktree();
    repo.restore(&["new_name.txt"], worktree_restore)?;

    let final_content = fs::read_to_string(repo_path.join("new_name.txt"))?;
    println!("     ✓ Working tree restored to committed state");
    println!("     Final content: {:?}", final_content.trim());
    println!();

    println!("=== Repository State Summary ===\n");

    let final_status = repo.status()?;
    println!("Final repository state:");
    println!("   Clean repository: {}", final_status.is_clean());
    println!("   Staged files: {}", final_status.staged_files().count());
    println!(
        "   Modified files: {}",
        final_status.unstaged_files().count()
    );
    println!(
        "   Untracked files: {}",
        final_status.untracked_entries().count()
    );

    if !final_status.is_clean() {
        println!("\n   Remaining changes:");
        for entry in final_status.staged_files() {
            println!("     Staged: {}", entry.path.display());
        }
        for entry in final_status.unstaged_files() {
            println!("     Modified: {}", entry.path.display());
        }
        for entry in final_status.untracked_entries() {
            println!("     Untracked: {}", entry.path.display());
        }
    }

    // Show .gitignore content
    let final_patterns = repo.ignore_list()?;
    println!("\n   .gitignore patterns: {}", final_patterns.len());
    for pattern in final_patterns {
        println!("     - {}", pattern);
    }

    println!("\n=== Summary ===\n");

    println!("File lifecycle operations demonstration completed!");
    println!("  Repository: {}", repo_path.display());

    println!("\nOperations demonstrated:");
    println!("  ✓ File restoration from HEAD (checkout_file)");
    println!("  ✓ Advanced file restoration with options (restore)");
    println!("  ✓ Unstaging files (reset_file)");
    println!("  ✓ File removal with various options (rm, rm_with_options)");
    println!("  ✓ File moving and renaming (mv, mv_with_options)");
    println!("  ✓ .gitignore pattern management (ignore_add, ignore_list, ignore_check)");
    println!("  ✓ Staged vs working tree restoration");
    println!("  ✓ Error handling for invalid operations");
    println!("  ✓ Dry run and verbose options");
    println!("  ✓ Recursive and cached removal options");

    // Clean up
    println!("\nCleaning up example repositories...");
    fs::remove_dir_all(&base_path)?;
    println!("File lifecycle operations example completed!");

    Ok(())
}
