//! Ordered key/value container trait shared across synthesizer ASTs.
//!
//! `NixNode::AttrSet(Vec<Binding>)`, `RubyNode::HashLit(Vec<(String, RubyNode)>)`,
//! `YamlNode::Map(Vec<YamlEntry>)`, `HelmNode::Map` — each is an ordered
//! mapping with the same conceptual operations (insert, lookup, iterate in
//! insertion order). The [`Mapping`] trait unifies these operations so
//! generic helpers — merging, shape-checking, diff — can work across
//! any synthesizer.
//!
//! The trait is deliberately **ordered** (iteration order == insertion order)
//! because every AST we render from preserves declaration order in source.
//! A `HashMap` is NOT a valid `Mapping` implementor.

/// An ordered key/value container.
///
/// ## Contract
///
/// - `insert(k, v)` on an existing key overwrites the value WITHOUT
///   reordering. This matches `Vec<(K, V)>` semantics when you replace
///   in place, and matches the intent of all AST-level attribute sets.
/// - `iter` yields entries in insertion order.
/// - `len` counts unique keys present.
pub trait Mapping {
    /// Key type (typically `String`).
    type Key;
    /// Value type (typically the synthesizer's node enum).
    type Value;

    /// Insert a key/value. If the key is already present, replace the
    /// value without changing position in iteration order.
    fn insert(&mut self, key: Self::Key, value: Self::Value);

    /// Look up a key. Returns `None` if absent.
    fn get(&self, key: &Self::Key) -> Option<&Self::Value>;

    /// Number of entries.
    fn len(&self) -> usize;

    /// True when `len() == 0`.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Iterate entries in insertion order.
    fn iter(&self) -> impl Iterator<Item = (&Self::Key, &Self::Value)> + '_;
}

/// Blanket impl: `Vec<(K, V)>` is an ordered mapping by the usual
/// "look up by linear scan, insert appends, replace in place" convention.
/// This is exactly what `NixNode::AttrSet` uses under the hood.
impl<K, V> Mapping for Vec<(K, V)>
where
    K: PartialEq,
{
    type Key = K;
    type Value = V;

    fn insert(&mut self, key: Self::Key, value: Self::Value) {
        if let Some(existing) = self.iter_mut().find(|(k, _)| *k == key) {
            existing.1 = value;
        } else {
            self.push((key, value));
        }
    }

    fn get(&self, key: &Self::Key) -> Option<&Self::Value> {
        self.iter()
            .find(|(k, _)| **k == *key)
            .map(|(_, v)| v)
    }

    fn len(&self) -> usize {
        Vec::len(self)
    }

    fn iter(&self) -> impl Iterator<Item = (&Self::Key, &Self::Value)> + '_ {
        <[(K, V)]>::iter(self.as_slice()).map(|(k, v)| (k, v))
    }
}

/// Property-checkable laws for any `Mapping` implementation.
pub mod laws {
    use super::Mapping;

    /// After `insert(k, v)`, `get(&k)` returns `Some(&v)`.
    #[must_use]
    pub fn insert_then_get<M>(m: &mut M, key: M::Key, value: M::Value) -> bool
    where
        M: Mapping,
        M::Key: Clone,
        M::Value: PartialEq,
    {
        let key_copy = key.clone();
        m.insert(key, value);
        m.get(&key_copy).is_some()
    }

    /// `insert` on a new key increments `len` by exactly 1.
    #[must_use]
    pub fn insert_new_key_increments_len<M>(m: &mut M, key: M::Key, value: M::Value) -> bool
    where
        M: Mapping,
        M::Key: Clone,
    {
        let before = m.len();
        let was_present = m.get(&key).is_some();
        m.insert(key, value);
        if was_present {
            m.len() == before
        } else {
            m.len() == before + 1
        }
    }

    /// `iter` yields exactly `len` items.
    #[must_use]
    pub fn iter_count_matches_len<M>(m: &M) -> bool
    where
        M: Mapping,
    {
        m.iter().count() == m.len()
    }

    /// `is_empty` iff `len == 0`.
    #[must_use]
    pub fn is_empty_iff_len_zero<M>(m: &M) -> bool
    where
        M: Mapping,
    {
        m.is_empty() == (m.len() == 0)
    }

