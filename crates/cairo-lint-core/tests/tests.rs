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
    "simple destructuring match with comment in scope",
    "comprehensive match",
    "comprehensive match allowed"
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
    "double parens in let statement allowed",
    "double parens in struct field access",
    "double parens in match arm"
);

test_file!(
    double_comparison,
    double_comparison,
    "simple double comparison allowed",
    "simple let double comparison allowed",
    "double comparison equal or greater than",
    "simplifiable comparison allowed",
    "contradictory comparison allowed",
    "redundant comparison allowed",
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
    "redundant greater than or equal and less than or equal",
    "impossible comparison",
    "every impossible comparison"
);

test_file!(
    loops,
    loop_match_pop_front,
    "simple loop match pop front",
    "simple loop match pop front with let",
    "simple loop match pop front impl path",
    "simple loop match pop front multiple dots",
    "loop match pop front with comment in some",
    "loop match pop front with comment in some allowed",
    "loop match pop front with comment in none",
    "loop match pop front with sutff in none"
);

test_file!(
    breaks,
    breaks,
    "Simple break",
    "Simple break allowed",
    "Break inside of if",
    "Break inside of if with comment"
);

test_file!(
    ifs,
    equatable_if_let,
    "simple equality cases ok",
    "complex equality destructuring if let",
    "Simple Value Pattern Matching",
    "Enum Unit Variant Pattern Matching",
    "Complex Equality Destructuring",
    "Matching With Simple Structs field",
    "Matching With Simple Structs field allowed"
);

test_file!(
    bool_comparison,
    bool_comparison,
    "Comparison with true",
    "Comparison with true on LHS",
    "Comparison with false",
    "Comparison with false allowed",
    "Comparison with false on LHS",
    "Negated comparison with true",
    "Negated comparison with true on LHS",
    "Negated comparison with false",
    "Negated comparison with false on LHS"
);

test_file!(
    erasing_operations,
    erasing_operations,
    "Multiplication by zero",
    "Division by zero",
    "Division by zero allowed",
    "Bitwise AND with zero",
    "Multiple operations",
    "Multiple bitwise operations"
);

test_file!(
    duplicate_underscore_args,
    duplicate_underscore_args,
    "duplicate underscore args allowed",
    "duplicate underscore args2",
    "duplicate underscore longer args",
    "duplicate underscore longer args2",
    "duplicate underscore longer args3",
    "duplicate underscore longer args4"
);

test_file!(
    ifs,
    collapsible_if_else,
    "Simple else if with new line",
    "Simple else if with new line allowed",
    "Simple else if without new line",
    "Multiple else if",
    "Else if with multiple statements",
    "Else if inside loop"
);

test_file!(
    eq_op,
    eq_op,
    "simple eq op",
    "simple neq op",
    "simple lt op",
    "simple gt op",
    "simple bitwise op",
    "simple bitwise op allowed",
    "simple sub op",
    "simple divide op",
    "op with method call"
);

test_file!(
    panic,
    panic,
    "Single Panic",
    "Multiple Panic",
    "Multiple Panic and other macros",
    "Empty Panic",
    "Empty Panic allowed",
    "Empty Panic function allowed",
    "Empty Panic function",
    "No Panic",
    "Panic inside function"
);

test_file!(
    loops,
    loop_for_while,
    "simple loop with break",
    "loop with comparison condition",
    "loop with negative condition",
    "loop with arithmetic condition",
    "loop with arithmetic condition allowed",
    "loop with multiple conditions",
    "loop with arithmetic condition and else block",
    "loop with multiple condition inside if block",
    "loop with arithmetic condition and second increment",
    "loop with multiple increments and comparison",
    "loop with condition depending on external variable"
);

test_file!(
    manual,
    manual_ok_or,
    "test error str",
    "test error str allowed",
    "test error enum",
    "test with comment in None",
    "test with comment in Some",
    "test match expression not a variable",
    "test manual if",
    "test manual if with additional instructions",
    "test other var",
    "test if other var"
);

test_file!(
    bitwise_for_parity_check,
    bitwise_for_parity_check,
    "with single variable",
    "with multiple variables",
    "with multiple variables allowed",
    "In a loop",
    "with conditional logic",
    "with conditional logic allowed"
);

