use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_syntax::node::ast::{Condition, ConditionLet, Expr, ExprIf, OptionPatternEnumInnerPattern, Pattern};
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::helpers::QueryAttrs;
use cairo_lang_syntax::node::TypedSyntaxNode;

pub const EQUATABLE_IF_LET: &str =
    "`if let` pattern used for equatable value. Consider using a simple comparison `==` instead";

pub fn check_equatable_if_let(db: &dyn SyntaxGroup, expr: &ExprIf, diagnostics: &mut Vec<PluginDiagnostic>) {
    if let Some(node) = expr.as_syntax_node().parent()
        && node.has_attr_with_arg(db, "allow", "equatable_if_let")
    {
        return;
    }
    let condition = expr.condition(db);

    if let Condition::Let(condition_let) = condition {
        let expr_is_simple = is_simple_equality_expr(&condition_let.expr(db));
        let condition_is_simple = is_simple_equality_condition(&condition_let, db);

        if expr_is_simple && condition_is_simple {
            diagnostics.push(PluginDiagnostic {
                stable_ptr: expr.as_syntax_node().stable_ptr(),
                message: EQUATABLE_IF_LET.to_string(),
                severity: Severity::Warning,
            });
        }
    }
}

fn is_simple_equality_expr(expr: &Expr) -> bool {
    match expr {
        // Simple literals like numbers, booleans, and strings
        Expr::Literal(_) | Expr::False(_) | Expr::True(_) | Expr::ShortString(_) | Expr::String(_) => true,

        // Path expression (typically variables or constants)
        Expr::Path(_) => true,

        // If it's any other expression, it’s considered complex
        _ => false,
    }
}

fn is_simple_equality_condition(condition: &ConditionLet, db: &dyn SyntaxGroup) -> bool {
    let patterns = condition.patterns(db).elements(db);

    for pattern in patterns {
        match pattern {
            Pattern::Literal(_)
            | Pattern::False(_)
            | Pattern::True(_)
            | Pattern::ShortString(_)
            | Pattern::String(_)
            | Pattern::Path(_) => return true,

            Pattern::Enum(enum_pattern) => match enum_pattern.pattern(db) {
                OptionPatternEnumInnerPattern::Empty(_) => return true,
                OptionPatternEnumInnerPattern::PatternEnumInnerPattern(inner_pattern) => {
                    match inner_pattern.pattern(db) {
                        Pattern::Literal(_)
                        | Pattern::False(_)
                        | Pattern::True(_)
                        | Pattern::ShortString(_)
                        | Pattern::String(_) => return true,
                        _ => continue,
                    }
                }
            },
            _ => continue,
        }
    }
    false
}
