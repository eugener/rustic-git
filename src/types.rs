/// Represents a Git object hash (commit, tree, blob, etc.).
#[derive(Debug, Clone, PartialEq, Eq)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_creation() {
        let hash = Hash("abc123def456".to_string());
        assert_eq!(hash.0, "abc123def456");
    }

    #[test]
    fn test_hash_as_str() {
        let hash = Hash("abc123def456".to_string());
        assert_eq!(hash.as_str(), "abc123def456");
    }

    #[test]
    fn test_hash_short_full_length() {
        let hash = Hash("abc123def456".to_string());
        assert_eq!(hash.short(), "abc123d");
    }

    #[test]
    fn test_hash_short_exactly_seven() {
        let hash = Hash("abcdefg".to_string());
        assert_eq!(hash.short(), "abcdefg");
    }

    #[test]
    fn test_hash_short_less_than_seven() {
        let hash = Hash("abc".to_string());
        assert_eq!(hash.short(), "abc");
    }

    #[test]
    fn test_hash_short_empty() {
        let hash = Hash("".to_string());
        assert_eq!(hash.short(), "");
    }

    #[test]
    fn test_hash_display() {
        let hash = Hash("abc123def456".to_string());
        assert_eq!(format!("{}", hash), "abc123def456");
    }

    #[test]
    fn test_hash_from_string() {
        let hash: Hash = "abc123def456".to_string().into();
        assert_eq!(hash.0, "abc123def456");
    }

    #[test]
    fn test_hash_from_str() {
        let hash: Hash = "abc123def456".into();
        assert_eq!(hash.0, "abc123def456");
    }

    #[test]
    fn test_hash_clone() {
        let hash1 = Hash("abc123def456".to_string());
        let hash2 = hash1.clone();
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_debug() {
        let hash = Hash("abc123def456".to_string());
        let debug_str = format!("{:?}", hash);
        assert!(debug_str.contains("abc123def456"));
    }

    #[test]
    fn test_hash_partial_eq() {
        let hash1 = Hash("abc123def456".to_string());
        let hash2 = Hash("abc123def456".to_string());
        let hash3 = Hash("different".to_string());

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_hash_from_conversions_edge_cases() {
        let empty_string: Hash = "".into();
        assert_eq!(empty_string.0, "");

        let empty_owned: Hash = String::new().into();
        assert_eq!(empty_owned.0, "");

        let unicode: Hash = "🚀commit".into();
        assert_eq!(unicode.0, "🚀commit");
    }
}
