use std::cmp::Reverse;
use std::path::Path;
use std::sync::{LazyLock, Mutex};

use cairo_lang_semantic::test_utils::setup_test_crate_ex;
use cairo_lang_test_utils::parse_test_file::{dump_to_test_file, parse_test_file, Test};
use cairo_lang_utils::ordered_hash_map::OrderedHashMap;
use cairo_lang_utils::Upcast;
use cairo_lint_core::db::AnalysisDatabase;
use cairo_lint_core::fix::{fix_semantic_diagnostic, Fix};
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
    "unused import trait"
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
    "double comparison equal and less than",
    "double comparison less than and equal",
    "double comparison equal and greater than",
    "double comparison greater than and equal",
    "double comparison less than and greater than",
    "double comparison greater than and less than",
    "double comparison less than or equal and greater than or equal",
    "double comparison greater than or equal and less than or equal"
);
