//! Commit Workflows Example
//!
//! This example demonstrates commit operations and Hash type usage:
//! - Basic commits with commit()
//! - Commits with custom authors using commit_with_author()
//! - Hash type methods (full hash, short hash, display)
//! - Error handling for empty commits
//! - Complete git workflows from init to commit
//!
//! Run with: cargo run --example commit_workflows

use rustic_git::{Repository, Hash, Result};
use std::fs;
use std::path::Path;

fn main() -> Result<()> {
    println!("Rustic Git - Commit Workflows Example\n");

    let repo_path = "/tmp/rustic_git_commit_example";
    
    // Clean up any previous run
    if Path::new(repo_path).exists() {
        fs::remove_dir_all(repo_path).expect("Failed to clean up previous example");
    }

    // Initialize repository
    println!("Setting up repository for commit demonstrations...");
    let repo = Repository::init(repo_path, false)?;
    println!("Repository initialized\n");

    println!("=== Basic Commit Operations ===\n");

    // Create initial files
    println!("Creating initial project files...");
    fs::create_dir_all(format!("{}/src", repo_path))?;
    
    fs::write(
        format!("{}/README.md", repo_path),
        "# Commit Demo Project\n\nThis project demonstrates commit workflows with rustic-git.\n"
    )?;
    
    fs::write(
        format!("{}/src/main.rs", repo_path),
        r#"fn main() {
    println!("Hello, Commit Demo!");
}
"#
    )?;
    
    fs::write(format!("{}/Cargo.toml", repo_path), r#"[package]
name = "commit-demo"
version = "0.1.0"
edition = "2021"
"#)?;
    
    println!("Created README.md, src/main.rs, and Cargo.toml");

    // Stage and commit with basic commit()
    println!("\nStaging files for first commit...");
    repo.add_all()?;
    
    println!("Creating first commit with basic commit() method:");
    let first_hash = repo.commit("Initial commit: Add project structure")?;
    
    println!("First commit created!");
    display_hash_info(&first_hash, "First commit");
    println!();

    println!("=== Hash Type Demonstrations ===\n");

    println!("Hash type methods and usage:");
    
    // Demonstrate different ways to work with Hash
    let hash_as_string: String = first_hash.to_string();
    let hash_as_str: &str = first_hash.as_str();
    let short_hash: &str = first_hash.short();
    
    println!("   Hash conversions:");
    println!("      as_str(): '{}'", hash_as_str);
    println!("      short(): '{}'", short_hash);
    println!("      to_string(): '{}'", hash_as_string);
    println!("      Display: '{}'", first_hash);
    
    // Demonstrate Hash equality and cloning
    let cloned_hash = first_hash.clone();
    println!("\n   Hash operations:");
    println!("      Original == Clone: {}", first_hash == cloned_hash);
    println!("      Hash length: {} characters", first_hash.as_str().len());
    println!("      Short hash length: {} characters", first_hash.short().len());
    
    // Create Hash from different sources for demonstration
    let hash_from_string: Hash = "1234567890abcdef".to_string().into();
    let hash_from_str: Hash = "fedcba0987654321".into();
    
    println!("      Hash from String: {}", hash_from_string.short());
    println!("      Hash from &str: {}", hash_from_str.short());
    println!();

    println!("=== Commits with Custom Authors ===\n");

    // Create more files to commit with custom author
    println!("Adding features for custom author commit...");
    fs::create_dir_all(format!("{}/tests", repo_path))?;
    
    fs::write(format!("{}/src/lib.rs", repo_path), r#"//! Commit demo library

pub fn greet(name: &str) -> String {
    format!("Hello, {}! This is a commit demo.", name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_greet() {
        assert_eq!(greet("Alice"), "Hello, Alice! This is a commit demo.");
    }
}
"#)?;

    fs::write(format!("{}/tests/integration_test.rs", repo_path), r#"use commit_demo::greet;

#[test]
fn test_integration() {
    let result = greet("Integration");
    assert!(result.contains("Integration"));
    assert!(result.contains("commit demo"));
}
"#)?;

    println!("Created src/lib.rs and tests/integration_test.rs");

    // Stage and commit with custom author
    repo.add_all()?;
    println!("\nCreating commit with custom author:");
    let second_hash = repo.commit_with_author(
        "Add library code and tests\n\n- Implement greet function with proper documentation\n- Add unit tests and integration tests\n- Prepare for version 0.2.0 release",
        "Jane Developer <jane.dev@example.com>"
    )?;
    
    println!("Commit with custom author created!");
    display_hash_info(&second_hash, "Second commit (custom author)");
    println!();

    println!("=== Multiple Commit Workflow ===\n");

    // Demonstrate a series of commits
    let mut commit_hashes = vec![first_hash, second_hash];
    
    // Commit 3: Update version
    println!("Step 1: Update version information...");
    fs::write(format!("{}/Cargo.toml", repo_path), r#"[package]
name = "commit-demo"
version = "0.2.0"
edition = "2021"
description = "A demo project for commit workflows"
"#)?;

    repo.add(&["Cargo.toml"])?;
    let third_hash = repo.commit("Bump version to 0.2.0 and add description")?;
    commit_hashes.push(third_hash);
    
    // Commit 4: Add documentation
    println!("Step 2: Add documentation...");
    fs::write(format!("{}/CHANGELOG.md", repo_path), r#"# Changelog

## [0.2.0] - 2024-01-01

### Added
- Library functionality with greet function
- Comprehensive test suite
- Project documentation

## [0.1.0] - 2024-01-01

### Added
- Initial project structure
- Basic Cargo configuration
"#)?;

    repo.add(&["CHANGELOG.md"])?;
    let fourth_hash = repo.commit_with_author(
        "docs: Add CHANGELOG with version history",
        "Doc Writer <docs@example.com>"
    )?;
    commit_hashes.push(fourth_hash);
    
    // Commit 5: Final polish
    println!("Step 3: Final polish...");
    fs::write(format!("{}/README.md", repo_path), r#"# Commit Demo Project

This project demonstrates commit workflows with rustic-git.

## Features

- Clean, type-safe Git operations
- Comprehensive commit history
- Multiple author support
- Hash management utilities

## Usage

```rust
use commit_demo::greet;

fn main() {
    println!("{}", greet("World"));
}
```

## Version

Current version: 0.2.0

See CHANGELOG.md for version history.
"#)?;

    repo.add(&["README.md"])?;
    let fifth_hash = repo.commit("docs: Enhance README with usage examples and features")?;
    commit_hashes.push(fifth_hash);

    println!("\nComplete commit history created!");
    
    // Display all commits
    println!("\n=== Commit History Summary ===\n");
    
    for (i, hash) in commit_hashes.iter().enumerate() {
        println!("{}. Commit {}", i + 1, i + 1);
        display_hash_info(hash, &format!("Commit {}", i + 1));
        println!();
    }

    // Compare hashes
    println!("Hash comparisons:");
    println!("   First commit == Last commit: {}", commit_hashes[0] == commit_hashes[4]);
    println!("   All hashes unique: {}", all_unique(&commit_hashes));
    
    // Show short hashes for all commits
    println!("\nAll commit short hashes:");
    for (i, hash) in commit_hashes.iter().enumerate() {
        println!("   {}: {}", i + 1, hash.short());
    }
    println!();

    println!("=== Error Handling for Commits ===\n");

    // Try to commit with nothing staged (should fail)
    println!("Testing commit with no staged changes:");
    match repo.commit("This should fail - no changes") {
        Ok(_hash) => println!("   Unexpectedly succeeded with empty commit"),
        Err(e) => {
            println!("   Expected error for empty commit: {:?}", e);
            println!("   This is normal behavior - Git requires changes to commit");
        }
    }

    // Try commit with custom author but no changes (should also fail)
    println!("\nTesting custom author commit with no changes:");
    match repo.commit_with_author("This should also fail", "Test Author <test@example.com>") {
        Ok(_hash) => println!("   Unexpectedly succeeded with empty custom author commit"),
        Err(e) => {
            println!("   Expected error for empty commit with custom author: {:?}", e);
        }
    }

    // Test commit with empty message (Git might handle this differently)
    println!("\nTesting commit with empty message:");
    
    // Create a change to commit
    fs::write(format!("{}/temp_for_empty_message.txt", repo_path), "temp content")?;
    repo.add(&["temp_for_empty_message.txt"])?;
    
    match repo.commit("") {
        Ok(hash) => {
            println!("   Commit with empty message succeeded: {}", hash.short());
            println!("   Some Git configurations allow empty commit messages");
        }
        Err(e) => {
            println!("   Empty commit message rejected: {:?}", e);
        }
    }

    println!();

    println!("=== Final Repository State ===\n");

    let final_status = repo.status()?;
    if final_status.is_clean() {
        println!("Repository is clean - all changes committed!");
    } else {
        println!("Repository has {} uncommitted changes", final_status.files.len());
    }

    println!("\nWorkflow summary:");
    println!("   Total commits created: {}", commit_hashes.len());
    println!("   Hash examples demonstrated: [OK]");
    println!("   Custom author commits: [OK]");
    println!("   Error handling tested: [OK]");

    // Clean up
    println!("\nCleaning up example repository...");
    fs::remove_dir_all(repo_path)?;
    println!("Commit workflows example completed!");

    Ok(())
}

/// Display comprehensive information about a Hash
fn display_hash_info(hash: &Hash, context: &str) {
    println!("   {}:", context);
    println!("      Full hash: {}", hash);
    println!("      Short hash: {}", hash.short());
    println!("      Hash length: {} chars", hash.as_str().len());
    
    // Show first and last few characters for visual reference
    let full = hash.as_str();
    if full.len() >= 10 {
        println!("      Pattern: {}...{}", &full[..5], &full[full.len()-5..]);
    }
}

/// Check if all hashes in a vector are unique
fn all_unique(hashes: &[Hash]) -> bool {
    let mut seen = std::collections::HashSet::new();
    for hash in hashes {
        if !seen.insert(hash.as_str()) {
            return false;
        }
    }
    true
}