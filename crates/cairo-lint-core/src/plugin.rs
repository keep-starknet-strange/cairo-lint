use cairo_lang_defs::ids::{FunctionWithBodyId, ModuleId, ModuleItemId};
use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_semantic::db::SemanticGroup;
use cairo_lang_semantic::items::attribute::SemanticQueryAttrs;
use cairo_lang_semantic::plugin::{AnalyzerPlugin, PluginSuite};
use cairo_lang_semantic::{Expr, Statement};
use cairo_lang_syntax::node::ast::Expr as AstExpr;
use cairo_lang_syntax::node::kind::SyntaxKind;
use cairo_lang_syntax::node::{TypedStablePtr, TypedSyntaxNode};

use crate::lints::ifs::{self, *};
use crate::lints::loops::{loop_for_while, loop_match_pop_front};
use crate::lints::manual::{self, *};
use crate::lints::{
    bitwise_for_parity_check, bool_comparison, breaks, double_comparison, double_parens, duplicate_underscore_args,
    eq_op, erasing_op, int_op_one, loops, panic, performance, single_match,
};

pub fn cairo_lint_plugin_suite() -> PluginSuite {
    let mut suite = PluginSuite::default();
    suite.add_analyzer_plugin::<CairoLint>();
    suite
}
#[derive(Debug, Default)]
pub struct CairoLint;

#[derive(Debug, PartialEq)]
pub enum CairoLintKind {
    DestructMatch,
    MatchForEquality,
    DoubleComparison,
    DoubleParens,
    EquatableIfLet,
    BreakUnit,
    BoolComparison,
    CollapsibleIfElse,
    CollapsibleIf,
    DuplicateUnderscoreArgs,
    LoopMatchPopFront,
    ManualUnwrapOrDefault,
    BitwiseForParityCheck,
    LoopForWhile,
    Unknown,
    Panic,
    ErasingOperation,
    ManualOkOr,
    ManualOk,
    ManualErr,
    ManualIsSome,
    ManualIsNone,
    ManualIsOk,
    ManualIsErr,
    ManualExpect,
    DuplicateIfCondition,
    ManualExpectErr,
    IntGePlusOne,
    IntGeMinOne,
    IntLePlusOne,
    IntLeMinOne,
}

pub fn diagnostic_kind_from_message(message: &str) -> CairoLintKind {
    match message {
        single_match::DESTRUCT_MATCH => CairoLintKind::DestructMatch,
        single_match::MATCH_FOR_EQUALITY => CairoLintKind::MatchForEquality,
        double_parens::DOUBLE_PARENS => CairoLintKind::DoubleParens,
        double_comparison::SIMPLIFIABLE_COMPARISON => CairoLintKind::DoubleComparison,
        double_comparison::REDUNDANT_COMPARISON => CairoLintKind::DoubleComparison,
        double_comparison::CONTRADICTORY_COMPARISON => CairoLintKind::DoubleComparison,
        breaks::BREAK_UNIT => CairoLintKind::BreakUnit,
        equatable_if_let::EQUATABLE_IF_LET => CairoLintKind::EquatableIfLet,
        bool_comparison::BOOL_COMPARISON => CairoLintKind::BoolComparison,
        collapsible_if_else::COLLAPSIBLE_IF_ELSE => CairoLintKind::CollapsibleIfElse,
        duplicate_underscore_args::DUPLICATE_UNDERSCORE_ARGS => CairoLintKind::DuplicateUnderscoreArgs,
        collapsible_if::COLLAPSIBLE_IF => CairoLintKind::CollapsibleIf,
        loop_match_pop_front::LOOP_MATCH_POP_FRONT => CairoLintKind::LoopMatchPopFront,
        manual_unwrap_or_default::MANUAL_UNWRAP_OR_DEFAULT => CairoLintKind::ManualUnwrapOrDefault,
        panic::PANIC_IN_CODE => CairoLintKind::Panic,
        loop_for_while::LOOP_FOR_WHILE => CairoLintKind::LoopForWhile,
        erasing_op::ERASING_OPERATION => CairoLintKind::ErasingOperation,
        manual_ok_or::MANUAL_OK_OR => CairoLintKind::ManualOkOr,
        manual_ok::MANUAL_OK => CairoLintKind::ManualOk,
        manual_err::MANUAL_ERR => CairoLintKind::ManualErr,
        bitwise_for_parity_check::BITWISE_FOR_PARITY => CairoLintKind::BitwiseForParityCheck,
        manual_is::MANUAL_IS_SOME => CairoLintKind::ManualIsSome,
        manual_is::MANUAL_IS_NONE => CairoLintKind::ManualIsNone,
        manual_is::MANUAL_IS_OK => CairoLintKind::ManualIsOk,
        manual_is::MANUAL_IS_ERR => CairoLintKind::ManualIsErr,
        manual_expect::MANUAL_EXPECT => CairoLintKind::ManualExpect,
        ifs_same_cond::DUPLICATE_IF_CONDITION => CairoLintKind::DuplicateIfCondition,
        manual_expect_err::MANUAL_EXPECT_ERR => CairoLintKind::ManualExpectErr,
        int_op_one::INT_GE_PLUS_ONE => CairoLintKind::IntGePlusOne,
        int_op_one::INT_GE_MIN_ONE => CairoLintKind::IntGeMinOne,
        int_op_one::INT_LE_PLUS_ONE => CairoLintKind::IntLePlusOne,
        int_op_one::INT_LE_MIN_ONE => CairoLintKind::IntLeMinOne,
        _ => CairoLintKind::Unknown,
    }
}

