//! Repository Configuration Operations Example
//!
//! This example demonstrates the repository configuration management features:
//! - Setting and getting user configuration
//! - Managing arbitrary git configuration values
//! - Repository-scoped configuration operations
//! - Configuration integration with commit operations
//!
//! Run with: cargo run --example config_operations

use rustic_git::{Repository, Result};
use std::fs;
use std::path::Path;

fn main() -> Result<()> {
    println!("Rustic Git - Repository Configuration Operations Example\n");

    // Use a temporary directory for this example
    let repo_path = "/tmp/rustic_git_config_example";

    // Clean up any previous run
    if Path::new(repo_path).exists() {
        fs::remove_dir_all(repo_path).expect("Failed to clean up previous example");
    }

    println!("Initializing new repository at: {}", repo_path);

    // Initialize a new repository
    let repo = Repository::init(repo_path, false)?;

    // ==================== USER CONFIGURATION ====================

    println!("\n[CONFIG] Configuring git user settings...");

    // Set user configuration (convenience method)
    repo.config()
        .set_user("Alice Developer", "alice@example.com")?;
    println!("Set user configuration");

    // Verify user configuration
    let (name, email) = repo.config().get_user()?;
    println!("Current user: {} <{}>", name, email);

    // ==================== GENERAL CONFIGURATION ====================

    println!("\n[CONFIG] Setting repository configuration values...");

    // Set various git configuration values
    repo.config().set("core.autocrlf", "false")?;
    repo.config().set("core.ignorecase", "true")?;
    repo.config().set("pull.rebase", "true")?;
    repo.config().set("push.default", "simple")?;
    repo.config().set("branch.autosetupmerge", "always")?;

    println!("Set core configuration values");

    // Get and display configuration values
    println!("\n[CONFIG] Current repository configuration:");

    let configs = [
        "core.autocrlf",
        "core.ignorecase",
        "pull.rebase",
        "push.default",
        "branch.autosetupmerge",
    ];

    for config_key in &configs {
        match repo.config().get(config_key) {
            Ok(value) => println!("  {} = {}", config_key, value),
            Err(_) => println!("  {} = <not set>", config_key),
        }
    }

    // ==================== CONFIGURATION WITH COMMITS ====================

    println!("\n[COMMIT] Testing configuration with commit operations...");

    // Create a test file
    let test_file_path = format!("{}/test.txt", repo_path);
    fs::write(
        &test_file_path,
        "Hello from rustic-git configuration example!",
    )?;
    println!("Created test file: test.txt");

    // Stage the file
    repo.add(&["test.txt"])?;
    println!("Staged test.txt");

    // Create a commit (this will use our configured user)
    let commit_hash = repo.commit("Add test file with configuration example")?;
    println!("Created commit: {}", commit_hash.short());

    // ==================== CONFIGURATION MODIFICATION ====================

    println!("\n[UPDATE] Modifying configuration values...");

    // Change some configuration values
    repo.config().set("core.autocrlf", "true")?;
    repo.config()
        .set("user.email", "alice.developer@newcompany.com")?;

    println!("Updated configuration values");

    // Display updated values
    let autocrlf = repo.config().get("core.autocrlf")?;
    let (updated_name, updated_email) = repo.config().get_user()?;

    println!("Updated configuration:");
    println!("  core.autocrlf = {}", autocrlf);
    println!("  user: {} <{}>", updated_name, updated_email);

    // ==================== CONFIGURATION REMOVAL ====================

    println!("\n[REMOVE] Removing configuration values...");

    // Remove a configuration value
    repo.config().unset("branch.autosetupmerge")?;
    println!("Removed branch.autosetupmerge");

    // Try to get the removed value (should fail)
    match repo.config().get("branch.autosetupmerge") {
        Ok(value) => println!("Unexpected: branch.autosetupmerge = {}", value),
        Err(_) => println!("Confirmed: branch.autosetupmerge is not set"),
    }

    // ==================== ADVANCED CONFIGURATION ====================

    println!("\n[ADVANCED] Setting advanced configuration...");

    // Set some advanced git configuration
    repo.config().set("diff.tool", "vimdiff")?;
    repo.config().set("merge.tool", "vimdiff")?;
    repo.config().set("alias.st", "status")?;
    repo.config().set("alias.co", "checkout")?;
    repo.config().set("alias.br", "branch")?;
    repo.config().set("alias.ci", "commit")?;

    println!("Set advanced configuration (diff/merge tools and aliases)");

    // Display all custom configuration
    println!("\n[SUMMARY] Complete repository configuration summary:");

    let all_configs = [
        ("User", vec![("user.name", ""), ("user.email", "")]),
        ("Core", vec![("core.autocrlf", ""), ("core.ignorecase", "")]),
        ("Workflow", vec![("pull.rebase", ""), ("push.default", "")]),
        ("Tools", vec![("diff.tool", ""), ("merge.tool", "")]),
        (
            "Aliases",
            vec![
                ("alias.st", ""),
                ("alias.co", ""),
                ("alias.br", ""),
                ("alias.ci", ""),
            ],
        ),
    ];

    for (category, configs) in &all_configs {
        println!("\n  {}:", category);
        for (key, _) in configs {
            match repo.config().get(key) {
                Ok(value) => println!("    {} = {}", key, value),
                Err(_) => println!("    {} = <not set>", key),
            }
        }
    }

    // ==================== PRACTICAL EXAMPLE ====================

    println!("\n[TEAM] Practical example: Setting up repository for a team...");

    // Configure repository for team development
    repo.config().set("user.name", "Team Member")?;
    repo.config().set("user.email", "team@company.com")?;
    repo.config().set("core.autocrlf", "input")?;
    repo.config().set("core.safecrlf", "true")?;
    repo.config().set("pull.rebase", "true")?;
    repo.config().set("push.default", "current")?;
    repo.config().set("init.defaultBranch", "main")?;

    println!("Configured repository for team development");

    // Create another commit with the team configuration
    fs::write(
        format!("{}/team.md", repo_path),
        "# Team Development\n\nThis repository is configured for team development.",
    )?;
    repo.add(&["team.md"])?;
    let team_commit = repo.commit("Add team development documentation")?;

    println!("Created team commit: {}", team_commit.short());

    // Final verification
    let (final_name, final_email) = repo.config().get_user()?;
    println!("\n[FINAL] Final repository configuration:");
    println!("  User: {} <{}>", final_name, final_email);
    println!("  Repository configured for team development workflow");

    // ==================== CLEANUP ====================

    println!("\n[CLEANUP] Cleaning up...");
    fs::remove_dir_all(repo_path).expect("Failed to clean up example");
    println!("Example completed successfully!");

    Ok(())
}
