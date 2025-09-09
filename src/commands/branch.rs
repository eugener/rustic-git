use crate::types::Hash;
use crate::utils::git;
use crate::{Repository, Result};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BranchType {
    Local,
    RemoteTracking,
}

impl fmt::Display for BranchType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BranchType::Local => write!(f, "local"),
            BranchType::RemoteTracking => write!(f, "remote-tracking"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Branch {
    pub name: String,
    pub branch_type: BranchType,
    pub is_current: bool,
    pub commit_hash: Hash,
    pub upstream: Option<String>,
}

impl Branch {
    /// Check if this is a local branch
    pub fn is_local(&self) -> bool {
        matches!(self.branch_type, BranchType::Local)
    }

    /// Check if this is a remote-tracking branch
    pub fn is_remote(&self) -> bool {
        matches!(self.branch_type, BranchType::RemoteTracking)
    }

    /// Get the short name of the branch (without remote prefix for remote branches)
    pub fn short_name(&self) -> &str {
        if self.is_remote() && self.name.contains('/') {
            self.name.split('/').nth(1).unwrap_or(&self.name)
        } else {
            &self.name
        }
    }
}

impl fmt::Display for Branch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let marker = if self.is_current { "*" } else { " " };
        write!(f, "{} {}", marker, self.name)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BranchList {
    branches: Box<[Branch]>,
}

impl BranchList {
    /// Create a new BranchList from a vector of branches
    pub fn new(branches: Vec<Branch>) -> Self {
        Self {
            branches: branches.into_boxed_slice(),
        }
    }

    /// Get all branches
    pub fn all(&self) -> &[Branch] {
        &self.branches
    }

    /// Get an iterator over all branches
    pub fn iter(&self) -> impl Iterator<Item = &Branch> {
        self.branches.iter()
    }

    /// Get an iterator over local branches
    pub fn local(&self) -> impl Iterator<Item = &Branch> {
        self.branches.iter().filter(|b| b.is_local())
    }

    /// Get an iterator over remote-tracking branches
    pub fn remote(&self) -> impl Iterator<Item = &Branch> {
        self.branches.iter().filter(|b| b.is_remote())
    }

    /// Get the current branch
    pub fn current(&self) -> Option<&Branch> {
        self.branches.iter().find(|b| b.is_current)
    }

    /// Find a branch by name
    pub fn find(&self, name: &str) -> Option<&Branch> {
        self.branches.iter().find(|b| b.name == name)
    }

    /// Find a branch by short name (useful for remote branches)
    pub fn find_by_short_name(&self, short_name: &str) -> Option<&Branch> {
        self.branches.iter().find(|b| b.short_name() == short_name)
    }

    /// Check if the list is empty
    pub fn is_empty(&self) -> bool {
        self.branches.is_empty()
    }

    /// Get the count of branches
    pub fn len(&self) -> usize {
        self.branches.len()
    }

    /// Get count of local branches
    pub fn local_count(&self) -> usize {
        self.local().count()
    }

    /// Get count of remote-tracking branches
    pub fn remote_count(&self) -> usize {
        self.remote().count()
    }
}

impl fmt::Display for BranchList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for branch in &self.branches {
            writeln!(f, "{}", branch)?;
        }
        Ok(())
    }
}

impl Repository {
    /// List all branches in the repository
    pub fn branches(&self) -> Result<BranchList> {
        Self::ensure_git()?;

        // Use git branch -vv --all for comprehensive branch information
        let stdout = git(&["branch", "-vv", "--all"], Some(self.repo_path()))?;

        let branches = parse_branch_output(&stdout)?;
        Ok(BranchList::new(branches))
    }

    /// Get the current branch
    pub fn current_branch(&self) -> Result<Option<Branch>> {
        Self::ensure_git()?;

        let stdout = git(&["branch", "--show-current"], Some(self.repo_path()))?;
        let current_name = stdout.trim();

        if current_name.is_empty() {
            // Might be in detached HEAD state
            return Ok(None);
        }

        // Get detailed info about the current branch
        let branches = self.branches()?;
        Ok(branches.current().cloned())
    }

    /// Create a new branch
    pub fn create_branch(&self, name: &str, start_point: Option<&str>) -> Result<Branch> {
        Self::ensure_git()?;

        let mut args = vec!["branch", name];
        if let Some(start) = start_point {
            args.push(start);
        }

        let _stdout = git(&args, Some(self.repo_path()))?;

        // Get information about the newly created branch
        let branches = self.branches()?;
        branches.find(name).cloned().ok_or_else(|| {
            crate::error::GitError::CommandFailed(format!("Failed to create branch: {}", name))
        })
    }

    /// Delete a branch
    pub fn delete_branch(&self, branch: &Branch, force: bool) -> Result<()> {
        Self::ensure_git()?;

        if branch.is_current {
            return Err(crate::error::GitError::CommandFailed(
                "Cannot delete the current branch".to_string(),
            ));
        }

        let flag = if force { "-D" } else { "-d" };
        let args = vec!["branch", flag, &branch.name];

        let _stdout = git(&args, Some(self.repo_path()))?;
        Ok(())
    }