impl AnalyzerPlugin for CairoLint {
    fn declared_allows(&self) -> Vec<String> {
        vec![
            bitwise_for_parity_check::ALLOWED.as_slice(),
            bool_comparison::ALLOWED.as_slice(),
            breaks::ALLOWED.as_slice(),
            double_comparison::ALLOWED.as_slice(),
            double_parens::ALLOWED.as_slice(),
            duplicate_underscore_args::ALLOWED.as_slice(),
            eq_op::ALLOWED.as_slice(),
            erasing_op::ALLOWED.as_slice(),
            loop_for_while::ALLOWED.as_slice(),
            loops::ALLOWED.as_slice(),
            panic::ALLOWED.as_slice(),
            single_match::ALLOWED.as_slice(),
            ifs::ALLOWED.as_slice(),
            manual::ALLOWED.as_slice(),
            performance::ALLOWED.as_slice(),
            int_op_one::ALLOWED.as_slice(),
        ]
        .into_iter()
        .flatten()
        .map(ToString::to_string)
        .collect()
    }

    fn diagnostics(&self, db: &dyn SemanticGroup, module_id: ModuleId) -> Vec<PluginDiagnostic> {
        let mut diags = Vec::new();
        let syntax_db = db.upcast();
        let Ok(items) = db.module_items(module_id) else {
            return diags;
        };
        for item in &*items {
            let function_nodes = match item {
                ModuleItemId::Constant(constant_id) => {
                    constant_id.stable_ptr(db.upcast()).lookup(syntax_db).as_syntax_node()
                }
                ModuleItemId::FreeFunction(free_function_id) => {
                    let func_id = FunctionWithBodyId::Free(*free_function_id);
                    check_function(db, func_id, &mut diags);
                    free_function_id.stable_ptr(db.upcast()).lookup(syntax_db).as_syntax_node()
                }
                ModuleItemId::Impl(impl_id) => {
                    let impl_functions = db.impl_functions(*impl_id);
                    let Ok(functions) = impl_functions else {
                        continue;
                    };
                    for (_fn_name, fn_id) in functions.iter() {
                        let func_id = FunctionWithBodyId::Impl(*fn_id);
                        check_function(db, func_id, &mut diags);
                    }
                    impl_id.stable_ptr(db.upcast()).lookup(syntax_db).as_syntax_node()
                }
                _ => continue,
            }
            .descendants(syntax_db);

            for node in function_nodes {
                match node.kind(syntax_db) {
                    SyntaxKind::ExprParenthesized => double_parens::check_double_parens(
                        db.upcast(),
                        &AstExpr::from_syntax_node(db.upcast(), node.clone()),
                        &mut diags,
                    ),
                    SyntaxKind::ExprIf => {}
                    SyntaxKind::ExprMatch => {}
                    _ => continue,
                }
            }
        }
        diags
    }
}
fn check_function(db: &dyn SemanticGroup, func_id: FunctionWithBodyId, diagnostics: &mut Vec<PluginDiagnostic>) {
    if let Ok(false) = func_id.has_attr_with_arg(db, "allow", "duplicate_underscore_args") {
        duplicate_underscore_args::check_duplicate_underscore_args(
            db.function_with_body_signature(func_id).unwrap().params,
            diagnostics,
        );
    }
    let Ok(function_body) = db.function_body(func_id) else {
        return;
    };
    for (_expression_id, expression) in &function_body.arenas.exprs {
        match &expression {
            Expr::Match(expr_match) => {
                single_match::check_single_match(db, expr_match, diagnostics, &function_body.arenas);
                manual_ok_or::check_manual_ok_or(db, &function_body.arenas, expr_match, diagnostics);
                manual_ok::check_manual_ok(db, &function_body.arenas, expr_match, diagnostics);
                manual_err::check_manual_err(db, &function_body.arenas, expr_match, diagnostics);
                manual_is::check_manual_is(db, &function_body.arenas, expr_match, diagnostics);
                manual_expect::check_manual_expect(db, &function_body.arenas, expr_match, diagnostics);
                manual_expect_err::check_manual_expect_err(db, &function_body.arenas, expr_match, diagnostics);
                manual_unwrap_or_default::check_manual_unwrap_or_default(
                    db,
                    &function_body.arenas,
                    expr_match,
                    diagnostics,
                );
            }
            Expr::Loop(expr_loop) => {
                loop_match_pop_front::check_loop_match_pop_front(db, expr_loop, diagnostics, &function_body.arenas);
                loop_for_while::check_loop_for_while(db, expr_loop, &function_body.arenas, diagnostics);
            }
            Expr::FunctionCall(expr_func) => {
                panic::check_panic_usage(db, expr_func, diagnostics);
                bool_comparison::check_bool_comparison(db, expr_func, &function_body.arenas, diagnostics);
                int_op_one::check_int_plus_one(db, expr_func, &function_body.arenas, diagnostics);
                bitwise_for_parity_check::check_bitwise_for_parity(db, expr_func, &function_body.arenas, diagnostics);
                eq_op::check_eq_op(db, expr_func, &function_body.arenas, diagnostics);
                erasing_op::check_erasing_operation(db, expr_func, &function_body.arenas, diagnostics);
            }

            Expr::LogicalOperator(expr_logical) => {
                double_comparison::check_double_comparison(db, expr_logical, &function_body.arenas, diagnostics);
            }
            Expr::If(expr_if) => {
                equatable_if_let::check_equatable_if_let(db, expr_if, &function_body.arenas, diagnostics);
                collapsible_if_else::check_collapsible_if_else(db, expr_if, &function_body.arenas, diagnostics);
                collapsible_if::check_collapsible_if(db, expr_if, &function_body.arenas, diagnostics);
                ifs_same_cond::check_duplicate_if_condition(db, expr_if, &function_body.arenas, diagnostics);
                manual_is::check_manual_if_is(db, &function_body.arenas, expr_if, diagnostics);
                manual_expect::check_manual_if_expect(db, &function_body.arenas, expr_if, diagnostics);
                manual_ok_or::check_manual_if_ok_or(db, &function_body.arenas, expr_if, diagnostics);
                manual_ok::check_manual_if_ok(db, &function_body.arenas, expr_if, diagnostics);
                manual_err::check_manual_if_err(db, &function_body.arenas, expr_if, diagnostics);
                manual_expect_err::check_manual_if_expect_err(db, &function_body.arenas, expr_if, diagnostics);
                manual_unwrap_or_default::check_manual_if_unwrap_or_default(
                    db,
                    &function_body.arenas,
                    expr_if,
                    diagnostics,
                );
            }
            Expr::While(expr_while) => {
                performance::check_inefficient_while_comp(db, expr_while, diagnostics, &function_body.arenas)
            }
            _ => (),
        };
    }
    for (_stmt_id, stmt) in &function_body.arenas.statements {
        if let Statement::Break(stmt_break) = &stmt {
            breaks::check_break(db, stmt_break, &function_body.arenas, diagnostics)
        }
    }
}
