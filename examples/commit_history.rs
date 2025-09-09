use chrono::{Duration, Utc};
use rustic_git::{LogOptions, Repository, Result};
use std::fs;
use std::path::Path;

fn main() -> Result<()> {
    let test_path = "/tmp/rustic_git_commit_history_example";

    // Clean up if exists
    if Path::new(test_path).exists() {
        fs::remove_dir_all(test_path).unwrap();
    }

    // Create a test repository
    let repo = Repository::init(test_path, false)?;
    println!("Created repository at: {}", test_path);

    // Create several commits to build history
    println!("\n=== Building Commit History ===");

    // First commit
    fs::write(
        format!("{}/README.md", test_path),
        "# Commit History Demo\n\nA demonstration of rustic-git log functionality.",
    )
    .unwrap();
    repo.add(&["README.md"])?;
    let commit1 = repo.commit("Initial commit - add README")?;
    println!("Created commit 1: {} - Initial commit", commit1.short());

    // Second commit
    fs::create_dir_all(format!("{}/src", test_path)).unwrap();
    fs::write(
        format!("{}/src/main.rs", test_path),
        "fn main() {\n    println!(\"Hello, world!\");\n}",
    )
    .unwrap();
    repo.add(&["src/main.rs"])?;
    let commit2 = repo.commit("Add main.rs with Hello World")?;
    println!("Created commit 2: {} - Add main.rs", commit2.short());

    // Third commit
    fs::write(
        format!("{}/src/lib.rs", test_path),
        "pub fn greet(name: &str) -> String {\n    format!(\"Hello, {}!\", name)\n}",
    )
    .unwrap();
    repo.add(&["src/lib.rs"])?;
    let commit3 = repo.commit("Add library module with greet function")?;
    println!("Created commit 3: {} - Add lib.rs", commit3.short());

    // Fourth commit
    fs::write(
        format!("{}/Cargo.toml", test_path),
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"",
    )
    .unwrap();
    repo.add(&["Cargo.toml"])?;
    let commit4 = repo.commit("Add Cargo.toml configuration")?;
    println!("Created commit 4: {} - Add Cargo.toml", commit4.short());

    // Fifth commit - bug fix
    fs::write(
        format!("{}/src/main.rs", test_path),
        "fn main() {\n    println!(\"Hello, rustic-git!\");\n}",
    )
    .unwrap();
    repo.add(&["src/main.rs"])?;
    let commit5 = repo.commit("Fix greeting message in main")?;
    println!("Created commit 5: {} - Fix greeting", commit5.short());

    // Sixth commit - documentation
    fs::write(format!("{}/README.md", test_path), "# Commit History Demo\n\nA demonstration of rustic-git log functionality.\n\n## Features\n\n- Greeting functionality\n- Command line interface\n").unwrap();
    repo.add(&["README.md"])?;
    let commit6 = repo.commit("Update README with features section")?;
    println!("Created commit 6: {} - Update README", commit6.short());

    println!("Built commit history with 6 commits");

    // Basic log operations
    println!("\n=== Basic Log Operations ===");

    let all_commits = repo.log()?;
    println!("Total commits in repository: {}", all_commits.len());

    println!("\nAll commits (most recent first):");
    for (i, commit) in all_commits.iter().enumerate() {
        println!("  {}. {}", i + 1, commit);
    }

    // Recent commits
    println!("\n=== Recent Commits ===");
    let recent = repo.recent_commits(3)?;
    println!("Last 3 commits:");
    for commit in recent.iter() {
        println!("  {} - {}", commit.hash.short(), commit.message.subject);
        if let Some(body) = &commit.message.body {
            println!("    Body: {}", body);
        }
    }

    // Advanced filtering with LogOptions
    println!("\n=== Advanced Filtering ===");

    // Filter by message content
    let fix_commits = all_commits.with_message_containing("fix");
    println!("Commits with 'fix' in message:");
    for commit in fix_commits {
        println!("  {} - {}", commit.hash.short(), commit.message.subject);
    }

    // Filter by date (recent commits)
    let now = Utc::now();
    let recent_commits = all_commits.since(now - Duration::minutes(5));
    println!("\nCommits from last 5 minutes: {}", recent_commits.count());

    // Using LogOptions for advanced queries
    println!("\n=== LogOptions Advanced Queries ===");

    // Get commits with grep
    let opts = LogOptions::new().max_count(10).grep("README".to_string());
    let readme_commits = repo.log_with_options(&opts)?;
    println!("Commits mentioning 'README': {}", readme_commits.len());
    for commit in readme_commits.iter() {
        println!("  {} - {}", commit.hash.short(), commit.message.subject);
    }

    // Get commits affecting specific paths
    println!("\n=== Path-Specific History ===");
    let src_commits = repo.log_for_paths(&["src/"])?;
    println!("Commits affecting src/ directory: {}", src_commits.len());
    for commit in src_commits.iter() {
        println!("  {} - {}", commit.hash.short(), commit.message.subject);
    }

    // Show detailed commit information
    println!("\n=== Detailed Commit Information ===");

    let commit_details = repo.show_commit(&commit3)?;
    println!("Detailed info for commit {}:", commit3.short());
    println!("  Author: {}", commit_details.commit.author);
    println!("  Committer: {}", commit_details.commit.committer);
    println!(
        "  Timestamp: {}",
        commit_details
            .commit
            .timestamp
            .format("%Y-%m-%d %H:%M:%S UTC")
    );
    println!("  Message: {}", commit_details.commit.message.subject);
    println!("  Parents: {}", commit_details.commit.parents.len());
    for parent in commit_details.commit.parents.iter() {
        println!("    - {}", parent.short());
    }
    println!("  Files changed: {}", commit_details.files_changed.len());
    for file in &commit_details.files_changed {
        println!("    - {}", file.display());
    }
    println!(
        "  Changes: +{} -{}",
        commit_details.insertions, commit_details.deletions
    );

    // Commit analysis
    println!("\n=== Commit Analysis ===");

    let merge_commits: Vec<_> = all_commits.merges_only().collect();
    let regular_commits: Vec<_> = all_commits.no_merges().collect();

    println!("Repository statistics:");
    println!("  Total commits: {}", all_commits.len());
    println!("  Merge commits: {}", merge_commits.len());
    println!("  Regular commits: {}", regular_commits.len());

    if let Some(first_commit) = all_commits.first() {
        println!(
            "  Most recent: {} ({})",
            first_commit.hash.short(),
            first_commit.message.subject
        );
    }

    if let Some(last_commit) = all_commits.last() {
        println!(
            "  Oldest: {} ({})",
            last_commit.hash.short(),
            last_commit.message.subject
        );
    }

    // Search operations
    println!("\n=== Search Operations ===");

    // Find by hash
    if let Some(found) = all_commits.find_by_hash(&commit2) {
        println!("Found commit by full hash: {}", found.message.subject);
    }

    // Find by short hash
    if let Some(found) = all_commits.find_by_short_hash(commit4.short()) {
        println!("Found commit by short hash: {}", found.message.subject);
    }

    // Commit range operations
    println!("\n=== Commit Range Operations ===");

    let range_commits = repo.log_range(&commit2, &commit5)?;
    println!(
        "Commits in range {}..{}: {}",
        commit2.short(),
        commit5.short(),
        range_commits.len()
    );
    for commit in range_commits.iter() {
        println!("  {} - {}", commit.hash.short(), commit.message.subject);
    }

    // Advanced LogOptions demonstration
    println!("\n=== Advanced LogOptions Usage ===");

    let advanced_opts = LogOptions::new()
        .max_count(5)
        .no_merges(true)
        .paths(vec!["src/main.rs".into()]);

    let filtered_commits = repo.log_with_options(&advanced_opts)?;
    println!(
        "Non-merge commits affecting src/main.rs (max 5): {}",
        filtered_commits.len()
    );
    for commit in filtered_commits.iter() {
        println!("  {} - {}", commit.hash.short(), commit.message.subject);
    }

    // Commit message analysis
    println!("\n=== Commit Message Analysis ===");

    let total_commits = all_commits.len();
    let commits_with_body: Vec<_> = all_commits
        .iter()
        .filter(|c| c.message.body.is_some())
        .collect();

    println!("Message statistics:");
    println!("  Total commits: {}", total_commits);
    println!("  Commits with body text: {}", commits_with_body.len());
    println!(
        "  Commits with subject only: {}",
        total_commits - commits_with_body.len()
    );

    // Display commit types by analyzing subjects
    let fix_count = all_commits
        .iter()
        .filter(|c| c.message.subject.to_lowercase().contains("fix"))
        .count();
    let add_count = all_commits
        .iter()
        .filter(|c| c.message.subject.to_lowercase().contains("add"))
        .count();
    let update_count = all_commits
        .iter()
        .filter(|c| c.message.subject.to_lowercase().contains("update"))
        .count();

    println!("  Commit types:");
    println!("    - Fix commits: {}", fix_count);
    println!("    - Add commits: {}", add_count);
    println!("    - Update commits: {}", update_count);
    println!(
        "    - Other commits: {}",
        total_commits - fix_count - add_count - update_count
    );

    // Timeline view
    println!("\n=== Timeline View ===");

    println!("Commit timeline (oldest to newest):");
    let commits: Vec<_> = all_commits.iter().collect();
    for commit in commits.iter().rev() {
        let commit_type = if commit.is_merge() { "MERGE" } else { "COMMIT" };
        println!(
            "  {} {} {} - {}",
            commit.timestamp.format("%H:%M:%S"),
            commit_type,
            commit.hash.short(),
            commit.message.subject
        );
    }

    // Summary
    println!("\n=== Summary ===");

    println!("Commit history demonstration completed!");
    println!("  Repository: {}", test_path);
    println!("  Total commits analyzed: {}", all_commits.len());
    println!("  Hash examples:");
    for commit in all_commits.iter().take(3) {
        println!("    - Full: {}", commit.hash.as_str());
        println!("      Short: {}", commit.hash.short());
    }

    // Clean up
    fs::remove_dir_all(test_path).unwrap();
    println!("\nCleaned up test repository");

    Ok(())
}