test_file!(
    manual,
    manual_is_some,
    "test basic is some",
    "test basic is some allowed",
    "test with comment in Some",
    "test with comment in None",
    "test match expression is a function",
    "test manual if",
    "test manual if with additional instructions"
);

test_file!(
    manual,
    manual_is_none,
    "test basic is none",
    "test basic is none allowed",
    "test with comment in Some",
    "test with comment in None",
    "test match expression is a function",
    "test manual if",
    "test manual if with additional instructions"
);

test_file!(
    manual,
    manual_unwrap_or_default,
    "manual unwrap or default for if let with default",
    "manual unwrap or default for if let with empty string",
    "manual unwrap or default for if let with new",
    "manual unwrap or default for if let with zero integer",
    "manual unwrap or default for if let with fixed array",
    "manual unwrap or default for if let with tuple",
    "manual unwrap or default for if let with array!",
    "manual unwrap or default for if let with comments",
    "manual unwrap or default for if let with tuple without trigger",
    "manual unwrap or default for if let with different type not trigger",
    "manual unwrap or default for if let without trigger",
    "manual unwrap or default for match with tuple without trigger",
    "manual unwrap or default for match with zero integer",
    "manual unwrap or default for match with empty string",
    "manual unwrap or default for match with default",
    "manual unwrap or default for match with new",
    "manual unwrap or default for match with fixed array",
    "manual unwrap or default for match with tuple",
    "manual unwrap or default for match with array!",
    "manual unwrap or default for match with without trigger",
    "manual unwrap or default for match with different type not trigger",
    "manual unwrap or default for match with comments"
);

test_file!(
    manual,
    manual_expect,
    "test core::panic_with_felt252",
    "test panic_with_felt252",
    "test with enum error",
    "test with comment in Some",
    "test with comment in None",
    "test match expression is a function",
    "test manual if",
    "test manual if allowed",
    "test manual if with additional instructions",
    "test manual result if",
    "test manual match result",
    "test manual match result with unwrapped error"
);

test_file!(
    performance,
    inefficient_while_comp,
    "while loop exit condition less than",
    "while loop exit condition less than or equal",
    "while loop exit condition greater than",
    "while loop exit condition greater than or equal",
    "while loop exit condition nested"
);

test_file!(
    manual,
    manual_expect_err,
    "test basic match expect err",
    "test basic match expect err allowed",
    "test basic if expect err",
    "test match with other err",
    "test if with other err",
    "test match with function",
    "test if with function"
);

test_file!(
    manual,
    manual_is_ok,
    "test basic is ok",
    "test basic is ok allowed",
    "test match expression is a function",
    "test manual if",
    "test manual if expression is a function"
);

test_file!(
    manual,
    manual_is_err,
    "test basic is err",
    "test match expression is a function",
    "test manual if",
    "test manual if expression is a function",
    "test manual if expression is a function allowed"
);

test_file!(
    manual,
    manual_ok,
    "test basic ok",
    "test basic if ok",
    "test basic if ok allowed",
    "test other var",
    "test if other var"
);

test_file!(
    manual,
    manual_err,
    "test basic err",
    "test basic err allowed",
    "test basic if err",
    "test other err",
    "test if other err"
);

test_file!(
    ifs,
    collapsible_if,
    "collapsible if in boolean conditions",
    "collapsible if in boolean conditions allowed",
    "collapsible if with combinable conditions",
    "collapsible if in conditions with complex expressions",
    "collapsible if with function calls",
    "collapsible if with simple numerical conditions",
    "collapsible if with else clause",
    "collapsible if with independent statement",
    "collapsible if with else on outer if"
);

test_file!(
    ifs,
    ifs_same_cond,
    "Same condition with else",
    "Same condition with boolean",
    "Same condition with felt252",
    "Same condition with struct",
    "Same condition with multiple if else",
    "Similar conditions",
    "Combined conditions with different if",
    "if with functions",
    "Greater lesser comparison",
    "Same conditions with literals and vars",
    "Same conditions with literals"
);
