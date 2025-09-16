use crate::utils::git;
use crate::{Repository, Result};

/// Repository configuration manager
///
/// Provides methods for getting and setting git configuration values
/// for a specific repository.
pub struct RepoConfig<'a> {
    repo: &'a Repository,
}

impl<'a> RepoConfig<'a> {
    /// Create a new RepoConfig instance
    pub(crate) fn new(repo: &'a Repository) -> Self {
        Self { repo }
    }

    pub const USER_NAME_KEY: &'static str = "user.name";
    pub const USER_EMAIL_KEY: &'static str = "user.email";

    /// Configure git user name and email for this repository
    ///
    /// This is a convenience method that sets both user.name and user.email
    /// configuration values.
    ///
    /// # Arguments
    ///
    /// * `name` - The user's full name
    /// * `email` - The user's email address
    ///
    /// # Example
    ///
    /// ```rust
    /// use rustic_git::Repository;
    /// use std::{env, fs};
    ///
    /// let test_path = env::temp_dir().join("config_set_user_test");
    /// if test_path.exists() {
    ///     fs::remove_dir_all(&test_path).unwrap();
    /// }
    ///
    /// let repo = Repository::init(&test_path, false)?;
    /// repo.config().set_user("John Doe", "john@example.com")?;
    ///
    /// // Clean up
    /// fs::remove_dir_all(&test_path).unwrap();
    /// # Ok::<(), rustic_git::GitError>(())
    /// ```
    pub fn set_user(&self, name: &str, email: &str) -> Result<()> {
        self.set(Self::USER_NAME_KEY, name)?;
        self.set(Self::USER_EMAIL_KEY, email)?;
        Ok(())
    }

    /// Get the current git user configuration
    ///
    /// Returns a tuple of (name, email) from the repository configuration.
    ///
    /// # Returns
    ///
    /// A tuple containing the user name and email, or an error if either
    /// configuration value is not set.
    pub fn get_user(&self) -> Result<(String, String)> {
        let name = self.get(Self::USER_NAME_KEY)?;
        let email = self.get(Self::USER_EMAIL_KEY)?;
        Ok((name, email))
    }

    /// Set a git configuration value for this repository
    ///
    /// # Arguments
    ///
    /// * `key` - The configuration key (e.g., "user.name", "core.autocrlf")
    /// * `value` - The value to set
    ///
    /// # Example
    ///
    /// ```rust
    /// use rustic_git::Repository;
    /// use std::{env, fs};
    ///
    /// let test_path = env::temp_dir().join("config_set_test");
    /// if test_path.exists() {
    ///     fs::remove_dir_all(&test_path).unwrap();
    /// }
    ///
    /// let repo = Repository::init(&test_path, false)?;
    /// repo.config().set("core.autocrlf", "false")?;
    /// repo.config().set("user.name", "Jane Doe")?;
    ///
    /// // Clean up
    /// fs::remove_dir_all(&test_path).unwrap();
    /// # Ok::<(), rustic_git::GitError>(())
    /// ```
    pub fn set(&self, key: &str, value: &str) -> Result<()> {
        git(&["config", key, value], Some(self.repo.repo_path()))?;
        Ok(())
    }

    /// Get a git configuration value from this repository
    ///
    /// # Arguments
    ///
    /// * `key` - The configuration key to retrieve
    ///
    /// # Returns
    ///
    /// The configuration value as a string, or an error if the key is not found.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rustic_git::Repository;
    /// use std::{env, fs};
    ///
    /// let test_path = env::temp_dir().join("config_get_test");
    /// if test_path.exists() {
    ///     fs::remove_dir_all(&test_path).unwrap();
    /// }
    ///
    /// let repo = Repository::init(&test_path, false)?;
    /// repo.config().set("user.name", "Jane Doe")?;
    /// let name = repo.config().get("user.name")?;
    /// assert_eq!(name, "Jane Doe");
    ///
    /// // Clean up
    /// fs::remove_dir_all(&test_path).unwrap();
    /// # Ok::<(), rustic_git::GitError>(())
    /// ```
    pub fn get(&self, key: &str) -> Result<String> {
        git(&["config", key], Some(self.repo.repo_path())).map(|s| s.trim().to_string())
    }

