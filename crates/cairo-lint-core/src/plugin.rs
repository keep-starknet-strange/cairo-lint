use cairo_lang_defs::ids::{ModuleId, ModuleItemId};
use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_semantic::db::SemanticGroup;
use cairo_lang_semantic::plugin::{AnalyzerPlugin, PluginSuite};
use cairo_lang_syntax::node::ast::{Expr, ExprLoop, ExprMatch};
use cairo_lang_syntax::node::kind::SyntaxKind;
use cairo_lang_syntax::node::TypedSyntaxNode;

use crate::lints::{double_parens, loop_for_while, single_match};

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
    LoopForWhile,
    Unknown,
}

pub fn diagnostic_kind_from_message(message: &str) -> CairoLintKind {
    match message {
        single_match::DESTRUCT_MATCH => CairoLintKind::DestructMatch,
        single_match::MATCH_FOR_EQUALITY => CairoLintKind::MatchForEquality,
        double_parens::DOUBLE_PARENS => CairoLintKind::DoubleParens,
        loop_for_while::LOOP_FOR_WHILE => CairoLintKind::LoopForWhile,
        _ => CairoLintKind::Unknown,
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
                            SyntaxKind::ExprParenthesized => double_parens::check_double_parens(
                                db.upcast(),
                                &Expr::from_syntax_node(db.upcast(), descendant),
                                &mut diags,
                            ),
                            SyntaxKind::ExprLoop => loop_for_while::check_loop_for_while(
                                db.upcast(),
                                &ExprLoop::from_syntax_node(db.upcast(), descendant),
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
