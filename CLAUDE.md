- We are using Rust edition of 2024
- Follow the Rust style guide for naming conventions and formatting
- Implement best practices for code organization and maintainability

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
- Run `cargo build && cargo test` after code changes
