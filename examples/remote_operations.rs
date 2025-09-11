//! Remote Operations Example
//!
//! This example demonstrates comprehensive remote management capabilities:
//! - Add, remove, and rename remotes
//! - List remotes and get their URLs
//! - Fetch and push operations with options
//! - Clone repositories
//! - Handle various remote scenarios and error cases
//!
//! Run with: cargo run --example remote_operations

use rustic_git::{FetchOptions, PushOptions, Repository, Result};
use std::{env, fs};

fn main() -> Result<()> {
    println!("Rustic Git - Remote Operations Example\n");

    let base_path = env::temp_dir().join("rustic_git_remote_example");
    let repo_path = base_path.join("main_repo");
    let clone_path = base_path.join("cloned_repo");

    // Clean up any previous runs
    if base_path.exists() {
        fs::remove_dir_all(&base_path).expect("Failed to clean up previous example");
    }
    fs::create_dir_all(&base_path)?;

    println!("=== Repository Setup ===\n");

    // Initialize repository
    println!("Initializing repository for remote demonstrations...");
    let repo = Repository::init(&repo_path, false)?;
    println!("Repository initialized at: {}", repo_path.display());

    // Create initial commit so we have something to work with
    fs::write(
        repo_path.join("README.md"),
        "# Remote Operations Demo\n\nDemonstrating rustic-git remote management capabilities.",
    )?;
    repo.add(&["README.md"])?;
    repo.commit("Initial commit for remote operations demo")?;
    println!("Created initial commit\n");

    println!("=== Basic Remote Management ===\n");

    // Check initial remote state
    println!("Checking initial remote state:");
    let remotes = repo.list_remotes()?;
    println!("   Initial remotes count: {}", remotes.len());
    if remotes.is_empty() {
        println!("   No remotes configured (as expected)");
    }
    println!();

    // Add remotes
    println!("Adding remotes...");
    repo.add_remote("origin", "https://github.com/user/demo-repo.git")?;
    println!("   Added 'origin' remote");

    repo.add_remote("upstream", "https://github.com/original/demo-repo.git")?;
    println!("   Added 'upstream' remote");

    repo.add_remote("fork", "git@github.com:user/fork-repo.git")?;
    println!("   Added 'fork' remote (SSH URL)");
    println!();

    // List remotes
    println!("Listing all remotes:");
    let remotes = repo.list_remotes()?;
    println!("   Total remotes: {}", remotes.len());

    for remote in remotes.iter() {
        println!("   {} -> {}", remote.name, remote.fetch_url);
        if let Some(push_url) = &remote.push_url {
            println!("     Push URL: {}", push_url);
        }
    }
    println!();

    // Get specific remote URLs
    println!("Getting specific remote URLs:");
    let origin_url = repo.get_remote_url("origin")?;
    println!("   Origin URL: {}", origin_url);

    let upstream_url = repo.get_remote_url("upstream")?;
    println!("   Upstream URL: {}", upstream_url);
    println!();

    // Rename a remote
    println!("Renaming 'fork' remote to 'my-fork'...");
    repo.rename_remote("fork", "my-fork")?;
    println!("   Remote renamed successfully");

    // Verify rename
    let remotes = repo.list_remotes()?;
    let renamed_remote = remotes.find("my-fork");
    match renamed_remote {
        Some(remote) => println!(
            "   Found renamed remote: {} -> {}",
            remote.name, remote.fetch_url
        ),
        None => println!("   Error: Could not find renamed remote"),
    }

    // Verify old name is gone
    if remotes.find("fork").is_none() {
        println!("   Confirmed: old 'fork' remote no longer exists");
    }
    println!();

    println!("=== Remote Operations with Options ===\n");

    // Demonstrate fetch options
    println!("Fetch operations (simulated - no actual network calls):");
    println!("   Basic fetch from origin:");
    match repo.fetch("origin") {
        Ok(_) => println!("     ✓ Fetch completed successfully"),
        Err(e) => println!("     ⚠ Fetch failed (expected): {}", e),
    }

    println!("   Fetch with options (prune + tags):");
    let fetch_options = FetchOptions::new().with_prune().with_tags();
    match repo.fetch_with_options("origin", fetch_options) {
        Ok(_) => println!("     ✓ Fetch with options completed successfully"),
        Err(e) => println!("     ⚠ Fetch with options failed (expected): {}", e),
    }

    println!("   Fetch all remotes:");
    let fetch_all_options = FetchOptions::new().with_all_remotes();
    match repo.fetch_with_options("", fetch_all_options) {
        Ok(_) => println!("     ✓ Fetch all completed successfully"),
        Err(e) => println!("     ⚠ Fetch all failed (expected): {}", e),
    }
    println!();

    // Demonstrate push options
    println!("Push operations (simulated - no actual network calls):");
    println!("   Basic push to origin:");
    match repo.push("origin", "main") {
        Ok(_) => println!("     ✓ Push completed successfully"),
        Err(e) => println!("     ⚠ Push failed (expected): {}", e),
    }

    println!("   Push with upstream tracking:");
    let push_options = PushOptions::new().with_set_upstream();
    match repo.push_with_options("origin", "main", push_options) {
        Ok(_) => println!("     ✓ Push with upstream completed successfully"),
        Err(e) => println!("     ⚠ Push with upstream failed (expected): {}", e),
    }

    println!("   Force push with tags:");
    let force_push_options = PushOptions::new().with_force().with_tags();
    match repo.push_with_options("my-fork", "feature-branch", force_push_options) {
        Ok(_) => println!("     ✓ Force push with tags completed successfully"),
        Err(e) => println!("     ⚠ Force push with tags failed (expected): {}", e),
    }
    println!();

    println!("=== Clone Operations ===\n");

    // Note: We can't actually clone from the URLs we added since they're fake,
    // but we can demonstrate the API and show how it would work
    println!("Clone operation demonstration:");
    println!("   Attempting to clone a repository...");

    // This will fail since the URL doesn't exist, but demonstrates the API
    match Repository::clone("https://github.com/nonexistent/fake-repo.git", &clone_path) {
        Ok(_repo) => {
            println!("     ✓ Clone completed successfully");
            println!("     Cloned repository location: {}", clone_path.display());
        }
        Err(e) => {
            println!("     ⚠ Clone failed (expected for demo): {}", e);
            println!("     In real usage, provide a valid repository URL");
        }
    }
    println!();

    println!("=== Error Handling and Edge Cases ===\n");

    // Test error cases
    println!("Testing error conditions:");

    // Try to get URL for non-existent remote
    println!("   Getting URL for non-existent remote:");
    match repo.get_remote_url("nonexistent") {
        Ok(url) => println!("     Unexpected success: {}", url),
        Err(e) => println!("     ✓ Expected error: {}", e),
    }

    // Try to remove non-existent remote
    println!("   Removing non-existent remote:");
    match repo.remove_remote("nonexistent") {
        Ok(_) => println!("     Unexpected success"),
        Err(e) => println!("     ✓ Expected error: {}", e),
    }

    // Try to add duplicate remote
    println!("   Adding duplicate remote:");
    match repo.add_remote("origin", "https://github.com/duplicate/repo.git") {
        Ok(_) => println!("     Unexpected success (git allows URL changes)"),
        Err(e) => println!("     Error: {}", e),
    }
    println!();

    println!("=== Remote Cleanup Operations ===\n");

    // Remove remotes one by one
    println!("Removing remotes:");

    println!("   Removing 'upstream' remote...");
    repo.remove_remote("upstream")?;

    println!("   Removing 'my-fork' remote...");
    repo.remove_remote("my-fork")?;

    println!("   Removing 'origin' remote...");
    repo.remove_remote("origin")?;

    // Verify all remotes are gone
    let final_remotes = repo.list_remotes()?;
    println!("   Final remote count: {}", final_remotes.len());

    if final_remotes.is_empty() {
        println!("   ✓ All remotes successfully removed");
    } else {
        println!("   ⚠ Some remotes still remain:");
        for remote in final_remotes.iter() {
            println!("     - {}", remote.name);
        }
    }
    println!();

    println!("=== Advanced Remote Information ===\n");

    // Re-add a remote for advanced operations demo
    repo.add_remote("demo", "https://github.com/demo/advanced-repo.git")?;

    // Show comprehensive remote information
    let remotes = repo.list_remotes()?;
    for remote in remotes.iter() {
        println!("Remote Details:");
        println!("   Name: {}", remote.name);
        println!("   Fetch URL: {}", remote.fetch_url);
        println!("   Push URL: {}", remote.push_url());
        println!("   Uses separate push URL: {}", remote.push_url.is_some());

        // Validate URL format
        if remote.fetch_url.starts_with("https://") {
            println!("   Protocol: HTTPS");
        } else if remote.fetch_url.starts_with("git@") {
            println!("   Protocol: SSH");
        } else if remote.fetch_url.starts_with("git://") {
            println!("   Protocol: Git");
        } else {
            println!("   Protocol: Other/Local");
        }
    }
    println!();

    println!("=== Summary ===\n");

    println!("Remote operations demonstration completed!");
    println!("  Repository: {}", repo_path.display());

    let final_remotes = repo.list_remotes()?;
    println!("  Final remotes configured: {}", final_remotes.len());

    for remote in final_remotes.iter() {
        println!("    - {} ({})", remote.name, remote.fetch_url);
    }

    println!("\nOperations demonstrated:");
    println!("  ✓ Adding remotes with different URL formats");
    println!("  ✓ Listing and inspecting remotes");
    println!("  ✓ Getting specific remote URLs");
    println!("  ✓ Renaming remotes");
    println!("  ✓ Removing remotes");
    println!("  ✓ Fetch operations with options");
    println!("  ✓ Push operations with options");
    println!("  ✓ Clone API demonstration");
    println!("  ✓ Error handling for invalid operations");
    println!("  ✓ Remote information analysis");

    // Clean up
    println!("\nCleaning up example repositories...");
    fs::remove_dir_all(&base_path)?;
    println!("Remote operations example completed!");

    Ok(())
}
