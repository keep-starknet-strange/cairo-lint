use std::ops::Deref;

use cairo_lang_defs::ids::{ModuleId, ModuleItemId};
use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_semantic::db::SemanticGroup;
use cairo_lang_semantic::plugin::{AnalyzerPlugin, PluginSuite};
use cairo_lang_syntax::node::ast::{Expr, ExprMatch, Pattern, Statement};
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::kind::SyntaxKind;
use cairo_lang_syntax::node::{TypedStablePtr, TypedSyntaxNode};
use crate::erasing_op::EraseOp;
use cairo_lang_syntax::node::ast::ExprBinary;

pub fn cairo_lint_plugin_suite() -> PluginSuite {
    let mut suite = PluginSuite::default();
    suite.add_analyzer_plugin::<CairoLint>();
    suite.add_analyzer_plugin::<EraseOp>();
    suite
}
#[derive(Debug, Default)]
pub struct CairoLint;

#[derive(Debug, PartialEq)]
pub enum CairoLintKind {
    IfLet,
    If,
    EraseOp,
    Unknown,
}

pub fn diagnostic_kind_from_message(message: &str) -> CairoLintKind {
    match message {
        CairoLint::IF_LET => CairoLintKind::IfLet,
        CairoLint::IF => CairoLintKind::If,
        "operation can be simplifield to zero" => CairoLintKind::EraseOp,
        _ => CairoLintKind::Unknown,
    }
}

impl CairoLint {
    const IF_LET: &'static str =
        "you seem to be trying to use `match` for destructuring a single pattern. Consider using `if let`";
    const IF: &'static str = "you seem to be trying to use `match` for an equality check. Consider using `if`";

    pub fn check_match(&self, db: &dyn SyntaxGroup, match_expr: &ExprMatch, diagnostics: &mut Vec<PluginDiagnostic>) {
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
                        is_single_armed = tuple_expr.is_some_and(|list| list.expressions(db).elements(db).is_empty());
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
        };
        match (is_single_armed, is_destructuring) {
            (true, false) => diagnostics.push(PluginDiagnostic {
                stable_ptr: match_expr.stable_ptr().untyped(),
                message: Self::IF.to_string(),
                severity: Severity::Warning,
            }),
            (true, true) => diagnostics.push(PluginDiagnostic {
                stable_ptr: match_expr.stable_ptr().untyped(),
                message: Self::IF_LET.to_string(),
                severity: Severity::Warning,
            }),
            (_, _) => (),
        }
    }
}

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
                            SyntaxKind::ExprMatch => self.check_match(
                                db.upcast(),
                                &ExprMatch::from_syntax_node(db.upcast(), descendant),
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

impl AnalyzerPlugin for EraseOp {
    fn diagnostics(&self, db: &dyn SemanticGroup, module_id: ModuleId) -> Vec<PluginDiagnostic> {
        let mut diagnostics = Vec::new();

        let items = db.module_items(module_id).unwrap_or_default();
        for item in items.iter() {
            if let ModuleItemId::FreeFunction(func_id) = item {
                let func = db.module_free_function_by_id(*func_id).unwrap().unwrap();
                let descendants = func.as_syntax_node().descendants(db.upcast());
                for descendant in descendants {
                    if let SyntaxKind::ExprBinary = descendant.kind(db.upcast()) {
                        let binary_expr = ExprBinary::from_syntax_node(db.upcast(), descendant);
                        if let Some(diagnostic) = self.check_expr(db.upcast(), &binary_expr) {
                            diagnostics.push(diagnostic);
                        }
                    }
                }
            }
        }

        diagnostics
    }
}
