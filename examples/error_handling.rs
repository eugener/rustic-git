//! Error Handling Example
//!
//! This example demonstrates comprehensive error handling patterns:
//! - Handle GitError variants (IoError, CommandFailed)
//! - Recovery strategies for common error scenarios
//! - Best practices for error propagation
//! - Graceful degradation when operations fail
//!
//! Run with: cargo run --example error_handling

use rustic_git::{Repository, GitError, Result};
use std::fs;
use std::path::Path;

fn main() -> Result<()> {
    println!("Rustic Git - Error Handling Example\n");

    let base_path = "/tmp/rustic_git_error_example";
    let repo_path = format!("{}/test_repo", base_path);
    
    // Clean up any previous runs
    if Path::new(base_path).exists() {
        fs::remove_dir_all(base_path).expect("Failed to clean up previous example");
    }
    fs::create_dir_all(base_path).expect("Failed to create base directory");

    println!("=== GitError Types and Handling ===\n");

    // Demonstrate different error types and handling strategies
    demonstrate_repository_errors(&repo_path)?;
    demonstrate_file_operation_errors(&repo_path)?;
    demonstrate_git_command_errors(&repo_path)?;
    demonstrate_error_recovery_patterns(&repo_path)?;
    demonstrate_error_propagation_strategies(base_path)?;

    // Clean up
    println!("Cleaning up error handling examples...");
    fs::remove_dir_all(base_path)?;
    println!("Error handling example completed successfully!");

    Ok(())
}

/// Demonstrate repository-related errors
fn demonstrate_repository_errors(repo_path: &str) -> Result<()> {
    println!("Repository Error Scenarios:\n");

    // 1. Opening non-existent repository
    println!("1. Attempting to open non-existent repository:");
    match Repository::open("/definitely/does/not/exist") {
        Ok(_) => println!("   Unexpectedly succeeded"),
        Err(GitError::IoError(msg)) => {
            println!("   IoError caught: {}", msg);
            println!("   This typically happens when the path doesn't exist");
        }
        Err(GitError::CommandFailed(msg)) => {
            println!("   CommandFailed caught: {}", msg);
            println!("   Git command failed - path exists but isn't a repo");
        }
    }

    // 2. Opening a file as a repository
    let fake_repo_path = format!("{}_fake.txt", repo_path);
    fs::write(&fake_repo_path, "This is not a git repository")?;
    
    println!("\n2. Attempting to open regular file as repository:");
    match Repository::open(&fake_repo_path) {
        Ok(_) => println!("   Unexpectedly succeeded"),
        Err(GitError::CommandFailed(msg)) => {
            println!("   CommandFailed caught: {}", msg);
            println!("   Git recognized the path but it's not a repository");
        }
        Err(GitError::IoError(msg)) => {
            println!("   IoError caught: {}", msg);
        }
    }
    
    fs::remove_file(&fake_repo_path)?;

    // 3. Initializing repository with invalid path
    println!("\n3. Attempting to initialize repository with problematic path:");
    
    // Try to initialize in a location that might cause issues
    match Repository::init("/root/definitely_no_permission", false) {
        Ok(_) => println!("   Unexpectedly succeeded (you might be running as root!)"),
        Err(GitError::IoError(msg)) => {
            println!("   IoError caught: {}", msg);
            println!("   Likely a permission issue");
        }
        Err(GitError::CommandFailed(msg)) => {
            println!("   CommandFailed caught: {}", msg);
            println!("   Git init command failed");
        }
    }

    println!();
    Ok(())
}

/// Demonstrate file operation related errors
fn demonstrate_file_operation_errors(repo_path: &str) -> Result<()> {
    println!("File Operation Error Scenarios:\n");

    // Set up a valid repository first
    let repo = Repository::init(repo_path, false)?;
    
    // Create some test files
    fs::write(format!("{}/test.txt", repo_path), "Test content")?;
    repo.add(&["test.txt"])?;
    repo.commit("Initial commit")?;

    // 1. Adding non-existent files
    println!("1. Attempting to add non-existent files:");
    match repo.add(&["does_not_exist.txt", "also_missing.txt"]) {
        Ok(_) => println!("   Unexpectedly succeeded"),
        Err(GitError::CommandFailed(msg)) => {
            println!("   CommandFailed caught: {}", msg);
            println!("   Git add failed because files don't exist");
        }
        Err(GitError::IoError(msg)) => {
            println!("   IoError caught: {}", msg);
        }
    }

    // 2. Mixed valid and invalid files
    println!("\n2. Adding mix of valid and invalid files:");
    fs::write(format!("{}/valid.txt", repo_path), "Valid file")?;
    
    match repo.add(&["valid.txt", "invalid.txt"]) {
        Ok(_) => {
            println!("   Partially succeeded - some Git versions allow this");
            // Check what actually got staged
            let status = repo.status()?;
            println!("   {} files staged despite error", status.files.len());
        }
        Err(GitError::CommandFailed(msg)) => {
            println!("   CommandFailed caught: {}", msg);
            println!("   Entire add operation failed due to invalid file");
            
            // Try recovery: add valid files individually
            println!("   Recovery: Adding valid files individually...");
            match repo.add(&["valid.txt"]) {
                Ok(_) => println!("      Successfully added valid.txt"),
                Err(e) => println!("      Recovery failed: {:?}", e),
            }
        }
        Err(GitError::IoError(msg)) => {
            println!("   IoError caught: {}", msg);
        }
    }

    println!();
    Ok(())
}

