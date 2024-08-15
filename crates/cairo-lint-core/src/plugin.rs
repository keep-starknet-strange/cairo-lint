use std::ops::Deref;
use cairo_lang_defs::ids::{ ModuleId, ModuleItemId };
use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_semantic::db::SemanticGroup;
use cairo_lang_semantic::plugin::{AnalyzerPlugin, PluginSuite};
use cairo_lang_syntax::node::ast::{Expr, ExprMatch, Pattern, Statement};
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::kind::SyntaxKind;
use cairo_lang_syntax::node::{ TypedStablePtr, TypedSyntaxNode };
use crate::lints::single_match;
use crate::lints::double_parens;

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
    DoubleParens,
    Unknown,
}

pub fn diagnostic_kind_from_message(message: &str) -> CairoLintKind {
    match message {
        single_match::DESTRUCT_MATCH => CairoLintKind::DestructMatch,
        single_match::MATCH_FOR_EQUALITY => CairoLintKind::MatchForEquality,
        double_parens::DOUBLE_PARENS => CairoLintKind::DoubleParens,
        _ => CairoLintKind::Unknown,
    }
}

<<<<<<< HEAD
=======
impl CairoLint {
    const DESTRUCT_MATCH: &'static str =
        "you seem to be trying to use `match` for destructuring a single pattern. Consider using `if let`";
    const MATCH_FOR_EQUALITY: &'static str =
        "you seem to be trying to use `match` for an equality check. Consider using `if`";
    const DOUBLE_PARENS: &'static str = "unnecessary double parentheses found. Consider removing them.";

    pub fn check_double_parens(&self, db: &dyn SyntaxGroup, expr: &Expr, diagnostics: &mut Vec<PluginDiagnostic>) {
        let span = match expr {
            Expr::Parenthesized(parenthesized_expr) => {
                if let Expr::Parenthesized(_) | Expr::Tuple(_) = parenthesized_expr.expr(db) {
                    Some(parenthesized_expr.as_syntax_node().span(db))
                } else {
                    None
                }
            }
            _ => None,
        };

        if let Some(_span) = span {
            diagnostics.push(PluginDiagnostic {
                stable_ptr: expr.stable_ptr().untyped(),
                message: Self::DOUBLE_PARENS.to_string(),
                severity: Severity::Warning,
            });
        }
    }

    pub fn check_destruct_match(
        &self,
        db: &dyn SyntaxGroup,
        match_expr: &ExprMatch,
        diagnostics: &mut Vec<PluginDiagnostic>,
    ) {
        let arms = match_expr.arms(db).deref().elements(db);
        let mut is_single_armed = false;
        let mut is_destructuring = false;
        if arms.len() == 2 {
            for arm in arms {
                let patterns = arm.patterns(db).elements(db);
                match patterns[0].clone() {
                    Pattern::Underscore(_) => {
                        let tuple_expr = match arm.expression(db) {
                            Expr::Block(block_expr) => {
                                let statements = block_expr.statements(db).elements(db);
                                if statements.is_empty() {
                                    is_single_armed = true;
                                }
                                if statements.len() == 1 {
                                    match &statements[0] {
                                        Statement::Expr(statement_expr) => {
                                            if let Expr::Tuple(tuple_expr) = statement_expr.expr(db) {
                                                Some(tuple_expr)
                                            } else {
                                                None
                                            }
                                        }
                                        _ => None,
                                    }
                                } else {
                                    None
                                }
                            }
                            Expr::Tuple(tuple_expr) => Some(tuple_expr),
                            _ => None,
                        };
                        is_single_armed = tuple_expr.is_some_and(|list| list.expressions(db).elements(db).is_empty())
                            || is_single_armed;
                    }

                    Pattern::Enum(pat) => {
                        is_destructuring = !pat.pattern(db).as_syntax_node().get_text(db).is_empty();
                    }
                    Pattern::Struct(pat) => {
                        is_destructuring = !pat.as_syntax_node().get_text(db).is_empty();
                    }
                    _ => (),
                };
            }
        }
        match (is_single_armed, is_destructuring) {
            (true, false) => diagnostics.push(PluginDiagnostic {
                stable_ptr: match_expr.stable_ptr().untyped(),
                message: Self::MATCH_FOR_EQUALITY.to_string(),
                severity: Severity::Warning,
            }),
            (true, true) => diagnostics.push(PluginDiagnostic {
                stable_ptr: match_expr.stable_ptr().untyped(),
                message: Self::DESTRUCT_MATCH.to_string(),
                severity: Severity::Warning,
            }),
            (_, _) => (),
        }
    }
}

>>>>>>> 846e943 (fix: format applied)
impl AnalyzerPlugin for CairoLint {
    fn diagnostics(&self, db: &dyn SemanticGroup, module_id: ModuleId) -> Vec<PluginDiagnostic> {
        let mut diags = Vec::new();
        let Ok(items) = db.module_items(module_id) else {
            return diags;
        };
        for item in items.iter() {
            match item {
                ModuleItemId::FreeFunction(func_id) => {
                    //
                    let func = db.module_free_function_by_id(*func_id).unwrap().unwrap();
                    let descendants = func.as_syntax_node().descendants(db.upcast());
                    for descendant in descendants.into_iter() {
                        match descendant.kind(db.upcast()) {
                            SyntaxKind::ExprMatch => single_match::check_single_match(
                                db.upcast(),
                                &ExprMatch::from_syntax_node(db.upcast(), descendant),
                                &mut diags,
                                &module_id,
                            ),
                            SyntaxKind::ExprParenthesized => double_parens::check_double_parens(
                                db.upcast(),
                                &Expr::from_syntax_node(db.upcast(), descendant),
                                &mut diags,
                            ),
                            SyntaxKind::ItemExternFunction => (),
                            _ => (),
                        }
                    }
                }
                ModuleItemId::ExternFunction(_) => (),
                _ => (),
            }
        }
        diags
    }
}
