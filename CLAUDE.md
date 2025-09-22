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
- **Core types**: Hash (in src/types.rs), IndexStatus, WorktreeStatus, FileEntry (in src/commands/status.rs), Branch, BranchList, BranchType (in src/commands/branch.rs), Commit, CommitLog, Author, CommitMessage, CommitDetails, LogOptions (in src/commands/log.rs), RepoConfig (in src/commands/config.rs), Remote, RemoteList, FetchOptions, PushOptions (in src/commands/remote.rs), RestoreOptions, RemoveOptions, MoveOptions (in src/commands/files.rs), DiffOutput, FileDiff, DiffStatus, DiffOptions, DiffStats, DiffChunk, DiffLine, DiffLineType (in src/commands/diff.rs), Tag, TagList, TagType, TagOptions (in src/commands/tag.rs), Stash, StashList, StashOptions, StashApplyOptions (in src/commands/stash.rs)
- **Utility functions**: git(args, working_dir) -> Result<String>, git_raw(args, working_dir) -> Result<Output>
- **Remote management**: Full remote operations with network support
  - Repository::add_remote(name, url) -> Result<()> - add remote repository
  - Repository::remove_remote(name) -> Result<()> - remove remote
  - Repository::rename_remote(old_name, new_name) -> Result<()> - rename remote
  - Repository::list_remotes() -> Result<RemoteList> - list all remotes with URLs
  - Repository::get_remote_url(name) -> Result<String> - get remote URL
  - Repository::fetch(remote) -> Result<()> - fetch from remote repository
  - Repository::fetch_with_options(remote, options) -> Result<()> - fetch with FetchOptions
  - Repository::push(remote, branch) -> Result<()> - push to remote repository
  - Repository::push_with_options(remote, branch, options) -> Result<()> - push with PushOptions
  - Repository::clone(url, path) -> Result<Repository> - clone repository (static method)
  - Remote struct: name, fetch_url, push_url with proper URL handling
  - RemoteList: Vec<Remote> with search methods (find, iter, len, is_empty)
  - FetchOptions: prune, tags, all_remotes with builder pattern (with_prune, with_tags, with_all_remotes)
  - PushOptions: force, tags, set_upstream with builder pattern (with_force, with_tags, with_set_upstream)
- **File lifecycle operations**: Comprehensive file management with advanced options
  - Repository::checkout_file(path) -> Result<()> - restore file from HEAD
  - Repository::restore(paths, options) -> Result<()> - advanced restore with RestoreOptions
  - Repository::reset_file(path) -> Result<()> - unstage specific file
  - Repository::rm(paths) -> Result<()> - remove files from repository
  - Repository::rm_with_options(paths, options) -> Result<()> - remove with RemoveOptions
  - Repository::mv(source, destination) -> Result<()> - move/rename files
  - Repository::mv_with_options(source, dest, options) -> Result<()> - move with MoveOptions
  - Repository::ignore_add(patterns) -> Result<()> - add patterns to .gitignore
  - Repository::ignore_check(path) -> Result<bool> - check if file is ignored
  - Repository::ignore_list() -> Result<Vec<String>> - list current ignore patterns
  - RestoreOptions: with_source(), with_staged(), with_worktree() - builder for restore configuration
  - RemoveOptions: with_force(), with_recursive(), with_cached(), with_ignore_unmatch() - builder for remove configuration
  - MoveOptions: with_force(), with_verbose(), with_dry_run() - builder for move configuration