/// Demonstrate Git command related errors
fn demonstrate_git_command_errors(repo_path: &str) -> Result<()> {
    println!("Git Command Error Scenarios:\n");

    let repo = Repository::open(repo_path)?;

    // 1. Empty commit (no staged changes)
    println!("1. Attempting commit with no staged changes:");
    match repo.commit("Empty commit attempt") {
        Ok(hash) => {
            println!("   Unexpectedly succeeded: {}", hash.short());
            println!("   Some Git configurations allow empty commits");
        }
        Err(GitError::CommandFailed(msg)) => {
            println!("   CommandFailed caught: {}", msg);
            println!("   Git requires changes to commit (normal behavior)");
        }
        Err(GitError::IoError(msg)) => {
            println!("   IoError caught: {}", msg);
        }
    }

    // 2. Commit with problematic message
    println!("\n2. Testing commit message edge cases:");
    
    // Stage a file for testing
    fs::write(format!("{}/commit_test.txt", repo_path), "Content for commit testing")?;
    repo.add(&["commit_test.txt"])?;

    // Very long commit message
    let very_long_message = "A ".repeat(1000) + "very long commit message";
    match repo.commit(&very_long_message) {
        Ok(hash) => {
            println!("   Long commit message succeeded: {}", hash.short());
            println!("   Git handled the long message fine");
        }
        Err(GitError::CommandFailed(msg)) => {
            println!("   Long commit message failed: {}", msg);
        }
        Err(GitError::IoError(msg)) => {
            println!("   IoError with long message: {}", msg);
        }
    }

    println!();
    Ok(())
}

/// Demonstrate error recovery patterns
fn demonstrate_error_recovery_patterns(repo_path: &str) -> Result<()> {
    println!("Error Recovery Patterns:\n");

    let repo = Repository::open(repo_path)?;

    // Pattern 1: Retry with different approach
    println!("1. Retry Pattern - Graceful degradation:");
    
    // Try to add specific files, fall back to add_all on failure
    let files_to_add = ["missing1.txt", "missing2.txt", "missing3.txt"];
    
    println!("   Attempting to add specific files...");
    match repo.add(&files_to_add) {
        Ok(_) => println!("      Specific files added successfully"),
        Err(e) => {
            println!("      Specific files failed: {:?}", e);
            println!("      Falling back to add_all()...");
            
            match repo.add_all() {
                Ok(_) => {
                    let status = repo.status()?;
                    println!("      add_all() succeeded, {} files staged", status.files.len());
                }
                Err(fallback_error) => {
                    println!("      Fallback also failed: {:?}", fallback_error);
                }
            }
        }
    }

    // Pattern 2: Partial success handling
    println!("\n2. Partial Success Pattern:");
    
    // Create some files with known issues
    fs::write(format!("{}/good1.txt", repo_path), "Good file 1")?;
    fs::write(format!("{}/good2.txt", repo_path), "Good file 2")?;
    // Don't create bad1.txt - it will be missing

    let mixed_files = ["good1.txt", "bad1.txt", "good2.txt"];
    
    println!("   Attempting to add mixed valid/invalid files...");
    match repo.add(&mixed_files) {
        Ok(_) => println!("      All files added (unexpected success)"),
        Err(GitError::CommandFailed(msg)) => {
            println!("      Batch add failed: {}", msg);
            println!("      Recovery: Adding files individually...");
            
            let mut successful_adds = 0;
            let mut failed_adds = 0;
            
            for file in &mixed_files {
                match repo.add(&[file]) {
                    Ok(_) => {
                        successful_adds += 1;
                        println!("         Added: {}", file);
                    }
                    Err(_) => {
                        failed_adds += 1;
                        println!("         Failed: {}", file);
                    }
                }
            }
            
            println!("      Results: {} succeeded, {} failed", successful_adds, failed_adds);
        }
        Err(GitError::IoError(msg)) => {
            println!("      IoError during batch add: {}", msg);
        }
    }

    // Pattern 3: Status checking before operations
    println!("\n3. Preventive Pattern - Check before operation:");
    
    println!("   Checking repository status before commit...");
    let status = repo.status()?;
    
    if status.is_clean() {
        println!("      Repository is clean - no commit needed");
    } else {
        println!("      Repository has {} changes", status.files.len());
        
        // Show what would be committed
        for (file_status, filename) in &status.files {
            println!("         {:?}: {}", file_status, filename);
        }
        
        // Safe commit since we know there are changes
        match repo.commit("Commit after status check") {
            Ok(hash) => println!("      Safe commit succeeded: {}", hash.short()),
            Err(e) => println!("      Even safe commit failed: {:?}", e),
        }
    }

    println!();
    Ok(())
}

