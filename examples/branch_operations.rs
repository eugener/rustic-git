use rustic_git::{Repository, Result};
use std::{env, fs};

fn main() -> Result<()> {
    let test_path = env::temp_dir().join("rustic_git_branch_example");

    // Clean up if exists
    if test_path.exists() {
        fs::remove_dir_all(&test_path).unwrap();
    }

    // Create a test repository
    let repo = Repository::init(&test_path, false)?;
    println!("Created repository at: {}", test_path.display());

    // Create initial commit so we have a valid HEAD
    fs::write(test_path.join("README.md"), "# Branch Operations Demo\n").unwrap();
    repo.add(&["README.md"])?;
    repo.commit("Initial commit")?;
    println!("Created initial commit");

    // List all branches
    let branches = repo.branches()?;
    println!("\n=== Initial Branches ===");
    for branch in branches.iter() {
        println!("  {}", branch);
    }

    // Get current branch
    if let Some(current) = repo.current_branch()? {
        println!(
            "\nCurrent branch: {} ({})",
            current.name,
            current.commit_hash.short()
        );
    }

    // Create new branches
    println!("\n=== Creating Branches ===");
    let feature_branch = repo.create_branch("feature/new-api", None)?;
    println!("Created branch: {}", feature_branch.name);

    let bugfix_branch = repo.create_branch("bugfix/issue-123", Some("HEAD"))?;
    println!("Created branch: {}", bugfix_branch.name);

    // List branches again
    let branches = repo.branches()?;
    println!("\n=== After Creating Branches ===");
    for branch in branches.local() {
        println!("  {} (local)", branch);
    }

    // Create and checkout a new branch
    println!("\n=== Creating and Checking Out Branch ===");
    let dev_branch = repo.checkout_new("develop", None)?;
    println!("Created and checked out: {}", dev_branch.name);

    // Make a commit on the new branch
    fs::write(test_path.join("feature.txt"), "New feature code\n").unwrap();
    repo.add(&["feature.txt"])?;
    repo.commit("Add new feature")?;
    println!("Made commit on develop branch");

    // Show current branch after checkout
    if let Some(current) = repo.current_branch()? {
        println!(
            "Now on branch: {} ({})",
            current.name,
            current.commit_hash.short()
        );
    }

    // Switch back to master branch
    let main_branch = branches.find("master").unwrap().clone();
    repo.checkout(&main_branch)?;
    println!("\nSwitched back to master branch");

    // List all branches with details
    let final_branches = repo.branches()?;
    println!("\n=== Final Branch List ===");
    println!("Total branches: {}", final_branches.len());
    println!("Local branches: {}", final_branches.local_count());

    for branch in final_branches.iter() {
        let marker = if branch.is_current { "*" } else { " " };
        let branch_type = if branch.is_local() { "local" } else { "remote" };
        println!(
            "  {}{} ({}) {}",
            marker,
            branch.name,
            branch_type,
            branch.commit_hash.short()
        );

        if let Some(upstream) = &branch.upstream {
            println!("    └── tracks: {}", upstream);
        }
    }

    // Demonstrate branch searching
    println!("\n=== Branch Search Examples ===");

    if let Some(branch) = final_branches.find("develop") {
        println!("Found branch by name: {}", branch.name);
    }

    if let Some(branch) = final_branches.find_by_short_name("new-api") {
        println!("Found branch by short name: {}", branch.name);
    }

    // Demonstrate branch filtering
    println!("\n=== Branch Filtering ===");

    println!("Local branches:");
    for branch in final_branches.local() {
        println!("  - {}", branch.name);
    }

    if final_branches.remote_count() > 0 {
        println!("Remote branches:");
        for branch in final_branches.remote() {
            println!("  - {}", branch.name);
        }
    }

    // Delete a branch (switch away first if it's current)
    println!("\n=== Branch Deletion ===");
    let bugfix = final_branches.find("bugfix/issue-123").unwrap().clone();
    repo.delete_branch(&bugfix, false)?;
    println!("Deleted branch: {}", bugfix.name);

    // Show final state
    let final_branches = repo.branches()?;
    println!("\nFinal branch count: {}", final_branches.len());

    // Clean up
    fs::remove_dir_all(&test_path).unwrap();
    println!("\nCleaned up test repository");

    Ok(())
}
