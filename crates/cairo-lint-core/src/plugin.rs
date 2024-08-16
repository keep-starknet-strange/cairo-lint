use cairo_lang_defs::ids::{ModuleId, ModuleItemId};
use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_semantic::db::SemanticGroup;
use cairo_lang_semantic::plugin::{AnalyzerPlugin, PluginSuite};
use cairo_lang_syntax::node::ast::{BinaryOperator, Expr, ExprMatch};
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::kind::SyntaxKind;
use cairo_lang_syntax::node::{TypedStablePtr, TypedSyntaxNode};

use crate::lints::single_match;

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
    Unknown,
}

pub fn diagnostic_kind_from_message(message: &str) -> CairoLintKind {
    match message {
        CairoLint::DESTRUCT_MATCH => CairoLintKind::DestructMatch,
        CairoLint::MATCH_FOR_EQUALITY => CairoLintKind::MatchForEquality,
        CairoLint::DOUBLE_COMPARISON => CairoLintKind::DoubleComparison,
        _ => CairoLintKind::Unknown,
    }
}

impl CairoLint {
    const DESTRUCT_MATCH: &'static str =
        "you seem to be trying to use `match` for destructuring a single pattern. Consider using `if let`";
    const MATCH_FOR_EQUALITY: &'static str =
        "you seem to be trying to use `match` for an equality check. Consider using `if`";
    const DOUBLE_COMPARISON: &'static str =
        "redundant double comparison found. Consider simplifying to a single comparison.";

    pub fn check_double_comparison(&self, db: &dyn SyntaxGroup, expr: &Expr, diagnostics: &mut Vec<PluginDiagnostic>) {
        if let Expr::Binary(binary_op) = expr {
            let lhs = binary_op.lhs(db);
            let rhs = binary_op.rhs(db);

            if let (Some(lhs_op), Some(rhs_op)) =
                (self.extract_binary_operator(&lhs, db), self.extract_binary_operator(&rhs, db))
            {
                if (matches!(lhs_op, BinaryOperator::EqEq(_)) && matches!(rhs_op, BinaryOperator::LT(_)))
                    || (matches!(lhs_op, BinaryOperator::LT(_)) && matches!(rhs_op, BinaryOperator::EqEq(_)))
                    || (matches!(lhs_op, BinaryOperator::GE(_)) && matches!(rhs_op, BinaryOperator::GT(_)))
                    || (matches!(lhs_op, BinaryOperator::LE(_)) && matches!(rhs_op, BinaryOperator::LT(_)))
                {
                    diagnostics.push(PluginDiagnostic {
                        stable_ptr: expr.stable_ptr().untyped(),
                        message: Self::DOUBLE_COMPARISON.to_string(),
                        severity: Severity::Warning,
                    });
                }
            }
        }
    }

    fn extract_binary_operator(&self, expr: &Expr, db: &dyn SyntaxGroup) -> Option<BinaryOperator> {
        if let Expr::Binary(binary_op) = expr { Some(binary_op.op(db)) } else { None }
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
                            SyntaxKind::ExprMatch => single_match::check_single_match(
                                db.upcast(),
                                &ExprMatch::from_syntax_node(db.upcast(), descendant),
                                &mut diags,
                                &module_id,
                            ),
                            SyntaxKind::ExprBinary => CairoLint::check_double_comparison(
                                &CairoLint,
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