/// Demonstrate error propagation strategies
fn demonstrate_error_propagation_strategies(base_path: &str) -> Result<()> {
    println!("Error Propagation Strategies:\n");

    // Strategy 1: Early return with ?
    println!("1. Early Return Strategy (using ?):");
    match workflow_with_early_return(base_path) {
        Ok(message) => println!("      Workflow completed: {}", message),
        Err(e) => println!("      Workflow failed early: {:?}", e),
    }

    // Strategy 2: Collect all errors
    println!("\n2. Error Collection Strategy:");
    let results = workflow_with_error_collection(base_path);
    
    let successful = results.iter().filter(|r| r.is_ok()).count();
    let failed = results.iter().filter(|r| r.is_err()).count();
    
    println!("      Operations: {} succeeded, {} failed", successful, failed);
    
    for (i, result) in results.iter().enumerate() {
        match result {
            Ok(msg) => println!("         Step {}: {}", i + 1, msg),
            Err(e) => println!("         Step {}: {:?}", i + 1, e),
        }
    }

    // Strategy 3: Error context enrichment
    println!("\n3. Error Context Strategy:");
    match workflow_with_context(base_path) {
        Ok(message) => println!("      Contextual workflow: {}", message),
        Err(e) => println!("      Contextual workflow failed: {:?}", e),
    }

    println!();
    Ok(())
}

/// Workflow that returns early on first error
fn workflow_with_early_return(base_path: &str) -> Result<String> {
    let repo_path = format!("{}/early_return_test", base_path);
    
    // This will propagate any error immediately
    let repo = Repository::init(&repo_path, false)?;
    
    fs::write(format!("{}/file1.txt", repo_path), "Content 1")?;
    repo.add(&["file1.txt"])?;
    
    let hash = repo.commit("Early return workflow commit")?;
    
    // Clean up
    fs::remove_dir_all(&repo_path)?;
    
    Ok(format!("Completed with commit {}", hash.short()))
}

/// Workflow that collects all errors instead of failing fast
fn workflow_with_error_collection(base_path: &str) -> Vec<Result<String>> {
    let repo_path = format!("{}/error_collection_test", base_path);
    let mut results = Vec::new();
    
    // Step 1: Initialize repo
    results.push(
        Repository::init(&repo_path, false)
            .map(|_| "Repository initialized".to_string())
    );
    
    // Step 2: Add files (some may fail)
    let files_to_create = ["good.txt", "another_good.txt"];
    
    for file in &files_to_create {
        results.push(
            fs::write(format!("{}/{}", repo_path, file), "Content")
                .map_err(GitError::from)
                .map(|_| format!("Created {}", file))
        );
    }
    
    // Step 3: Try to add files (continue even if repo init failed)
    if let Ok(repo) = Repository::open(&repo_path) {
        results.push(
            repo.add(&files_to_create)
                .map(|_| "Files added to staging".to_string())
        );
        
        results.push(
            repo.commit("Error collection workflow")
                .map(|hash| format!("Committed: {}", hash.short()))
        );
    } else {
        results.push(Err(GitError::CommandFailed("Could not open repo for adding files".to_string())));
        results.push(Err(GitError::CommandFailed("Could not open repo for commit".to_string())));
    }
    
    // Cleanup (don't add to results as it's not part of main workflow)
    let _ = fs::remove_dir_all(&repo_path);
    
    results
}

/// Workflow with enhanced error context
fn workflow_with_context(base_path: &str) -> Result<String> {
    let repo_path = format!("{}/context_test", base_path);
    
    // Add context to errors
    let repo = Repository::init(&repo_path, false)
        .inspect_err(|_e| {
            eprintln!("Context: Failed to initialize repository at {}", repo_path);
        })?;
    
    // Create file with context
    fs::write(format!("{}/context_file.txt", repo_path), "Content with context")
        .map_err(|e| {
            eprintln!("Context: Failed to create context_file.txt");
            GitError::from(e)
        })?;
    
    // Add with context
    repo.add(&["context_file.txt"])
        .inspect_err(|_e| {
            eprintln!("Context: Failed to stage context_file.txt");
        })?;
    
    // Commit with context
    let hash = repo.commit("Context workflow commit")
        .inspect_err(|_e| {
            eprintln!("Context: Failed to create commit");
        })?;
    
    // Clean up
    fs::remove_dir_all(&repo_path)?;
    
    Ok(format!("Context workflow completed: {}", hash.short()))
}