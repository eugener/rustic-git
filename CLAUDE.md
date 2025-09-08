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
- Available methods: Repository::init(path, bare), Repository::open(path), Repository::status(), Repository::add(paths), Repository::add_all(), Repository::add_update(), Repository::commit(message), Repository::commit_with_author(message, author)
- Status functionality: GitStatus with FileStatus enum, files as Box<[(FileStatus, String)]>
- Add functionality: Stage specific files, all changes, or tracked file updates
- Commit functionality: Create commits and return Hash of created commit
- Hash type: Universal git object hash representation with short() and Display methods
- Utility functions: git(args, working_dir) -> Result<String>, git_raw(args, working_dir) -> Result<Output>
- Command modules: status.rs, add.rs, commit.rs (in src/commands/)
- Core types: Hash (in src/types.rs)
- Run `cargo fmt && cargo build && cargo test && cargo clippy --all-targets --all-features -- -D warnings` after code changes
- Make sure all examples are running

## Examples
The `examples/` directory contains comprehensive demonstrations of library functionality:

- **basic_usage.rs**: Complete workflow from init to commit - demonstrates fundamental rustic-git usage
- **repository_operations.rs**: Repository lifecycle - init regular/bare repos, open existing repos, error handling
- **status_checking.rs**: GitStatus and FileStatus usage - all status query methods and file state filtering
- **staging_operations.rs**: Staging operations - add(), add_all(), add_update() with before/after comparisons
- **commit_workflows.rs**: Commit operations and Hash type - commit(), commit_with_author(), Hash methods
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