    /// Switch to an existing branch
    pub fn checkout(&self, branch: &Branch) -> Result<()> {
        Self::ensure_git()?;

        let branch_name = if branch.is_remote() {
            branch.short_name()
        } else {
            &branch.name
        };

        let _stdout = git(&["checkout", branch_name], Some(self.repo_path()))?;
        Ok(())
    }

    /// Create a new branch and switch to it
    pub fn checkout_new(&self, name: &str, start_point: Option<&str>) -> Result<Branch> {
        Self::ensure_git()?;

        let mut args = vec!["checkout", "-b", name];
        if let Some(start) = start_point {
            args.push(start);
        }

        let _stdout = git(&args, Some(self.repo_path()))?;

        // Get information about the newly created and checked out branch
        self.current_branch()?.ok_or_else(|| {
            crate::error::GitError::CommandFailed(format!(
                "Failed to create and checkout branch: {}",
                name
            ))
        })
    }
}

/// Parse the output of `git branch -vv --all`
fn parse_branch_output(output: &str) -> Result<Vec<Branch>> {
    let mut branches = Vec::new();

    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Skip the line that shows HEAD -> branch mapping for remotes
        if line.contains("->") {
            continue;
        }

        let is_current = line.starts_with('*');
        let line = if is_current {
            line[1..].trim() // Skip the '*' and trim
        } else {
            line.trim() // Just trim whitespace for non-current branches
        };

        // Parse branch name (first word)
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        let name = parts[0].to_string();

        // Determine branch type
        let branch_type = if name.starts_with("remotes/") {
            BranchType::RemoteTracking
        } else {
            BranchType::Local
        };

        // Extract commit hash (second part if available)
        let commit_hash = if parts.len() > 1 {
            Hash::from(parts[1].to_string())
        } else {
            Hash::from("0000000000000000000000000000000000000000".to_string())
        };

        // Extract upstream information (look for [upstream] pattern)
        let upstream = if let Some(bracket_start) = line.find('[') {
            if let Some(bracket_end) = line.find(']') {
                let upstream_info = &line[bracket_start + 1..bracket_end];
                // Extract just the upstream branch name, ignore ahead/behind info
                let upstream_branch = upstream_info
                    .split(':')
                    .next()
                    .unwrap_or(upstream_info)
                    .trim();
                if upstream_branch.is_empty() {
                    None
                } else {
                    Some(upstream_branch.to_string())
                }
            } else {
                None
            }
        } else {
            None
        };

        // Clean up remote branch names
        let clean_name = if branch_type == BranchType::RemoteTracking {
            name.strip_prefix("remotes/").unwrap_or(&name).to_string()
        } else {
            name
        };

        let branch = Branch {
            name: clean_name,
            branch_type,
            is_current,
            commit_hash,
            upstream,
        };

        branches.push(branch);
    }

    Ok(branches)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    #[test]
    fn test_branch_type_display() {
        assert_eq!(format!("{}", BranchType::Local), "local");
        assert_eq!(format!("{}", BranchType::RemoteTracking), "remote-tracking");
    }

    #[test]
    fn test_branch_is_local() {
        let branch = Branch {
            name: "main".to_string(),
            branch_type: BranchType::Local,
            is_current: true,
            commit_hash: Hash::from("abc123".to_string()),
            upstream: None,
        };

        assert!(branch.is_local());
        assert!(!branch.is_remote());
    }

    #[test]
    fn test_branch_is_remote() {
        let branch = Branch {
            name: "origin/main".to_string(),
            branch_type: BranchType::RemoteTracking,
            is_current: false,
            commit_hash: Hash::from("abc123".to_string()),
            upstream: None,
        };

        assert!(branch.is_remote());
        assert!(!branch.is_local());
    }

    #[test]
    fn test_branch_short_name() {
        let local_branch = Branch {
            name: "feature".to_string(),
            branch_type: BranchType::Local,
            is_current: false,
            commit_hash: Hash::from("abc123".to_string()),
            upstream: None,
        };

        let remote_branch = Branch {
            name: "origin/feature".to_string(),
            branch_type: BranchType::RemoteTracking,
            is_current: false,
            commit_hash: Hash::from("abc123".to_string()),
            upstream: None,
        };

        assert_eq!(local_branch.short_name(), "feature");
        assert_eq!(remote_branch.short_name(), "feature");
    }

    #[test]
    fn test_branch_display() {
        let current_branch = Branch {
            name: "main".to_string(),
            branch_type: BranchType::Local,
            is_current: true,
            commit_hash: Hash::from("abc123".to_string()),
            upstream: None,
        };

        let other_branch = Branch {
            name: "feature".to_string(),
            branch_type: BranchType::Local,
            is_current: false,
            commit_hash: Hash::from("def456".to_string()),
            upstream: None,
        };

        assert_eq!(format!("{}", current_branch), "* main");
        assert_eq!(format!("{}", other_branch), "  feature");
    }

    #[test]
    fn test_branch_list_creation() {
        let branches = vec![
            Branch {
                name: "main".to_string(),
                branch_type: BranchType::Local,
                is_current: true,
                commit_hash: Hash::from("abc123".to_string()),
                upstream: Some("origin/main".to_string()),
            },
            Branch {
                name: "origin/main".to_string(),
                branch_type: BranchType::RemoteTracking,
                is_current: false,
                commit_hash: Hash::from("abc123".to_string()),
                upstream: None,
            },
        ];

        let branch_list = BranchList::new(branches);

        assert_eq!(branch_list.len(), 2);
        assert_eq!(branch_list.local_count(), 1);
        assert_eq!(branch_list.remote_count(), 1);
        assert!(!branch_list.is_empty());
    }

    #[test]
    fn test_branch_list_find() {
        let branches = vec![
            Branch {
                name: "main".to_string(),
                branch_type: BranchType::Local,
                is_current: true,
                commit_hash: Hash::from("abc123".to_string()),
                upstream: None,
            },
            Branch {
                name: "origin/feature".to_string(),
                branch_type: BranchType::RemoteTracking,
                is_current: false,
                commit_hash: Hash::from("def456".to_string()),
                upstream: None,
            },
        ];

        let branch_list = BranchList::new(branches);

        assert!(branch_list.find("main").is_some());
        assert!(branch_list.find("origin/feature").is_some());
        assert!(branch_list.find("nonexistent").is_none());

        assert!(branch_list.find_by_short_name("main").is_some());
        assert!(branch_list.find_by_short_name("feature").is_some());
    }

    #[test]
    fn test_branch_list_current() {
        let branches = vec![
            Branch {
                name: "main".to_string(),
                branch_type: BranchType::Local,
                is_current: true,
                commit_hash: Hash::from("abc123".to_string()),
                upstream: None,
            },
            Branch {
                name: "feature".to_string(),
                branch_type: BranchType::Local,
                is_current: false,
                commit_hash: Hash::from("def456".to_string()),
                upstream: None,
            },
        ];

        let branch_list = BranchList::new(branches);
        let current = branch_list.current().unwrap();

        assert_eq!(current.name, "main");
        assert!(current.is_current);
    }

    #[test]
    fn test_parse_branch_output() {
        let output = r#"
* main                abc1234 [origin/main] Initial commit
  feature             def5678 Feature branch
  remotes/origin/main abc1234 Initial commit
"#;

        let branches = parse_branch_output(output).unwrap();

        assert_eq!(branches.len(), 3);

        // Check main branch
        let main_branch = branches.iter().find(|b| b.name == "main");
        assert!(main_branch.is_some());
        let main_branch = main_branch.unwrap();
        assert!(main_branch.is_current);
        assert_eq!(main_branch.branch_type, BranchType::Local);
        assert_eq!(main_branch.upstream, Some("origin/main".to_string()));

        // Check feature branch
        let feature_branch = branches.iter().find(|b| b.name == "feature").unwrap();
        assert!(!feature_branch.is_current);
        assert_eq!(feature_branch.branch_type, BranchType::Local);
        assert_eq!(feature_branch.upstream, None);

        // Check remote branch
        let remote_branch = branches.iter().find(|b| b.name == "origin/main").unwrap();
        assert!(!remote_branch.is_current);
        assert_eq!(remote_branch.branch_type, BranchType::RemoteTracking);
    }

    #[test]
    fn test_repository_current_branch() {
        let test_path = "/tmp/test_current_branch_repo";

        // Clean up if exists
        if Path::new(test_path).exists() {
            fs::remove_dir_all(test_path).unwrap();
        }

        // Create a repository and test current branch
        let repo = Repository::init(test_path, false).unwrap();

        // In a new repo, there might not be a current branch until first commit
        let _current = repo.current_branch().unwrap();
        // This might be None in a fresh repository with no commits

        // Clean up
        fs::remove_dir_all(test_path).unwrap();
    }

    #[test]
    fn test_repository_create_branch() {
        let test_path = "/tmp/test_create_branch_repo";

        // Clean up if exists
        if Path::new(test_path).exists() {
            fs::remove_dir_all(test_path).unwrap();
        }

        // Create a repository with an initial commit
        let repo = Repository::init(test_path, false).unwrap();

        // Create a test file and commit to have a valid HEAD
        std::fs::write(format!("{}/test.txt", test_path), "test content").unwrap();
        repo.add(&["test.txt"]).unwrap();
        repo.commit("Initial commit").unwrap();

        // Create a new branch
        let branch = repo.create_branch("feature", None).unwrap();
        assert_eq!(branch.name, "feature");
        assert_eq!(branch.branch_type, BranchType::Local);
        assert!(!branch.is_current);

        // Verify the branch exists in the branch list
        let branches = repo.branches().unwrap();
        assert!(branches.find("feature").is_some());

        // Clean up
        fs::remove_dir_all(test_path).unwrap();
    }
}
