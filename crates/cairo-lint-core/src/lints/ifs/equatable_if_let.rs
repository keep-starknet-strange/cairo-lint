use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_semantic::db::SemanticGroup;
use cairo_lang_semantic::{Arenas, Condition, Expr, ExprIf, Pattern, PatternId};
use cairo_lang_syntax::node::helpers::QueryAttrs;
use cairo_lang_syntax::node::{TypedStablePtr, TypedSyntaxNode};

pub const EQUATABLE_IF_LET: &str =
    "`if let` pattern used for equatable value. Consider using a simple comparison `==` instead";
pub(super) const LINT_NAME: &str = "equatable_if_let";

pub fn check_equatable_if_let(
    db: &dyn SemanticGroup,
    expr: &ExprIf,
    arenas: &Arenas,
    diagnostics: &mut Vec<PluginDiagnostic>,
) {
    let mut current_node = expr.stable_ptr.lookup(db.upcast()).as_syntax_node();
    while let Some(node) = current_node.parent() {
        if node.has_attr_with_arg(db.upcast(), "allow", LINT_NAME) {
            return;
        }
        current_node = node;
    }

    if let Condition::Let(condition_let, patterns) = &expr.condition {
        let expr_is_simple = is_simple_equality_expr(&arenas.exprs[*condition_let]);
        let condition_is_simple = is_simple_equality_condition(patterns, arenas);

        if expr_is_simple && condition_is_simple {
            diagnostics.push(PluginDiagnostic {
                stable_ptr: expr.stable_ptr.untyped(),
                message: EQUATABLE_IF_LET.to_string(),
                severity: Severity::Warning,
            });
        }
    }
}

fn is_simple_equality_expr(expr: &Expr) -> bool {
    match expr {
        // Simple literals and variables
        Expr::Literal(_) | Expr::StringLiteral(_) | Expr::Var(_) => true,
        _ => false,
    }
}

fn is_simple_equality_condition(patterns: &[PatternId], arenas: &Arenas) -> bool {
    for pattern in patterns {
        match &arenas.patterns[*pattern] {
            Pattern::Literal(_) | Pattern::StringLiteral(_) => return true,
            Pattern::EnumVariant(pat) => {
                return pat.inner_pattern.is_none_or(|pat_id| {
                    matches!(arenas.patterns[pat_id], Pattern::Literal(_) | Pattern::StringLiteral(_))
                })
            }
            _ => continue,
        }
    }
    false
}
