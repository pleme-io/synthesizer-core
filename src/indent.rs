//! Parameterized indentation for synthesizer output.
//!
//! Different target languages have different indentation conventions:
//! Nix / YAML / Ruby / Rust use 2 spaces, Python / SQL use 4, Go uses
//! tabs. `IndentStyle` captures the choice as data, so renderers can
//! share indent-manipulation code.
//!
//! Prefer this over `format!("{}{}", " ".repeat(n*2), body)` in new code.

/// The indentation convention for a target language.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndentStyle {
    /// N spaces per level.
    Spaces(u8),
    /// A single tab per level.
    Tab,
}

impl IndentStyle {
    /// Two-space indent — used by Nix, YAML, Ruby, Rust, TypeScript.
    #[must_use]
    pub const fn two_spaces() -> Self {
        Self::Spaces(2)
    }

    /// Four-space indent — used by Python, SQL.
    #[must_use]
    pub const fn four_spaces() -> Self {
        Self::Spaces(4)
    }

    /// Tab indent — used by Go.
    #[must_use]
    pub const fn tab() -> Self {
        Self::Tab
    }

    /// Emit the prefix for `level` levels of indentation.
    #[must_use]
    pub fn prefix(&self, level: usize) -> String {
        match self {
            Self::Spaces(n) => " ".repeat(level * usize::from(*n)),
            Self::Tab => "\t".repeat(level),
        }
    }

    /// The single-level indent unit (equivalent to `prefix(1)`).
    #[must_use]
    pub fn unit(&self) -> String {
        self.prefix(1)
    }

    /// Indent a single line. Appends no newline.
    #[must_use]
    pub fn indent_line(&self, line: &str, level: usize) -> String {
        let mut out = self.prefix(level);
        out.push_str(line);
        out
    }

    /// Indent each line of `body` by `level`. Preserves embedded newlines.
    /// Empty lines are left empty (no trailing whitespace pollution).
    #[must_use]
    pub fn indent_block(&self, body: &str, level: usize) -> String {
        let prefix = self.prefix(level);
        body.lines()
            .map(|line| {
                if line.is_empty() {
                    String::new()
                } else {
                    format!("{prefix}{line}")
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_spaces_level_zero_is_empty() {
        assert_eq!(IndentStyle::two_spaces().prefix(0), "");
    }

    #[test]
    fn two_spaces_level_one_is_two_spaces() {
        assert_eq!(IndentStyle::two_spaces().prefix(1), "  ");
    }

    #[test]
    fn two_spaces_level_three_is_six_spaces() {
        assert_eq!(IndentStyle::two_spaces().prefix(3), "      ");
    }

    #[test]
    fn four_spaces_level_two_is_eight_spaces() {
        assert_eq!(IndentStyle::four_spaces().prefix(2), "        ");
    }

    #[test]
    fn tab_level_zero_is_empty() {
        assert_eq!(IndentStyle::tab().prefix(0), "");
    }

    #[test]
    fn tab_level_one_is_one_tab() {
        assert_eq!(IndentStyle::tab().prefix(1), "\t");
    }

    #[test]
    fn tab_level_four_is_four_tabs() {
        assert_eq!(IndentStyle::tab().prefix(4), "\t\t\t\t");
    }

    #[test]
    fn unit_equals_prefix_one() {
        assert_eq!(IndentStyle::two_spaces().unit(), IndentStyle::two_spaces().prefix(1));
        assert_eq!(IndentStyle::four_spaces().unit(), IndentStyle::four_spaces().prefix(1));
        assert_eq!(IndentStyle::tab().unit(), IndentStyle::tab().prefix(1));
    }

    #[test]
    fn indent_line_combines_prefix_and_body() {
        let style = IndentStyle::two_spaces();
        assert_eq!(style.indent_line("hello", 2), "    hello");
    }

    #[test]
    fn indent_block_per_line() {
        let style = IndentStyle::two_spaces();
        let body = "line1\nline2\nline3";
        let out = style.indent_block(body, 1);
        assert_eq!(out, "  line1\n  line2\n  line3");
    }

    #[test]
    fn indent_block_preserves_empty_lines_without_whitespace() {
        let style = IndentStyle::two_spaces();
        let body = "line1\n\nline3";
        let out = style.indent_block(body, 1);
        assert_eq!(out, "  line1\n\n  line3");
    }

    #[test]
    fn indent_block_single_line() {
        let style = IndentStyle::four_spaces();
        assert_eq!(style.indent_block("solo", 1), "    solo");
    }

    #[test]
    fn indent_block_empty_input_is_empty() {
        let style = IndentStyle::tab();
        assert_eq!(style.indent_block("", 5), "");
    }

    #[test]
    fn custom_spaces_count_works() {
        let style = IndentStyle::Spaces(3);
        assert_eq!(style.prefix(2), "      "); // 3 * 2 = 6 spaces
    }

    #[test]
    fn zero_space_style_emits_empty_prefix() {
        let style = IndentStyle::Spaces(0);
        assert_eq!(style.prefix(5), "");
    }

    proptest::proptest! {
        #[test]
        fn prop_two_spaces_prefix_len_is_2n(n in 0usize..50) {
            proptest::prop_assert_eq!(IndentStyle::two_spaces().prefix(n).len(), 2 * n);
        }

        #[test]
        fn prop_tab_prefix_len_is_n(n in 0usize..50) {
            proptest::prop_assert_eq!(IndentStyle::tab().prefix(n).len(), n);
        }

        #[test]
        fn prop_indent_line_starts_with_prefix(level in 0usize..10, body in "[a-z]{1,20}") {
            let style = IndentStyle::two_spaces();
            let out = style.indent_line(&body, level);
            proptest::prop_assert!(out.starts_with(&style.prefix(level)));
            proptest::prop_assert!(out.ends_with(&body));
        }
    }
}