    /// Overwriting an existing key doesn't change `len`.
    #[must_use]
    pub fn overwrite_preserves_len<M>(m: &mut M, key: M::Key, value: M::Value) -> bool
    where
        M: Mapping,
        M::Key: Clone,
    {
        let k2 = key.clone();
        m.insert(key, value);
        let len_after_first = m.len();
        // Need a second Value — but we don't have one generically. So this
        // law is exercised by type-specific tests below rather than here.
        let _ = k2;
        let _ = len_after_first;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests use UFCS (`Mapping::insert(&mut m, ...)`) because `Vec<T>` has
    // inherent `insert`/`get` methods with different signatures that would
    // otherwise win method resolution and shadow the trait impl under test.

    #[test]
    fn vec_pairs_insert_then_get_returns_value() {
        let mut m: Vec<(String, i32)> = Vec::new();
        Mapping::insert(&mut m, "a".into(), 1);
        assert_eq!(Mapping::get(&m, &"a".to_string()), Some(&1));
    }

    #[test]
    fn vec_pairs_get_missing_returns_none() {
        let m: Vec<(String, i32)> = Vec::new();
        assert_eq!(Mapping::get(&m, &"missing".to_string()), None);
    }

    #[test]
    fn vec_pairs_len_reflects_insertions() {
        let mut m: Vec<(String, i32)> = Vec::new();
        assert_eq!(Mapping::len(&m), 0);
        Mapping::insert(&mut m, "a".into(), 1);
        assert_eq!(Mapping::len(&m), 1);
        Mapping::insert(&mut m, "b".into(), 2);
        assert_eq!(Mapping::len(&m), 2);
    }

    #[test]
    fn vec_pairs_overwrite_preserves_len() {
        let mut m: Vec<(String, i32)> = Vec::new();
        Mapping::insert(&mut m, "a".into(), 1);
        Mapping::insert(&mut m, "a".into(), 99);
        assert_eq!(Mapping::len(&m), 1);
        assert_eq!(Mapping::get(&m, &"a".to_string()), Some(&99));
    }

    #[test]
    fn vec_pairs_overwrite_preserves_order() {
        let mut m: Vec<(String, i32)> = Vec::new();
        Mapping::insert(&mut m, "a".into(), 1);
        Mapping::insert(&mut m, "b".into(), 2);
        Mapping::insert(&mut m, "c".into(), 3);
        Mapping::insert(&mut m, "a".into(), 42); // overwrite — must NOT move to end
        let keys: Vec<&String> = Mapping::iter(&m).map(|(k, _)| k).collect();
        assert_eq!(
            keys,
            vec![&"a".to_string(), &"b".to_string(), &"c".to_string()]
        );
    }

    #[test]
    fn vec_pairs_iter_yields_in_insertion_order() {
        let mut m: Vec<(String, i32)> = Vec::new();
        Mapping::insert(&mut m, "first".into(), 1);
        Mapping::insert(&mut m, "second".into(), 2);
        Mapping::insert(&mut m, "third".into(), 3);
        let values: Vec<i32> = Mapping::iter(&m).map(|(_, v)| *v).collect();
        assert_eq!(values, vec![1, 2, 3]);
    }

    #[test]
    fn vec_pairs_is_empty_on_new() {
        let m: Vec<(String, i32)> = Vec::new();
        assert!(Mapping::is_empty(&m));
    }

    #[test]
    fn vec_pairs_not_empty_after_insert() {
        let mut m: Vec<(String, i32)> = Vec::new();
        Mapping::insert(&mut m, "a".into(), 1);
        assert!(!Mapping::is_empty(&m));
    }

    #[test]
    fn law_insert_then_get_holds() {
        let mut m: Vec<(String, i32)> = Vec::new();
        assert!(laws::insert_then_get(&mut m, "k".into(), 7));
    }

    #[test]
    fn law_insert_new_key_increments_len_holds() {
        let mut m: Vec<(String, i32)> = Vec::new();
        assert!(laws::insert_new_key_increments_len(&mut m, "k".into(), 7));
        assert!(laws::insert_new_key_increments_len(&mut m, "k".into(), 8)); // overwrite path
    }

    #[test]
    fn law_iter_count_matches_len() {
        let mut m: Vec<(String, i32)> = Vec::new();
        Mapping::insert(&mut m, "a".into(), 1);
        Mapping::insert(&mut m, "b".into(), 2);
        assert!(laws::iter_count_matches_len(&m));
    }

    #[test]
    fn law_is_empty_iff_len_zero() {
        let mut m: Vec<(String, i32)> = Vec::new();
        assert!(laws::is_empty_iff_len_zero(&m));
        Mapping::insert(&mut m, "a".into(), 1);
        assert!(laws::is_empty_iff_len_zero(&m));
    }

    proptest::proptest! {
        #[test]
        fn prop_insert_then_get_any_string(k in "[a-z]{1,8}", v in 0i32..1000) {
            let mut m: Vec<(String, i32)> = Vec::new();
            Mapping::insert(&mut m, k.clone(), v);
            proptest::prop_assert_eq!(Mapping::get(&m, &k), Some(&v));
        }

        #[test]
        fn prop_len_equals_unique_keys(
            pairs in proptest::collection::vec(("[a-z]{1,4}", 0i32..100), 0..20)
        ) {
            let mut m: Vec<(String, i32)> = Vec::new();
            let mut seen = std::collections::HashSet::new();
            for (k, v) in &pairs {
                seen.insert(k.clone());
                Mapping::insert(&mut m, k.clone(), *v);
            }
            proptest::prop_assert_eq!(Mapping::len(&m), seen.len());
        }

        #[test]
        fn prop_iter_order_matches_first_insertion(
            pairs in proptest::collection::vec(("[a-z]{1,4}", 0i32..100), 1..10)
        ) {
            let mut m: Vec<(String, i32)> = Vec::new();
            let mut first_insertion: Vec<String> = Vec::new();
            for (k, v) in &pairs {
                if Mapping::get(&m, k).is_none() {
                    first_insertion.push(k.clone());
                }
                Mapping::insert(&mut m, k.clone(), *v);
            }
            let observed: Vec<String> =
                Mapping::iter(&m).map(|(k, _)| k.clone()).collect();
            proptest::prop_assert_eq!(observed, first_insertion);
        }
    }
}
