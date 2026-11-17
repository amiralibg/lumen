//! Stable widget identity.

use std::hash::{Hash, Hasher};

/// A widget identity, stable across frames. Derived by hashing a parent id
/// with a key (usually the widget kind and label).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct Id(pub u64);

impl Id {
    pub const ROOT: Id = Id(0x6c75_6d65_6e00_0001);

    pub fn new(key: impl Hash) -> Id {
        Id::ROOT.with(key)
    }

    /// A child id of `self`.
    pub fn with(self, key: impl Hash) -> Id {
        let mut h = ahash::AHasher::default();
        self.0.hash(&mut h);
        key.hash(&mut h);
        Id(h.finish())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_and_distinct() {
        assert_eq!(Id::new("a"), Id::new("a"));
        assert_ne!(Id::new("a"), Id::new("b"));
        assert_ne!(Id::new("a").with(1), Id::new("a").with(2));
    }
}
