use rustic_git::{FastForwardMode, MergeOptions, MergeStatus, Repository, Result};
use std::{env, fs};

/// Comprehensive demonstration of merge operations in rustic-git
///
/// This example showcases:
/// - Simple branch merging
/// - Fast-forward vs non-fast-forward merges
/// - Merge conflict detection and handling
/// - Different merge strategies and options
/// - Merge status checking and abort operations
///
/// Merge operations are fundamental for collaborative development workflows.
fn main() -> Result<()> {
    println!("=== Merge Operations Demo ===\n");

    // Create a temporary directory for our example
    let temp_dir = env::temp_dir().join("rustic_git_merge_demo");

    // Clean up if exists
    if temp_dir.exists() {
        fs::remove_dir_all(&temp_dir)?;
    }

    println!("Working in temporary directory: {:?}\n", temp_dir);

    // Initialize a new repository
    let repo = Repository::init(&temp_dir, false)?;

    // Configure user for commits
    repo.config().set_user("Example User", "example@test.com")?;

    demonstrate_fast_forward_merge(&repo, &temp_dir)?;
    demonstrate_no_fast_forward_merge(&repo, &temp_dir)?;
    demonstrate_merge_conflicts(&repo, &temp_dir)?;
    demonstrate_merge_status_and_abort(&repo, &temp_dir)?;

    println!("\n=== Merge Operations Demo Complete ===");

    // Clean up
    fs::remove_dir_all(&temp_dir)?;
    Ok(())
}

fn demonstrate_fast_forward_merge(repo: &Repository, temp_dir: &std::path::Path) -> Result<()> {
    println!("--- Demonstrating Fast-Forward Merge ---\n");

    // Create initial commit
    println!("1. Creating initial commit on master...");
    let file1_path = temp_dir.join("README.md");
    fs::write(&file1_path, "# Project\n\nInitial content")?;
    repo.add(&["README.md"])?;
    let initial_commit = repo.commit("Initial commit")?;
    println!("   Created commit: {}", initial_commit);

    // Create feature branch and add commits
    println!("\n2. Creating feature branch and adding commits...");
    repo.checkout_new("feature/fast-forward", None)?;

    let file2_path = temp_dir.join("feature.txt");
    fs::write(&file2_path, "New feature implementation")?;
    repo.add(&["feature.txt"])?;
    let feature_commit = repo.commit("Add new feature")?;
    println!("   Feature commit: {}", feature_commit);

    // Switch back to master
    println!("\n3. Switching back to master...");
    let branches = repo.branches()?;
    let master_branch = branches.find("master").unwrap();
    repo.checkout(master_branch)?;
    println!("   Switched to master");

    // Perform fast-forward merge
    println!("\n4. Performing fast-forward merge...");
    let merge_status = repo.merge("feature/fast-forward")?;

    match merge_status {
        MergeStatus::FastForward(hash) => {
            println!("   ✓ Fast-forward merge completed!");
            println!("   New HEAD: {}", hash);
            println!("   Both files are now present on master");
        }
        _ => println!("   Unexpected merge result: {:?}", merge_status),
    }

    println!("   Files in repository:");
    for file in ["README.md", "feature.txt"] {
        if temp_dir.join(file).exists() {
            println!("     ✓ {}", file);
        }
    }

    Ok(())
}

fn demonstrate_no_fast_forward_merge(repo: &Repository, temp_dir: &std::path::Path) -> Result<()> {
    println!("\n--- Demonstrating No-Fast-Forward Merge ---\n");

    // Add a commit to master to prevent fast-forward
    println!("1. Adding commit to master...");
    let readme_path = temp_dir.join("README.md");
    fs::write(
        &readme_path,
        "# Project\n\nInitial content\n\n## Updates\nAdded documentation",
    )?;
    repo.add(&["README.md"])?;
    let master_commit = repo.commit("Update documentation")?;
    println!("   Master commit: {}", master_commit);

    // Create another feature branch
    println!("\n2. Creating another feature branch...");
    repo.checkout_new("feature/no-ff", None)?;

    let config_path = temp_dir.join("config.yaml");
    fs::write(&config_path, "app:\n  name: example\n  version: 1.0")?;
    repo.add(&["config.yaml"])?;
    let config_commit = repo.commit("Add configuration file")?;
    println!("   Config commit: {}", config_commit);

    // Switch back to master
    println!("\n3. Switching back to master...");
    let branches = repo.branches()?;
    let master_branch = branches.find("master").unwrap();
    repo.checkout(master_branch)?;

    // Perform no-fast-forward merge
    println!("\n4. Performing no-fast-forward merge...");
    let options = MergeOptions::new()
        .with_fast_forward(FastForwardMode::Never)
        .with_message("Merge feature/no-ff into master".to_string());

    let merge_status = repo.merge_with_options("feature/no-ff", options)?;

    match merge_status {
        MergeStatus::Success(hash) => {
            println!("   ✓ Merge commit created!");
            println!("   Merge commit: {}", hash);
            println!("   Created explicit merge commit preserving branch history");
        }
        _ => println!("   Unexpected merge result: {:?}", merge_status),
    }

    // Show the commit history
    println!("\n5. Recent commit history:");
    let commits = repo.recent_commits(3)?;
    for (i, commit) in commits.iter().enumerate() {
        println!(
            "   {}: {} - {}",
            i + 1,
            commit.hash.short(),
            commit.message.subject
        );
    }

    Ok(())
}