    /// Remove a git configuration value from this repository
    ///
    /// # Arguments
    ///
    /// * `key` - The configuration key to remove
    ///
    /// # Example
    ///
    /// ```rust
    /// use rustic_git::Repository;
    /// use std::{env, fs};
    ///
    /// let test_path = env::temp_dir().join("config_unset_test");
    /// if test_path.exists() {
    ///     fs::remove_dir_all(&test_path).unwrap();
    /// }
    ///
    /// let repo = Repository::init(&test_path, false)?;
    /// repo.config().set("test.value", "temporary")?;
    /// repo.config().unset("test.value")?;
    ///
    /// // Clean up
    /// fs::remove_dir_all(&test_path).unwrap();
    /// # Ok::<(), rustic_git::GitError>(())
    /// ```
    pub fn unset(&self, key: &str) -> Result<()> {
        git(&["config", "--unset", key], Some(self.repo.repo_path()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;

    #[test]
    fn test_config_set_and_get_user() {
        let test_path = env::temp_dir().join("test_config_user");

        // Clean up if exists
        if test_path.exists() {
            fs::remove_dir_all(&test_path).unwrap();
        }

        let repo = Repository::init(&test_path, false).unwrap();

        // Set user configuration
        repo.config()
            .set_user("Test User", "test@example.com")
            .unwrap();

        // Get user configuration
        let (name, email) = repo.config().get_user().unwrap();
        assert_eq!(name, "Test User");
        assert_eq!(email, "test@example.com");

        // Clean up
        fs::remove_dir_all(&test_path).unwrap();
    }

    #[test]
    fn test_config_set_and_get_generic() {
        let test_path = env::temp_dir().join("test_config_generic");

        // Clean up if exists
        if test_path.exists() {
            fs::remove_dir_all(&test_path).unwrap();
        }

        let repo = Repository::init(&test_path, false).unwrap();

        // Set generic configuration
        repo.config().set("core.autocrlf", "false").unwrap();
        repo.config().set("user.name", "Generic User").unwrap();

        // Get generic configuration
        let autocrlf = repo.config().get("core.autocrlf").unwrap();
        let name = repo.config().get("user.name").unwrap();

        assert_eq!(autocrlf, "false");
        assert_eq!(name, "Generic User");

        // Clean up
        fs::remove_dir_all(&test_path).unwrap();
    }

    #[test]
    fn test_config_unset() {
        let test_path = env::temp_dir().join("test_config_unset");

        // Clean up if exists
        if test_path.exists() {
            fs::remove_dir_all(&test_path).unwrap();
        }

        let repo = Repository::init(&test_path, false).unwrap();

        // Set a test value
        repo.config().set("test.temporary", "value").unwrap();
        let value = repo.config().get("test.temporary").unwrap();
        assert_eq!(value, "value");

        // Unset the value
        repo.config().unset("test.temporary").unwrap();

        // Verify it's gone (should return error)
        let result = repo.config().get("test.temporary");
        assert!(result.is_err());

        // Clean up
        fs::remove_dir_all(&test_path).unwrap();
    }

    #[test]
    fn test_config_get_nonexistent_key() {
        let test_path = env::temp_dir().join("test_config_nonexistent");

        // Clean up if exists
        if test_path.exists() {
            fs::remove_dir_all(&test_path).unwrap();
        }

        let repo = Repository::init(&test_path, false).unwrap();

        // Try to get a non-existent key
        let result = repo.config().get("nonexistent.key");
        assert!(result.is_err());

        // Clean up
        fs::remove_dir_all(&test_path).unwrap();
    }

    #[test]
    fn test_config_integration_with_commit() {
        let test_path = env::temp_dir().join("test_config_commit_integration");

        // Clean up if exists
        if test_path.exists() {
            fs::remove_dir_all(&test_path).unwrap();
        }

        let repo = Repository::init(&test_path, false).unwrap();

        // Configure user for commits
        repo.config()
            .set_user("Integration Test", "integration@example.com")
            .unwrap();

        // Create a file and commit
        std::fs::write(test_path.join("test.txt"), "test content").unwrap();
        repo.add(&["test.txt"]).unwrap();
        let hash = repo.commit("Test commit with config API").unwrap();

        // Verify commit was created successfully
        assert!(!hash.as_str().is_empty());

        // Clean up
        fs::remove_dir_all(&test_path).unwrap();
    }
}
