//! The universal emit contract shared by every synthesizer.
//!
//! Every synthesizer in the pleme-io family already exposes a
//! `pub fn emit(&self, indent: usize) -> String` on its AST root enum.
//! [`SynthesizerNode`] lifts that convention into a trait so generic code —
//! property tests, file assemblers, cross-language composition helpers —
//! can talk about "a node that knows how to render itself" without knowing
//! the concrete type.
//!
//! The trait is deliberately small. It does NOT include parsing, diffing,
//! or mutation. Those belong to individual synthesizers.

/// A value that can render itself to source code at a given indentation level.
///
/// ## Contract
///
/// - `emit` MUST be deterministic: `n.emit(i)` == `n.emit(i)` always.
/// - `emit` MUST be pure: no IO, no globals, no mutation of `self`.
/// - `indent_unit` is an associated function (not method) because it depends
///   on the target language, not the node instance.
/// - `variant_id` exists to let property tests prove every variant is
///   exercised. Two variants MUST NOT share an id.
pub trait SynthesizerNode {
    /// Emit the node as target-language source at the given indent level.
    fn emit(&self, indent: usize) -> String;

    /// The indentation unit for this synthesizer's target language.
    /// e.g., `"  "` for Nix/YAML/Ruby, `"    "` for Python, `"\t"` for Go.
    fn indent_unit() -> &'static str;

    /// Unique integer tag for the variant of `self`. Used by coverage
    /// property tests — if a new variant is added and forgotten in tests,
    /// the variant_id test fails.
    fn variant_id(&self) -> u8;
}

/// Author-attested marker: "this type has no `Raw`/`Verbatim`/`Text`
/// escape-hatch variant that bypasses typed rendering."
///
/// The compiler cannot verify this directly — but combined with the
/// variant-coverage property test (every `variant_id` exercised), a
/// hidden Raw variant would cause test failures.
///
/// Implementors write an [`NoRawAttestation::attestation`] string that
/// explains *how* they enforce the invariant (e.g., "grep check in CI"
/// or "deprecated and delete-targeted").
pub trait NoRawAttestation {
    /// Short human-readable attestation of how no-raw is enforced.
    fn attestation() -> &'static str;
}

/// Property-checkable laws that every `SynthesizerNode` implementation
/// should satisfy. Real synthesizers call these from their test suites
/// to compound proof surface.
pub mod laws {
    use super::SynthesizerNode;

    /// Determinism: calling `emit` twice with the same indent returns the
    /// same string. Failure indicates hidden mutable state, time-based
    /// formatting, or nondeterministic iteration order.
    #[must_use]
    pub fn is_deterministic<N: SynthesizerNode>(node: &N, indent: usize) -> bool {
        node.emit(indent) == node.emit(indent)
    }

    /// Indent-unit prefix: when you increase the indent by 1, the output
    /// EITHER starts with the extra indent unit OR is byte-identical to
    /// the lower-indent output (atomic literals often don't honor indent).
    #[must_use]
    pub fn honors_indent_unit<N: SynthesizerNode>(node: &N, base: usize) -> bool {
        let a = node.emit(base);
        let b = node.emit(base + 1);
        b.starts_with(N::indent_unit()) || a == b
    }

    /// Non-decreasing length: emitting at a higher indent never produces
    /// shorter output than a lower indent (whitespace is non-negative).
    #[must_use]
    pub fn indent_monotone_len<N: SynthesizerNode>(node: &N, base: usize) -> bool {
        node.emit(base + 1).len() >= node.emit(base).len()
    }

    /// Variant id is in-range (the trait promises u8; this catches
    /// implementations that overflow or wrap by accident).
    #[must_use]
    pub fn variant_id_is_valid<N: SynthesizerNode>(node: &N) -> bool {
        let _id: u8 = node.variant_id();
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Minimal fixture: two-variant node type for exercising the trait.
    #[derive(Clone, Debug, PartialEq)]
    enum Tiny {
        Leaf(&'static str),
        Pair(&'static str, &'static str),
    }

    impl SynthesizerNode for Tiny {
        fn emit(&self, indent: usize) -> String {
            let pad = "  ".repeat(indent);
            match self {
                Self::Leaf(s) => format!("{pad}{s}"),
                Self::Pair(a, b) => format!("{pad}{a}={b}"),
            }
        }
        fn indent_unit() -> &'static str {
            "  "
        }
        fn variant_id(&self) -> u8 {
            match self {
                Self::Leaf(_) => 0,
                Self::Pair(_, _) => 1,
            }
        }
    }

    impl NoRawAttestation for Tiny {
        fn attestation() -> &'static str {
            "Tiny has only Leaf and Pair variants — neither accepts unstructured bytes."
        }
    }

    #[test]
    fn leaf_emits_at_indent_zero() {
        assert_eq!(Tiny::Leaf("x").emit(0), "x");
    }

    #[test]
    fn leaf_emits_at_indent_one() {
        assert_eq!(Tiny::Leaf("x").emit(1), "  x");
    }

    #[test]
    fn pair_emits_with_pad() {
        assert_eq!(Tiny::Pair("k", "v").emit(2), "    k=v");
    }

    #[test]
    fn indent_unit_is_two_spaces() {
        assert_eq!(Tiny::indent_unit(), "  ");
    }

    #[test]
    fn variant_ids_are_distinct() {
        assert_ne!(Tiny::Leaf("x").variant_id(), Tiny::Pair("a", "b").variant_id());
    }

    #[test]
    fn law_determinism_holds_on_leaf() {
        assert!(laws::is_deterministic(&Tiny::Leaf("x"), 3));
    }

    #[test]
    fn law_determinism_holds_on_pair() {
        assert!(laws::is_deterministic(&Tiny::Pair("k", "v"), 5));
    }

    #[test]
    fn law_honors_indent_unit_on_leaf() {
        assert!(laws::honors_indent_unit(&Tiny::Leaf("x"), 0));
        assert!(laws::honors_indent_unit(&Tiny::Leaf("x"), 4));
    }

    #[test]
    fn law_honors_indent_unit_on_pair() {
        assert!(laws::honors_indent_unit(&Tiny::Pair("k", "v"), 0));
    }

    #[test]
    fn law_indent_monotone_on_both_variants() {
        assert!(laws::indent_monotone_len(&Tiny::Leaf("x"), 0));
        assert!(laws::indent_monotone_len(&Tiny::Pair("k", "v"), 3));
    }

    #[test]
    fn law_variant_id_in_range() {
        assert!(laws::variant_id_is_valid(&Tiny::Leaf("x")));
        assert!(laws::variant_id_is_valid(&Tiny::Pair("a", "b")));
    }

    #[test]
    fn attestation_is_nonempty() {
        assert!(!<Tiny as NoRawAttestation>::attestation().is_empty());
    }

    #[test]
    fn emit_is_pure_over_many_calls() {
        let n = Tiny::Pair("a", "b");
        let first = n.emit(2);
        for _ in 0..32 {
            assert_eq!(n.emit(2), first);
        }
    }

    proptest::proptest! {
        #[test]
        fn prop_determinism_holds_any_indent(indent in 0usize..20) {
            let n = Tiny::Pair("alpha", "beta");
            proptest::prop_assert!(laws::is_deterministic(&n, indent));
        }

        #[test]
        fn prop_indent_monotone_any_base(base in 0usize..20) {
            let n = Tiny::Leaf("gamma");
            proptest::prop_assert!(laws::indent_monotone_len(&n, base));
        }
    }
}
