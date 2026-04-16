//! Ordered element container trait shared across synthesizer ASTs.
//!
//! `NixNode::List(Vec<NixNode>)`, `RubyNode::ArrayLit(Vec<RubyNode>)`,
//! `YamlNode::Seq(Vec<YamlNode>)` — each is an ordered sequence with the
//! same operations. [`Sequence`] unifies them.
//!
//! Like [`super::mapping::Mapping`], this trait is ordered: iteration
//! order == push order. A `HashSet` is NOT a valid `Sequence` implementor.

/// An ordered, indexable sequence of elements.
///
/// ## Contract
///
/// - `push(v)` appends to the end.
/// - `get(i)` returns `Some(&v)` for `i < len`, `None` otherwise.
/// - `iter` yields elements in push order.
pub trait Sequence {
    /// Element type.
    type Item;

    /// Append to the end.
    fn push(&mut self, value: Self::Item);

    /// Get element by index. `None` if out of bounds.
    fn get(&self, index: usize) -> Option<&Self::Item>;

    /// Number of elements.
    fn len(&self) -> usize;

    /// True when `len() == 0`.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Iterate elements in push order.
    fn iter(&self) -> impl Iterator<Item = &Self::Item> + '_;
}

/// Blanket impl for `Vec<T>`. Matches the representation used by
/// `NixNode::List`, `RubyNode::ArrayLit`, `YamlNode::Seq`.
impl<T> Sequence for Vec<T> {
    type Item = T;

    fn push(&mut self, value: T) {
        Vec::push(self, value);
    }

    fn get(&self, index: usize) -> Option<&T> {
        <[T]>::get(self.as_slice(), index)
    }

    fn len(&self) -> usize {
        Vec::len(self)
    }

    fn iter(&self) -> impl Iterator<Item = &T> + '_ {
        <[T]>::iter(self.as_slice())
    }
}

/// Property-checkable laws for any `Sequence` implementation.
pub mod laws {
    use super::Sequence;

    /// After `push(v)`, `get(len-1)` returns `Some(&v)` (we just pushed).
    #[must_use]
    pub fn push_then_get_last<S>(s: &mut S, value: S::Item) -> bool
    where
        S: Sequence,
        S::Item: PartialEq + Clone,
    {
        let expected = value.clone();
        s.push(value);
        let last_idx = s.len().saturating_sub(1);
        s.get(last_idx) == Some(&expected)
    }

    /// `push` increments `len` by exactly 1.
    #[must_use]
    pub fn push_increments_len<S>(s: &mut S, value: S::Item) -> bool
    where
        S: Sequence,
    {
        let before = s.len();
        s.push(value);
        s.len() == before + 1
    }

    /// `iter().count() == len()`.
    #[must_use]
    pub fn iter_count_matches_len<S>(s: &S) -> bool
    where
        S: Sequence,
    {
        s.iter().count() == s.len()
    }

    /// `get(i)` for `i >= len` returns `None`.
    #[must_use]
    pub fn get_out_of_bounds_is_none<S>(s: &S) -> bool
    where
        S: Sequence,
    {
        s.get(s.len()).is_none() && s.get(s.len() + 10).is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vec_push_appends() {
        let mut s: Vec<i32> = Vec::new();
        Sequence::push(&mut s, 1);
        Sequence::push(&mut s, 2);
        Sequence::push(&mut s, 3);
        assert_eq!(s, vec![1, 2, 3]);
    }

    #[test]
    fn vec_get_in_bounds() {
        let s: Vec<&str> = vec!["a", "b", "c"];
        assert_eq!(<Vec<&str> as Sequence>::get(&s, 0), Some(&"a"));
        assert_eq!(<Vec<&str> as Sequence>::get(&s, 2), Some(&"c"));
    }

    #[test]
    fn vec_get_out_of_bounds() {
        let s: Vec<&str> = vec!["a"];
        assert_eq!(<Vec<&str> as Sequence>::get(&s, 1), None);
        assert_eq!(<Vec<&str> as Sequence>::get(&s, 99), None);
    }

    #[test]
    fn vec_is_empty_reflects_len() {
        let mut s: Vec<i32> = Vec::new();
        assert!(Sequence::is_empty(&s));
        Sequence::push(&mut s, 7);
        assert!(!Sequence::is_empty(&s));
    }

    #[test]
    fn vec_iter_in_push_order() {
        let mut s: Vec<i32> = Vec::new();
        for n in [10, 20, 30] {
            Sequence::push(&mut s, n);
        }
        let observed: Vec<i32> = Sequence::iter(&s).copied().collect();
        assert_eq!(observed, vec![10, 20, 30]);
    }

    #[test]
    fn law_push_then_get_last_holds() {
        let mut s: Vec<i32> = Vec::new();
        assert!(laws::push_then_get_last(&mut s, 42));
    }

    #[test]
    fn law_push_increments_len_holds() {
        let mut s: Vec<i32> = Vec::new();
        assert!(laws::push_increments_len(&mut s, 1));
        assert!(laws::push_increments_len(&mut s, 2));
        assert!(laws::push_increments_len(&mut s, 3));
    }

    #[test]
    fn law_iter_count_matches_len_holds() {
        let s: Vec<i32> = vec![1, 2, 3, 4, 5];
        assert!(laws::iter_count_matches_len(&s));
    }

    #[test]
    fn law_get_out_of_bounds_is_none_holds() {
        let s: Vec<i32> = vec![1, 2];
        assert!(laws::get_out_of_bounds_is_none(&s));
    }

    #[test]
    fn empty_sequence_passes_all_laws() {
        let s: Vec<i32> = Vec::new();
        assert!(laws::iter_count_matches_len(&s));
        assert!(laws::get_out_of_bounds_is_none(&s));
    }

    proptest::proptest! {
        #[test]
        fn prop_push_then_get_last_any_i32(v in proptest::prelude::any::<i32>()) {
            let mut s: Vec<i32> = Vec::new();
            proptest::prop_assert!(laws::push_then_get_last(&mut s, v));
        }

        #[test]
        fn prop_push_n_times_yields_len_n(
            elements in proptest::collection::vec(proptest::prelude::any::<i32>(), 0..50)
        ) {
            let mut s: Vec<i32> = Vec::new();
            for e in &elements {
                Sequence::push(&mut s, *e);
            }
            proptest::prop_assert_eq!(Sequence::len(&s), elements.len());
        }

        #[test]
        fn prop_iter_order_matches_push_order(
            elements in proptest::collection::vec(proptest::prelude::any::<i32>(), 0..20)
        ) {
            let mut s: Vec<i32> = Vec::new();
            for e in &elements {
                Sequence::push(&mut s, *e);
            }
            let observed: Vec<i32> = Sequence::iter(&s).copied().collect();
            proptest::prop_assert_eq!(observed, elements);
        }
    }
}
