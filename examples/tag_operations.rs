//! Tag Operations Example
//!
//! This example demonstrates comprehensive tag management functionality in rustic-git:
//! - Creating lightweight and annotated tags
//! - Listing and filtering tags
//! - Deleting tags
//! - Tag options and configuration
//! - Working with tag metadata
//!
//! Run with: cargo run --example tag_operations

use rustic_git::{Repository, Result, TagOptions, TagType};
use std::env;
use std::fs;

fn main() -> Result<()> {
    println!("Rustic Git - Tag Operations Example\n");

    // Use a temporary directory for this example
    let repo_path = env::temp_dir().join("rustic_git_tag_example");

    // Clean up any previous run
    if repo_path.exists() {
        fs::remove_dir_all(&repo_path).expect("Failed to clean up previous example");
    }

    println!("Initializing repository at: {}", repo_path.display());

    // Initialize repository and configure user
    let repo = Repository::init(&repo_path, false)?;
    repo.config()
        .set_user("Tag Demo User", "tags@example.com")?;

    // Create some commits to tag
    println!("\nCreating initial commits...");

    // First commit
    fs::write(
        repo_path.join("README.md"),
        "# Tag Demo Project\n\nDemonstrating Git tag operations.\n",
    )?;
    repo.add(&["README.md"])?;
    let first_commit_hash = repo.commit("Initial commit: Add README")?;
    println!("Created commit: {}", first_commit_hash.short());

    // Second commit
    fs::create_dir_all(repo_path.join("src"))?;
    fs::write(
        repo_path.join("src/main.rs"),
        "fn main() {\n    println!(\"Hello, tags!\");\n}\n",
    )?;
    repo.add(&["src/main.rs"])?;
    let second_commit_hash = repo.commit("Add main.rs with hello world")?;
    println!("Created commit: {}", second_commit_hash.short());

    // Third commit
    fs::write(
        repo_path.join("src/lib.rs"),
        "//! Tag demo library\n\npub fn greet(name: &str) -> String {\n    format!(\"Hello, {}!\", name)\n}\n",
    )?;
    repo.add(&["src/lib.rs"])?;
    let third_commit_hash = repo.commit("Add library with greet function")?;
    println!("Created commit: {}", third_commit_hash.short());

    // Demonstrate tag creation
    println!("\n=== Creating Tags ===");

    // Create lightweight tags
    println!("\n1. Creating lightweight tags:");

    let v0_1_0 = repo.create_tag("v0.1.0", Some(&first_commit_hash))?;
    println!(
        "Created lightweight tag: {} -> {} ({})",
        v0_1_0.name,
        v0_1_0.hash.short(),
        v0_1_0.tag_type
    );

    let v0_2_0 = repo.create_tag("v0.2.0", Some(&second_commit_hash))?;
    println!(
        "Created lightweight tag: {} -> {} ({})",
        v0_2_0.name,
        v0_2_0.hash.short(),
        v0_2_0.tag_type
    );

    // Create annotated tags
    println!("\n2. Creating annotated tags:");

    let options =
        TagOptions::new().with_message("First stable release with basic functionality".to_string());
    let v1_0_0 = repo.create_tag_with_options("v1.0.0", Some(&third_commit_hash), options)?;
    println!(
        "Created annotated tag: {} -> {} ({})",
        v1_0_0.name,
        v1_0_0.hash.short(),
        v1_0_0.tag_type
    );
    if let Some(message) = &v1_0_0.message {
        println!("  Message: {}", message);
    }

    // Tag current HEAD
    let latest_options = TagOptions::new().with_message("Latest development version".to_string());
    let latest_tag = repo.create_tag_with_options("latest", None, latest_options)?;
    println!(
        "Created annotated tag on HEAD: {} -> {} ({})",
        latest_tag.name,
        latest_tag.hash.short(),
        latest_tag.tag_type
    );

    // Create some feature tags
    println!("\n3. Creating feature and release candidate tags:");

    let feature_options = TagOptions::new().with_message("Feature branch snapshot".to_string());
    repo.create_tag_with_options("feature/demo", None, feature_options)?;

    let rc_options = TagOptions::new().with_message("Release candidate for v1.1.0".to_string());
    repo.create_tag_with_options("v1.1.0-rc1", None, rc_options)?;

    // Create a couple more version tags
    repo.create_tag("v0.3.0", None)?;
    repo.create_tag("v0.9.0", None)?;

    // Demonstrate tag listing and filtering
    println!("\n=== Tag Listing and Filtering ===");

    let tags = repo.tags()?;
    println!("\nAll tags ({} total):", tags.len());
    for tag in tags.iter() {
        let type_marker = match tag.tag_type {
            TagType::Lightweight => "L",
            TagType::Annotated => "A",
        };
        println!("  [{}] {} -> {}", type_marker, tag.name, tag.hash.short());
        if let Some(message) = &tag.message {
            println!("      Message: {}", message.lines().next().unwrap_or(""));
        }
    }

    // Filter by type
    println!("\nLightweight tags ({} total):", tags.lightweight_count());
    for tag in tags.lightweight() {
        println!("  {} -> {}", tag.name, tag.hash.short());
    }

    println!("\nAnnotated tags ({} total):", tags.annotated_count());
    for tag in tags.annotated() {
        println!("  {} -> {}", tag.name, tag.hash.short());
        if let Some(message) = &tag.message {
            println!("    Message: {}", message.lines().next().unwrap_or(""));
        }
    }

    // Search and filtering
    println!("\n=== Tag Searching ===");

    // Find specific tag
    if let Some(tag) = tags.find("v1.0.0") {
        println!("\nFound tag 'v1.0.0':");
        println!("  Type: {}", tag.tag_type);
        println!("  Hash: {}", tag.hash.short());
        if let Some(message) = &tag.message {
            println!("  Message: {}", message);
        }
    }

    // Find version tags
    let version_tags: Vec<_> = tags.find_containing("v").collect();
    println!(
        "\nVersion tags (containing 'v'): {} found",
        version_tags.len()
    );
    for tag in &version_tags {
        println!("  {}", tag.name);
    }

    // Find release candidates
    let rc_tags: Vec<_> = tags.find_containing("rc").collect();
    println!("\nRelease candidate tags: {} found", rc_tags.len());
    for tag in &rc_tags {
        println!("  {}", tag.name);
    }

    // Find tags for specific commit
    let tags_for_third_commit: Vec<_> = tags.for_commit(&third_commit_hash).collect();
    println!(
        "\nTags pointing to commit {}: {} found",
        third_commit_hash.short(),
        tags_for_third_commit.len()
    );
    for tag in &tags_for_third_commit {
        println!("  {}", tag.name);
    }

    // Demonstrate tag details
    println!("\n=== Tag Details ===");

    let detailed_tag = repo.show_tag("v1.0.0")?;
    println!("\nDetailed information for 'v1.0.0':");
    println!("  Name: {}", detailed_tag.name);
    println!("  Type: {}", detailed_tag.tag_type);
    println!("  Commit: {}", detailed_tag.hash);
    println!("  Short hash: {}", detailed_tag.hash.short());

    if let Some(message) = &detailed_tag.message {
        println!("  Message: {}", message);
    }

    if let Some(tagger) = &detailed_tag.tagger {
        println!("  Tagger: {}", tagger);
        println!(
            "  Tagged at: {}",
            tagger.timestamp.format("%Y-%m-%d %H:%M:%S UTC")
        );
    }

    if let Some(timestamp) = &detailed_tag.timestamp {
        println!("  Timestamp: {}", timestamp.format("%Y-%m-%d %H:%M:%S UTC"));
    }

    // Demonstrate tag operations
    println!("\n=== Tag Operations ===");

    // Create and force overwrite a tag
    println!("\n1. Testing tag overwrite:");

    // This should fail (tag already exists)
    match repo.create_tag("latest", None) {
        Ok(_) => println!("  ERROR: Should have failed to create existing tag"),
        Err(e) => println!("  Expected error creating existing tag: {}", e),
    }

    // Force overwrite
    let force_options = TagOptions::new()
        .with_force()
        .with_message("Forcefully updated latest tag".to_string());

    match repo.create_tag_with_options("latest", None, force_options) {
        Ok(tag) => println!("  Successfully force-created tag: {}", tag.name),
        Err(e) => println!("  Error force-creating tag: {}", e),
    }

    // Tag deletion
    println!("\n2. Testing tag deletion:");

    // Create a temporary tag to delete
    repo.create_tag("temp-tag", None)?;
    println!("  Created temporary tag: temp-tag");

    // Verify it exists
    let tags_before = repo.tags()?;
    let temp_exists_before = tags_before.find("temp-tag").is_some();
    println!("  Temp tag exists before deletion: {}", temp_exists_before);

    // Delete it
    repo.delete_tag("temp-tag")?;
    println!("  Deleted temp-tag");

    // Verify it's gone
    let tags_after = repo.tags()?;
    let temp_exists_after = tags_after.find("temp-tag").is_some();
    println!("  Temp tag exists after deletion: {}", temp_exists_after);

    // Summary
    println!("\n=== Summary ===");
    let final_tags = repo.tags()?;
    println!("\nFinal repository state:");
    println!("  Total tags: {}", final_tags.len());
    println!("  Lightweight tags: {}", final_tags.lightweight_count());
    println!("  Annotated tags: {}", final_tags.annotated_count());

    println!("\nTag creation options demonstrated:");
    println!("  ✓ Lightweight tags (simple references)");
    println!("  ✓ Annotated tags (with messages and metadata)");
    println!("  ✓ Tags on specific commits");
    println!("  ✓ Tags on current HEAD");
    println!("  ✓ Force tag creation/overwrite");

    println!("\nTag listing and filtering demonstrated:");
    println!("  ✓ List all tags");
    println!("  ✓ Filter by tag type (lightweight/annotated)");
    println!("  ✓ Search by name patterns");
    println!("  ✓ Find tags by commit hash");
    println!("  ✓ Show detailed tag information");

    println!("\nTag management demonstrated:");
    println!("  ✓ Tag creation with options");
    println!("  ✓ Tag deletion");
    println!("  ✓ Error handling for duplicate tags");

    // Clean up
    println!("\nCleaning up example repository...");
    fs::remove_dir_all(&repo_path)?;
    println!("Tag operations example completed successfully!");

    Ok(())
}
