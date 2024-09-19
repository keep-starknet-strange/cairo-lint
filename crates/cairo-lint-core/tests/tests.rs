use std::collections::HashMap;
use std::path::Path;
use std::sync::{LazyLock, Mutex};

use annotate_snippets::Renderer;
use cairo_lang_compiler::db::RootDatabase;
use cairo_lang_filesystem::ids::FileId;
use cairo_lang_semantic::diagnostic::SemanticDiagnosticKind;
use cairo_lang_semantic::inline_macros::get_default_plugin_suite;
use cairo_lang_semantic::test_utils::setup_test_crate_ex;
use cairo_lang_syntax::node::SyntaxNode;
use cairo_lang_test_plugin::test_plugin_suite;
use cairo_lang_test_utils::parse_test_file::{dump_to_test_file, parse_test_file, Test};
use cairo_lang_utils::ordered_hash_map::OrderedHashMap;
use cairo_lang_utils::Upcast;
use cairo_lint_core::diagnostics::format_diagnostic;
use cairo_lint_core::fix::{apply_import_fixes, collect_unused_imports, fix_semantic_diagnostic, Fix, ImportFix};
use cairo_lint_core::plugin::cairo_lint_plugin_suite;
use cairo_lint_test_utils::{get_diags, test_file, Tests};
use ctor::dtor;
use itertools::Itertools;
use paste::paste;
use pretty_assertions::assert_eq;
use test_case::test_case;

const CRATE_CONFIG: &str = r#"
edition = "2024_07"

[experimental_features]
negative_impls = true
coupons = true
"#;

test_file!(unused_variables, unused_variables, "one unused variable", "two unused variable", "plenty unused variables");

test_file!(
    single_match,
    destructuring_match,
    "simple destructuring match",
    "simple destructuring match second arm",
    "simple destructuring match with scope",
    "simple destructuring match with unit in scope",
    "nested destructuring match",
    "destructuring match twisted",
    "destructuring match twisted differently",
    "destructuring match second arm",
    "destructuring comprehensive match",
    "reversed destructuring comprehensive match",
    "simple destructuring match with unit and comment in scope",
    "simple destructuring match with comment in scope"
);

test_file!(
    unused_imports,
    unused_imports,
    "single unused import",
    "multiple unused imports",
    "unused import aliased",
    "unused import trait",
    "multi with one used and one unused",
    "mix of multi and leaf imports in a single statement",
    "multiple import statements lines with some used and some unused"
);

test_file!(
    double_parens,
    double_parens,
    "simple double parens",
    "unnecessary parentheses in arithmetic expression",
    "necessary parentheses in arithmetic expression",
    "tuple double parens",
    "assert expressions",
    "double parens with function call",
    "double parens with return",
    "double parens in let statement",
    "double parens in struct field access",
    "double parens in match arm"
);

test_file!(
    double_comparison,
    double_comparison,
    "double comparison equal or greater than",
    "double comparison equal or less than",
    "double comparison greater than or equal",
    "double comparison greater than or less than",
    "double comparison greater than or equal and less than or equal",
    "double comparison less than or equal",
    "double comparison less than or greater than",
    "double comparison less than or equal and greater than or equal",
    "not redundant double comparison equal or greater than",
    "contradictory less than and greater than",
    "contradictory equal and less than",
    "redundant greater than or equal and less than or equal"
);

test_file!(loops, loop_match_pop_front, "simple loop match pop front");

test_file!(breaks, breaks, "Simple break", "Break inside of if", "Break inside of if with comment");

test_file!(
    bool_comparison,
    bool_comparison,
    "Comparison with true",
    "Comparison with true on LHS",
    "Comparison with false",
    "Comparison with false on LHS",
    "Negated comparison with true",
    "Negated comparison with true on LHS",
    "Negated comparison with false",
    "Negated comparison with false on LHS"
);

test_file!(
    useless_conversion,
    useless_conversion,
    "useless conversion from felt to felt with try_into",
    "useless conversion from felt252 to felt252",
    "valid conversion from felt to felt252",
    "useless conversion in complex expression",
    "valid try_into conversion",
    "nested useless conversions"
);