fn demonstrate_merge_conflicts(repo: &Repository, temp_dir: &std::path::Path) -> Result<()> {
    println!("\n--- Demonstrating Merge Conflicts ---\n");

    // Create conflicting branch
    println!("1. Creating branch with conflicting changes...");
    repo.checkout_new("feature/conflict", None)?;

    // Modify the same file differently
    let readme_path = temp_dir.join("README.md");
    fs::write(
        &readme_path,
        "# Project\n\nFeature branch changes\n\n## Updates\nAdded documentation",
    )?;
    repo.add(&["README.md"])?;
    let feature_commit = repo.commit("Update README from feature branch")?;
    println!("   Feature commit: {}", feature_commit);

    // Switch back to master and make conflicting change
    println!("\n2. Making conflicting change on master...");
    let branches = repo.branches()?;
    let master_branch = branches.find("master").unwrap();
    repo.checkout(master_branch)?;

    fs::write(
        &readme_path,
        "# Project\n\nMaster branch changes\n\n## Updates\nAdded documentation",
    )?;
    repo.add(&["README.md"])?;
    let master_conflict_commit = repo.commit("Update README from master")?;
    println!("   Master commit: {}", master_conflict_commit);

    // Attempt merge (will have conflicts)
    println!("\n3. Attempting merge (will have conflicts)...");
    let merge_status = repo.merge("feature/conflict")?;

    match merge_status {
        MergeStatus::Conflicts(files) => {
            println!("   ⚠️  Merge conflicts detected!");
            println!("   Conflicted files:");
            for file in &files {
                println!("     - {}", file.display());
            }

            // Check merge in progress
            if repo.merge_in_progress()? {
                println!("   ✓ Merge in progress status detected");
            }

            // Show conflict markers in file
            println!("\n4. Conflict markers in README.md:");
            let content = fs::read_to_string(&readme_path)?;
            for (i, line) in content.lines().enumerate() {
                if line.starts_with("<<<<<<< ")
                    || line.starts_with("======= ")
                    || line.starts_with(">>>>>>> ")
                {
                    println!("     {}: {} <-- conflict marker", i + 1, line);
                } else {
                    println!("     {}: {}", i + 1, line);
                }
            }

            // Abort the merge
            println!("\n5. Aborting merge...");
            repo.abort_merge()?;
            println!("   ✓ Merge aborted successfully");

            // Verify merge is no longer in progress
            if !repo.merge_in_progress()? {
                println!("   ✓ Repository is back to clean state");
            }
        }
        _ => println!("   Unexpected merge result: {:?}", merge_status),
    }

    Ok(())
}

fn demonstrate_merge_status_and_abort(repo: &Repository, temp_dir: &std::path::Path) -> Result<()> {
    println!("\n--- Demonstrating Merge Status and Options ---\n");

    // Create a simple feature branch
    println!("1. Creating simple feature branch...");
    repo.checkout_new("feature/simple", None)?;

    let simple_path = temp_dir.join("simple.txt");
    fs::write(&simple_path, "Simple feature content")?;
    repo.add(&["simple.txt"])?;
    repo.commit("Add simple feature")?;

    // Switch back to master
    let branches = repo.branches()?;
    let master_branch = branches.find("master").unwrap();
    repo.checkout(master_branch)?;

    // Test merge with different options
    println!("\n2. Testing merge with custom options...");
    let options = MergeOptions::new()
        .with_fast_forward(FastForwardMode::Auto)
        .with_message("Integrate simple feature".to_string());

    let merge_status = repo.merge_with_options("feature/simple", options)?;

    match merge_status {
        MergeStatus::FastForward(hash) => {
            println!("   ✓ Fast-forward merge completed: {}", hash);
        }
        MergeStatus::Success(hash) => {
            println!("   ✓ Merge commit created: {}", hash);
        }
        MergeStatus::UpToDate => {
            println!("   ✓ Already up to date");
        }
        MergeStatus::Conflicts(_) => {
            println!("   ⚠️  Unexpected conflicts");
        }
    }

    // Show final repository state
    println!("\n3. Final repository state:");
    let status = repo.status()?;
    println!(
        "   Working directory clean: {}",
        status.staged_files().count() == 0 && status.unstaged_files().count() == 0
    );

    let commits = repo.recent_commits(5)?;
    println!("   Recent commits:");
    for (i, commit) in commits.iter().enumerate() {
        println!(
            "     {}: {} - {}",
            i + 1,
            commit.hash.short(),
            commit.message.subject
        );
    }

    Ok(())
}