- **Diff operations**: Multi-level API for comprehensive change comparison
  - Repository::diff() -> Result<DiffOutput> - working directory vs index (unstaged changes)
  - Repository::diff_staged() -> Result<DiffOutput> - index vs HEAD (staged changes)
  - Repository::diff_head() -> Result<DiffOutput> - working directory vs HEAD (all changes)
  - Repository::diff_commits(from, to) -> Result<DiffOutput> - between specific commits
  - Repository::diff_with_options(options) -> Result<DiffOutput> - advanced diff with DiffOptions
  - DiffOutput: files, stats with immutable collections and comprehensive filtering
  - FileDiff: path, old_path, status, chunks, additions, deletions with change details
  - DiffStatus enum: Added, Modified, Deleted, Renamed, Copied with const char conversion
  - DiffOptions: context_lines, whitespace handling, path filtering, output formats (name-only, stat, numstat)
  - DiffStats: files_changed, insertions, deletions with aggregate statistics
  - Complete filtering: files_with_status(), iter(), is_empty(), len() for result analysis
- **Tag operations**: Complete tag management with type-safe API
  - Repository::tags() -> Result<TagList> - list all tags with comprehensive filtering
  - Repository::create_tag(name, target) -> Result<Tag> - create lightweight tag
  - Repository::create_tag_with_options(name, target, options) -> Result<Tag> - create tag with options
  - Repository::delete_tag(name) -> Result<()> - delete tag
  - Repository::show_tag(name) -> Result<Tag> - detailed tag information
  - Tag struct: name, hash, tag_type, message, tagger (may default), timestamp (may default)
  - TagType enum: Lightweight, Annotated
  - TagList: Box<[Tag]> with iterator methods (iter, lightweight, annotated), search (find, find_containing, for_commit), counting (len, lightweight_count, annotated_count)
  - TagOptions builder: annotated, force, message, sign with builder pattern (with_annotated, with_force, with_message, with_sign)
  - Uses unified Author struct from log module for tagger metadata
- **Stash operations**: Complete stash management with type-safe API
  - Repository::stash_list() -> Result<StashList> - list all stashes with comprehensive filtering
  - Repository::stash_save(message) -> Result<Stash> - create simple stash
  - Repository::stash_push(message, options) -> Result<Stash> - create stash with options
  - Repository::stash_apply(index, options) -> Result<()> - apply stash without removing it
  - Repository::stash_pop(index, options) -> Result<()> - apply and remove stash
  - Repository::stash_show(index) -> Result<String> - show stash contents
  - Repository::stash_drop(index) -> Result<()> - remove specific stash
  - Repository::stash_clear() -> Result<()> - remove all stashes
  - Stash struct: index, message, hash, branch, timestamp
  - StashList: Box<[Stash]> with iterator methods (iter), search (find_containing, for_branch), access (latest, get), counting (len, is_empty)
  - StashOptions builder: untracked, keep_index, patch, staged_only, paths with builder pattern (with_untracked, with_keep_index, with_patch, with_staged_only, with_paths)
  - StashApplyOptions builder: restore_index, quiet with builder pattern (with_index, with_quiet)
- **Command modules**: status.rs, add.rs, commit.rs, branch.rs, log.rs, config.rs, remote.rs, files.rs, diff.rs, tag.rs, stash.rs (in src/commands/)
- **Testing**: 161+ tests covering all functionality with comprehensive edge cases
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
- **config_operations.rs**: Repository configuration management - user setup, configuration values, repository-scoped settings
- **remote_operations.rs**: Complete remote management - add/remove/rename remotes, fetch/push operations with options, network operations, error handling
- **file_lifecycle_operations.rs**: Comprehensive file management - restore/reset/remove/move operations, .gitignore management, advanced file lifecycle workflows, staging area manipulation
- **diff_operations.rs**: Comprehensive diff operations showcase - unstaged/staged diffs, commit comparisons, advanced options (whitespace handling, path filtering), output formats (name-only, stat, numstat), and change analysis
- **tag_operations.rs**: Complete tag management - create/delete/list tags, lightweight vs annotated tags, TagOptions builder, tag filtering and search, comprehensive tag workflows
- **stash_operations.rs**: Complete stash management - save/apply/pop/list stashes, advanced options (untracked files, keep index, specific paths), stash filtering and search, comprehensive stash workflows
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
