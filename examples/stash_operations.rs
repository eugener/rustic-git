//! Stash Operations Example
//!
//! This example demonstrates comprehensive stash management functionality in rustic-git:
//! - Creating and saving stashes with various options
//! - Listing and filtering stashes
//! - Applying and popping stashes
//! - Showing stash contents
//! - Dropping and clearing stashes
//! - Working with stash metadata and options
//!
//! Run with: cargo run --example stash_operations

use rustic_git::{Repository, Result, StashApplyOptions, StashOptions};
use std::env;
use std::fs;

fn main() -> Result<()> {
    println!("Rustic Git - Stash Operations Example\n");

    // Use a temporary directory for this example
    let repo_path = env::temp_dir().join("rustic_git_stash_example");

    // Clean up any previous run
    if repo_path.exists() {
        fs::remove_dir_all(&repo_path).expect("Failed to clean up previous example");
    }

    println!("Initializing repository at: {}", repo_path.display());

    // Initialize repository and configure user
    let repo = Repository::init(&repo_path, false)?;
    repo.config()
        .set_user("Stash Demo User", "stash@example.com")?;

    // Create initial commit to have a base
    println!("\nCreating initial commit...");
    fs::write(
        repo_path.join("README.md"),
        "# Stash Demo Project\n\nDemonstrating Git stash operations.\n",
    )?;
    repo.add(&["README.md"])?;
    let initial_commit = repo.commit("Initial commit: Add README")?;
    println!("Created initial commit: {}", initial_commit.short());

    // Create some work to stash
    println!("\n=== Creating Work to Stash ===");

    // Create tracked file modifications
    fs::write(
        repo_path.join("README.md"),
        "# Stash Demo Project\n\nDemonstrating Git stash operations.\n\nAdded some new content!\n",
    )?;

    // Create new tracked files
    fs::create_dir_all(repo_path.join("src"))?;
    fs::write(
        repo_path.join("src/main.rs"),
        "fn main() {\n    println!(\"Hello, stash!\");\n}\n",
    )?;

    // Create untracked files
    fs::write(
        repo_path.join("untracked.txt"),
        "This file is not tracked by git\n",
    )?;
    fs::write(repo_path.join("temp.log"), "Temporary log file\n")?;

    // Stage some changes
    repo.add(&["src/main.rs"])?;

    println!("Created various types of changes:");
    println!("  - Modified tracked file (README.md)");
    println!("  - Added new file and staged it (src/main.rs)");
    println!("  - Created untracked files (untracked.txt, temp.log)");

    // Check repository status before stashing
    let status = repo.status()?;
    println!("\nRepository status before stashing:");
    println!("  Staged files: {}", status.staged_files().count());
    println!("  Unstaged files: {}", status.unstaged_files().count());
    println!("  Untracked files: {}", status.untracked_entries().count());

    // Demonstrate stash creation
    println!("\n=== Creating Stashes ===");

    // 1. Simple stash save
    println!("\n1. Creating simple stash:");
    let simple_stash = repo.stash_save("WIP: working on main function")?;
    println!("Created stash: {}", simple_stash);
    println!("  Index: {}", simple_stash.index);
    println!("  Branch: {}", simple_stash.branch);
    println!("  Hash: {}", simple_stash.hash.short());

    // Check status after stash
    let status_after_stash = repo.status()?;
    println!("\nStatus after simple stash:");
    println!(
        "  Staged files: {}",
        status_after_stash.staged_files().count()
    );
    println!(
        "  Unstaged files: {}",
        status_after_stash.unstaged_files().count()
    );
    println!(
        "  Untracked files: {}",
        status_after_stash.untracked_entries().count()
    );

    // 2. Make more changes and create stash with untracked files
    println!("\n2. Creating stash with untracked files:");

    // Modify file again
    fs::write(
        repo_path.join("README.md"),
        "# Stash Demo Project\n\nDemonstrating Git stash operations.\n\nSecond round of changes!\n",
    )?;

    // Create more untracked files
    fs::write(repo_path.join("config.json"), "{\"debug\": true}\n")?;

    let untracked_options = StashOptions::new().with_untracked().with_keep_index();
    let untracked_stash = repo.stash_push(
        "WIP: config changes with untracked files",
        untracked_options,
    )?;
    println!("Created stash with untracked files: {}", untracked_stash);

    // 3. Create stash with specific paths
    println!("\n3. Creating stash with specific paths:");

    // Make changes to multiple files and add them to git
    fs::write(repo_path.join("file1.txt"), "Content for file 1\n")?;
    fs::write(repo_path.join("file2.txt"), "Content for file 2\n")?;
    fs::write(repo_path.join("file3.txt"), "Content for file 3\n")?;

    // Add all files so they're tracked
    repo.add(&["file1.txt", "file2.txt", "file3.txt"])?;

    // Now modify them so there are changes to stash
    fs::write(repo_path.join("file1.txt"), "Modified content for file 1\n")?;
    fs::write(repo_path.join("file2.txt"), "Modified content for file 2\n")?;
    fs::write(repo_path.join("file3.txt"), "Modified content for file 3\n")?;

    let path_options = StashOptions::new().with_paths(vec!["file1.txt".into(), "file2.txt".into()]);
    let path_stash = repo.stash_push("WIP: specific files only", path_options)?;
    println!("Created stash with specific paths: {}", path_stash);

    // Demonstrate stash listing and filtering
    println!("\n=== Stash Listing and Filtering ===");

    let stashes = repo.stash_list()?;
    println!("\nAll stashes ({} total):", stashes.len());
    for stash in stashes.iter() {
        println!(
            "  [{}] {} -> {}",
            stash.index,
            stash.message,
            stash.hash.short()
        );
        println!(
            "      Branch: {} | Created: {}",
            stash.branch,
            stash.timestamp.format("%Y-%m-%d %H:%M:%S")
        );
    }

    // Test filtering
    println!("\nFiltering examples:");

    // Find stashes containing specific text
    let wip_stashes: Vec<_> = stashes.find_containing("WIP").collect();
    println!("Stashes containing 'WIP': {} found", wip_stashes.len());
    for stash in &wip_stashes {
        println!("  - {}", stash.message);
    }

    let config_stashes: Vec<_> = stashes.find_containing("config").collect();
    println!(
        "Stashes containing 'config': {} found",
        config_stashes.len()
    );

    // Get latest stash
    if let Some(latest) = stashes.latest() {
        println!("Latest stash: {}", latest.message);
    }

    // Get specific stash by index
    if let Some(second_stash) = stashes.get(1) {
        println!("Second stash: {}", second_stash.message);
    }

    // Demonstrate stash content viewing
    println!("\n=== Viewing Stash Contents ===");

    println!("\nShowing contents of latest stash:");
    let stash_contents = repo.stash_show(0)?;
    println!("{}", stash_contents);

    // Demonstrate stash application
    println!("\n=== Applying and Popping Stashes ===");

    println!("\n1. Testing stash apply (keeps stash in list):");
    let stashes_before_apply = repo.stash_list()?;
    println!("Stashes before apply: {}", stashes_before_apply.len());

    // Apply the latest stash
    repo.stash_apply(0, StashApplyOptions::new())?;
    println!("Applied stash@{{0}}");

    let stashes_after_apply = repo.stash_list()?;
    println!("Stashes after apply: {}", stashes_after_apply.len());

    // Check what was restored
    let status_after_apply = repo.status()?;
    println!("Status after apply:");
    println!(
        "  Staged files: {}",
        status_after_apply.staged_files().count()
    );
    println!(
        "  Unstaged files: {}",
        status_after_apply.unstaged_files().count()
    );
    println!(
        "  Untracked files: {}",
        status_after_apply.untracked_entries().count()
    );

    println!("\n2. Testing stash pop (removes stash from list):");

    // First, stash current changes again to have something to pop
    repo.stash_save("Temporary stash for pop test")?;

    let stashes_before_pop = repo.stash_list()?;
    println!("Stashes before pop: {}", stashes_before_pop.len());

    // Pop the latest stash
    repo.stash_pop(0, StashApplyOptions::new().with_quiet())?;
    println!("Popped stash@{{0}}");

    let stashes_after_pop = repo.stash_list()?;
    println!("Stashes after pop: {}", stashes_after_pop.len());

    // Demonstrate advanced apply options
    println!("\n3. Testing apply with index restoration:");

    // Create a stash with staged changes
    fs::write(repo_path.join("staged_file.txt"), "This will be staged\n")?;
    repo.add(&["staged_file.txt"])?;

    fs::write(
        repo_path.join("unstaged_file.txt"),
        "This will be unstaged\n",
    )?;

    repo.stash_save("Stash with staged and unstaged changes")?;

    // Apply with index restoration
    let apply_options = StashApplyOptions::new().with_index();
    repo.stash_apply(0, apply_options)?;
    println!("Applied stash with index restoration");

    let final_status = repo.status()?;
    println!("Final status after index restoration:");
    println!("  Staged files: {}", final_status.staged_files().count());
    println!(
        "  Unstaged files: {}",
        final_status.unstaged_files().count()
    );

    // Demonstrate stash management
    println!("\n=== Stash Management ===");

    // Create a few test stashes
    for i in 1..=3 {
        fs::write(
            repo_path.join(format!("test{}.txt", i)),
            format!("Test content {}\n", i),
        )?;
        repo.stash_save(&format!("Test stash {}", i))?;
    }

    let management_stashes = repo.stash_list()?;
    println!(
        "\nCreated {} test stashes for management demo",
        management_stashes.len()
    );

    // Drop a specific stash
    println!("\n1. Dropping middle stash:");
    println!("Before drop: {} stashes", management_stashes.len());

    repo.stash_drop(1)?; // Drop second stash (index 1)
    println!("Dropped stash@{{1}}");

    let after_drop = repo.stash_list()?;
    println!("After drop: {} stashes", after_drop.len());

    // Show remaining stashes
    println!("Remaining stashes:");
    for stash in after_drop.iter() {
        println!("  [{}] {}", stash.index, stash.message);
    }

    // Clear all stashes
    println!("\n2. Clearing all stashes:");
    repo.stash_clear()?;
    println!("Cleared all stashes");

    let final_stashes = repo.stash_list()?;
    println!("Stashes after clear: {}", final_stashes.len());

    // Demonstrate error handling
    println!("\n=== Error Handling ===");

    println!("\n1. Testing operations on empty stash list:");

    // Try to apply non-existent stash
    match repo.stash_apply(0, StashApplyOptions::new()) {
        Ok(_) => println!("ERROR: Should have failed to apply non-existent stash"),
        Err(e) => println!("Expected error applying non-existent stash: {}", e),
    }

    // Try to show non-existent stash
    match repo.stash_show(0) {
        Ok(_) => println!("ERROR: Should have failed to show non-existent stash"),
        Err(e) => println!("Expected error showing non-existent stash: {}", e),
    }

    // Try to drop non-existent stash
    match repo.stash_drop(0) {
        Ok(_) => println!("ERROR: Should have failed to drop non-existent stash"),
        Err(e) => println!("Expected error dropping non-existent stash: {}", e),
    }

    // Summary
    println!("\n=== Summary ===");
    println!("\nStash operations demonstrated:");
    println!("  ✓ Basic stash save and push with options");
    println!("  ✓ Stash with untracked files and keep-index");
    println!("  ✓ Stash specific paths only");
    println!("  ✓ Comprehensive stash listing and filtering");
    println!("  ✓ Stash content viewing");
    println!("  ✓ Apply vs pop operations");
    println!("  ✓ Index restoration during apply");
    println!("  ✓ Stash dropping and clearing");
    println!("  ✓ Error handling for edge cases");

    println!("\nStash options demonstrated:");
    println!("  ✓ with_untracked() - Include untracked files");
    println!("  ✓ with_keep_index() - Keep staged changes");
    println!("  ✓ with_paths() - Stash specific files only");
    println!("  ✓ with_index() - Restore staged state on apply");
    println!("  ✓ with_quiet() - Suppress output messages");

    println!("\nStash filtering demonstrated:");
    println!("  ✓ find_containing() - Search by message content");
    println!("  ✓ latest() - Get most recent stash");
    println!("  ✓ get() - Get stash by index");
    println!("  ✓ for_branch() - Filter by branch name");

    // Clean up
    println!("\nCleaning up example repository...");
    fs::remove_dir_all(&repo_path)?;
    println!("Stash operations example completed successfully!");

    Ok(())
}
