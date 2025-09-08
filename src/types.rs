/// Represents a Git object hash (commit, tree, blob, etc.).
#[derive(Debug, Clone, PartialEq)]
pub struct Hash(pub String);

impl Hash {
    /// Get the hash as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
    
    /// Get the short version of the hash (first 7 characters).
    pub fn short(&self) -> &str {
        if self.0.len() >= 7 {
            &self.0[..7]
        } else {
            &self.0
        }
    }
}

impl std::fmt::Display for Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for Hash {
    fn from(s: String) -> Self {
        Hash(s)
    }
}

impl From<&str> for Hash {
    fn from(s: &str) -> Self {
        Hash(s.to_string())
    }
}