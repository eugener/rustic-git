# Rustic Git - Development Guidelines

## Code Standards
- We are using Rust edition of 2024
- Follow the Rust style guide for naming conventions and formatting
- Implement best practices for code organization and maintainability
- Do not use emoji while coding or in commit messages
- Follow conventional commit format: type(scope): description
- Use commit types: feat, fix, docs, style, refactor, test, chore

## Design Choices
- **Repository-centric API**: Static lifecycle methods (init, open) return Repository instances, instance methods for git operations
- **Module-based organization**: Separate files for repository.rs, error.rs, with lib.rs for re-exports only
- **Co-located unit tests**: Tests within each module (#[cfg(test)] mod tests) rather than separate test files
- **Early validation**: Always call Repository::ensure_git() before git operations to validate git availability
- **Path handling**: Use PathBuf for internal storage, &Path for method parameters and returns, impl AsRef<Path> for flexibility
- **Error handling**: Custom GitError enum with From<io::Error> trait for ergonomic error propagation
- **Command execution**: Use std::process::Command with proper error handling and stderr capture

## Implementation
- **Repository lifecycle**: Repository::init(path, bare), Repository::open(path)
- **Status functionality**: Enhanced GitStatus API with separate staged/unstaged file tracking
  - GitStatus with entries as Box<[FileEntry]> for immutable, efficient storage
  - FileEntry contains PathBuf, IndexStatus, and WorktreeStatus for precise Git state representation
  - IndexStatus enum: Clean, Modified, Added, Deleted, Renamed, Copied (with const from_char/to_char methods)
  - WorktreeStatus enum: Clean, Modified, Deleted, Untracked, Ignored (with const from_char/to_char methods)
  - API methods: staged_files(), unstaged_files(), untracked_entries(), files_with_index_status(), files_with_worktree_status()
- **Staging functionality**: Repository::add(paths), Repository::add_all(), Repository::add_update()
- **Commit functionality**: Repository::commit(message), Repository::commit_with_author(message, author) - return Hash of created commit
- **Configuration functionality**: Repository::config() -> RepoConfig - manage git configuration values
  - RepoConfig::set_user(name, email) -> Result<()> - convenience method for user.name and user.email
  - RepoConfig::get_user() -> Result<(String, String)> - get user configuration as tuple
  - RepoConfig::set(key, value) -> Result<()> - set any git configuration value
  - RepoConfig::get(key) -> Result<String> - get any git configuration value  
  - RepoConfig::unset(key) -> Result<()> - remove git configuration value
- **Branch functionality**: Complete branch operations with type-safe API
  - Repository::branches() -> Result<BranchList> - list all branches with comprehensive filtering
  - Repository::current_branch() -> Result<Option<Branch>> - get currently checked out branch
  - Repository::create_branch(name, start_point) -> Result<Branch> - create new branch
  - Repository::delete_branch(branch, force) -> Result<()> - delete branch with safety checks
  - Repository::checkout(branch) -> Result<()> - switch to existing branch
  - Repository::checkout_new(name, start_point) -> Result<Branch> - create and checkout branch
  - Branch struct: name, branch_type, is_current, commit_hash, upstream tracking
  - BranchType enum: Local, RemoteTracking
  - BranchList: Box<[Branch]> with iterator methods (iter, local, remote), search (find, find_by_short_name), counting (len, local_count, remote_count)
- **Commit history & log operations**: Multi-level API for comprehensive commit analysis
  - Repository::log() -> Result<CommitLog> - get all commits with simple API
  - Repository::recent_commits(count) -> Result<CommitLog> - get recent N commits
  - Repository::log_with_options(options) -> Result<LogOptions> - advanced queries with filters
  - Repository::log_range(from, to) -> Result<CommitLog> - commits between two points
  - Repository::log_for_paths(paths) -> Result<CommitLog> - commits affecting specific paths
  - Repository::show_commit(hash) -> Result<CommitDetails> - detailed commit information
  - Commit struct: hash, author, committer, message, timestamp, parents
  - CommitLog: Box<[Commit]> with iterator-based filtering (with_message_containing, since, until, merges_only, no_merges, find_by_hash)
  - LogOptions builder: max_count, since/until dates, author/committer filters, grep, paths, merge filtering
  - Author struct: name, email, timestamp with Display implementation
  - CommitMessage: subject and optional body parsing
  - CommitDetails: full commit info including file changes and diff stats
- **Core types**: Hash (in src/types.rs), IndexStatus, WorktreeStatus, FileEntry (in src/commands/status.rs), Branch, BranchList, BranchType (in src/commands/branch.rs), Commit, CommitLog, Author, CommitMessage, CommitDetails, LogOptions (in src/commands/log.rs), RepoConfig (in src/commands/config.rs)
- **Utility functions**: git(args, working_dir) -> Result<String>, git_raw(args, working_dir) -> Result<Output>
- **Command modules**: status.rs, add.rs, commit.rs, branch.rs, log.rs, config.rs (in src/commands/)
- **Testing**: 106+ tests covering all functionality with comprehensive edge cases
- Run `cargo fmt && cargo build && cargo test && cargo clippy --all-targets --all-features -- -D warnings` after code changes
- Make sure all examples are running

## Examples
The `examples/` directory contains comprehensive demonstrations of library functionality:

- **basic_usage.rs**: Complete workflow from init to commit - demonstrates fundamental rustic-git usage
- **repository_operations.rs**: Repository lifecycle - init regular/bare repos, open existing repos, error handling
- **status_checking.rs**: Enhanced GitStatus API usage - staged/unstaged file queries, IndexStatus/WorktreeStatus filtering, comprehensive status analysis
- **staging_operations.rs**: Staging operations - add(), add_all(), add_update() with before/after comparisons
- **commit_workflows.rs**: Commit operations and Hash type - commit(), commit_with_author(), Hash methods
- **branch_operations.rs**: Complete branch management - create/delete/checkout branches, BranchList filtering, branch type handling, search operations
- **commit_history.rs**: Comprehensive commit history & log operations - demonstrates all commit querying APIs, filtering, analysis, and advanced LogOptions usage
- **error_handling.rs**: Comprehensive error handling patterns - GitError variants, recovery strategies

Run examples with: `cargo run --example <example_name>`
All examples use temporary directories and include cleanup for safe execution.

## Contributing Guidelines

### Development Workflow
Before any code changes, ensure you follow this workflow:

1. **Format code**: `cargo fmt`
2. **Build project**: `cargo build` 
3. **Run all tests**: `cargo test`
4. **Run linting**: `cargo clippy --all-targets --all-features -- -D warnings` (no warnings allowed)
5. **Verify examples**: Make sure all examples run successfully

### Pull Request Requirements
1. All tests must pass and examples must run successfully
2. Code must be properly formatted and pass clippy without warnings
3. Follow the project's design principles and architecture patterns
4. Use conventional commit messages with appropriate types
5. Keep commit messages concise and in present tense
