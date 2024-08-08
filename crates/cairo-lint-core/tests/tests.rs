use std::cmp::Reverse;
use std::path::Path;
use std::sync::{LazyLock, Mutex};

use cairo_lang_semantic::test_utils::setup_test_crate;
use cairo_lang_test_utils::parse_test_file::{dump_to_test_file, parse_test_file, Test};
use cairo_lang_utils::ordered_hash_map::OrderedHashMap;
use cairo_lang_utils::Upcast;
use cairo_lint_core::db::AnalysisDatabase;
use cairo_lint_core::fix::{fix_semantic_diagnostic, Fix};
use cairo_lint_test_utils::{get_diags, test_file, Tests};
use ctor::dtor;
use paste::paste;
use pretty_assertions::assert_eq;
use test_case::test_case;

test_file!(unused_variables, "one unused variable", "two unused variable");

test_file!(
    destruct_if_let,
    "simple destructuring match",
    "simple destructuring match second arm",
    "simple destructuring match with scope",
    "simple destructuring match with scope second arm",
    "simple destructuring match with unit in scope",
    "simple destructuring match with unit in scope second arm",
    "nested destructuring match",
    "nested destructuring match twisted",
    "nested destructuring match twisted differently",
    "nested destructuring match second arm"
);
