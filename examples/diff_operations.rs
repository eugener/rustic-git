use rustic_git::{DiffOptions, DiffStatus, Repository};
use std::{env, fs};

fn main() -> rustic_git::Result<()> {
    println!("Rustic Git - Diff Operations Example\n");

    let repo_path = env::temp_dir().join("rustic_git_diff_example");
    // Clean up any previous run
    if repo_path.exists() {
        fs::remove_dir_all(&repo_path).ok();
    }
    println!("Working in temporary directory: {}", repo_path.display());

    // Initialize repository
    let repo = Repository::init(&repo_path, false)?;
    println!("Repository initialized successfully\n");

    // Configure git user for commits
    let config = repo.config();
    config.set_user("Test User", "test@example.com")?;

    println!("=== Creating Initial Files ===");

    // Create initial files
    let readme_path = repo_path.join("README.md");
    let src_dir = repo_path.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    let main_path = src_dir.join("main.rs");
    let lib_path = src_dir.join("lib.rs");

    fs::write(
        &readme_path,
        "# Test Project\n\nA sample project for testing diff operations.\n",
    )
    .unwrap();
    fs::write(
        &main_path,
        "fn main() {\n    println!(\"Hello, world!\");\n}\n",
    )
    .unwrap();
    fs::write(
        &lib_path,
        "pub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n",
    )
    .unwrap();

    println!("Created initial files: README.md, src/main.rs, src/lib.rs");

    // Stage and commit initial files
    repo.add_all()?;
    let initial_commit = repo.commit("feat: initial commit with basic files")?;
    println!("Initial commit: {}\n", initial_commit.short());

    println!("=== Testing Different Diff Operations ===");

    // Test 1: Diff with no changes (should be empty)
    println!("1. Diff with no changes:");
    let diff = repo.diff()?;
    if diff.is_empty() {
        println!("   ✓ No changes detected (as expected)");
    } else {
        println!("   ✗ Unexpected changes found");
    }
    println!();

    // Test 2: Modify files and show unstaged changes
    println!("2. Creating unstaged changes:");
    fs::write(&readme_path, "# Test Project\n\nA sample project for testing diff operations.\n\n## Features\n- Git operations\n- Diff functionality\n").unwrap();
    fs::write(&main_path, "fn main() {\n    println!(\"Hello, world!\");\n    println!(\"Testing diff operations!\");\n}\n").unwrap();

    let diff = repo.diff()?;
    println!("   Unstaged changes found:");
    println!("   Files changed: {}", diff.len());
    for file in diff.iter() {
        println!("   - {} ({})", file.path.display(), file.status);
    }
    println!("   {}", diff.stats);
    println!();

    // Test 3: Stage some changes and show staged vs unstaged
    println!("3. Staging README.md and checking staged diff:");
    repo.add(&[&readme_path])?;

    let staged_diff = repo.diff_staged()?;
    println!("   Staged changes:");
    for file in staged_diff.iter() {
        println!("   - {} ({})", file.path.display(), file.status);
    }
    println!("   {}", staged_diff.stats);

    let unstaged_diff = repo.diff()?;
    println!("   Remaining unstaged changes:");
    for file in unstaged_diff.iter() {
        println!("   - {} ({})", file.path.display(), file.status);
    }
    println!("   {}", unstaged_diff.stats);
    println!();

    // Test 4: Diff with options
    println!("4. Using diff options (name-only):");
    let name_only_diff = repo.diff_with_options(&DiffOptions::new().name_only())?;
    println!("   Modified files (name-only):");
    for file in name_only_diff.iter() {
        println!("   - {}", file.path.display());
    }
    println!();

    // Test 5: Diff with file filtering
    println!("5. Diff with path filtering (src/ only):");
    let src_paths = vec![src_dir.clone()];
    let filtered_diff = repo.diff_with_options(&DiffOptions::new().paths(src_paths))?;
    println!("   Changes in src/ directory:");
    for file in filtered_diff.iter() {
        println!("   - {} ({})", file.path.display(), file.status);
    }
    println!();

    // Stage remaining changes and commit
    repo.add_all()?;
    let second_commit = repo.commit("feat: add features section and improve main function")?;
    println!("Second commit: {}", second_commit.short());

    // Test 6: Diff between commits
    println!("\n6. Diff between commits:");
    let commit_diff = repo.diff_commits(&initial_commit, &second_commit)?;
    println!(
        "   Changes from {} to {}:",
        initial_commit.short(),
        second_commit.short()
    );
    for file in commit_diff.iter() {
        println!(
            "   - {} ({}) +{} -{}",
            file.path.display(),
            file.status,
            file.additions,
            file.deletions
        );
    }
    println!("   {}", commit_diff.stats);
    println!();

    // Test 7: Add a new file and show it in diff
    println!("7. Adding new file and checking diff:");
    let test_path = repo_path.join("test.txt");
    fs::write(
        &test_path,
        "This is a new test file.\nWith multiple lines.\n",
    )
    .unwrap();

    let new_file_diff = repo.diff()?;
    println!("   New file detected:");
    for file in new_file_diff.iter() {
        println!("   - {} ({})", file.path.display(), file.status);
    }
    println!();

    // Test 8: Delete a file and show in diff
    println!("8. Deleting file and checking diff:");
    fs::remove_file(&lib_path).unwrap();

    let deleted_file_diff = repo.diff()?;
    println!("   Changes after file deletion:");
    for file in deleted_file_diff.iter() {
        println!("   - {} ({})", file.path.display(), file.status);
    }
    println!();

    // Test 9: Diff with ignore whitespace options
    println!("9. Testing whitespace options:");

    // Add some whitespace changes
    fs::write(&main_path, "fn main() {\n    println!(\"Hello, world!\");\n    println!(\"Testing diff operations!\");    \n}\n").unwrap();

    let normal_diff = repo.diff()?;
    let whitespace_diff = repo.diff_with_options(&DiffOptions::new().ignore_whitespace())?;

    println!("   Normal diff shows {} files changed", normal_diff.len());
    println!(
        "   Whitespace-ignoring diff shows {} files changed",
        whitespace_diff.len()
    );
    println!();

    // Test 10: Show diff with HEAD
    println!("10. Diff with HEAD (all changes since last commit):");
    let head_diff = repo.diff_head()?;
    println!("    All changes since last commit:");
    for file in head_diff.iter() {
        println!("    - {} ({})", file.path.display(), file.status);
    }
    println!("    {}", head_diff.stats);
    println!();

    // Test 11: Different diff output formats
    println!("11. Testing different output formats:");

    let stat_diff = repo.diff_with_options(&DiffOptions::new().stat_only())?;
    println!("    Stat format:");
    println!("    {}", stat_diff);

    let numstat_diff = repo.diff_with_options(&DiffOptions::new().numstat())?;
    println!("    Numstat format - {} files changed", numstat_diff.len());
    for file in numstat_diff.iter() {
        println!(
            "    {} +{} -{}",
            file.path.display(),
            file.additions,
            file.deletions
        );
    }
    println!();

    // Test 12: Filtering by file status
    println!("12. Filtering files by status:");
    let all_changes = repo.diff_head()?;

    let added_files: Vec<_> = all_changes.files_with_status(DiffStatus::Added).collect();
    let modified_files: Vec<_> = all_changes
        .files_with_status(DiffStatus::Modified)
        .collect();
    let deleted_files: Vec<_> = all_changes.files_with_status(DiffStatus::Deleted).collect();

    println!("    Added files: {}", added_files.len());
    for file in added_files {
        println!("      - {}", file.path.display());
    }

    println!("    Modified files: {}", modified_files.len());
    for file in modified_files {
        println!("      - {}", file.path.display());
    }

    println!("    Deleted files: {}", deleted_files.len());
    for file in deleted_files {
        println!("      - {}", file.path.display());
    }
    println!();

    println!("=== Diff Operations Demo Complete ===");
    println!("All diff operations completed successfully!");
    println!("Summary of tested features:");
    println!("✓ Basic diff operations (working dir vs index)");
    println!("✓ Staged diff operations (index vs HEAD)");
    println!("✓ Diff between specific commits");
    println!("✓ Diff with various options (name-only, stat, numstat)");
    println!("✓ Path filtering");
    println!("✓ Whitespace handling options");
    println!("✓ File status filtering");
    println!("✓ Comprehensive diff statistics");

    println!("\nCleaning up temporary repository...");
    fs::remove_dir_all(&repo_path).ok();

    Ok(())
}
