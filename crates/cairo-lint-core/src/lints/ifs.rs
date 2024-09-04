use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_semantic::db::SemanticGroup;
use cairo_lang_semantic::{Arenas, Condition, Expr, ExprIf};
use cairo_lang_syntax::node::kind::SyntaxKind;

pub const REDUNDANT_COMPARISON: &str = "redundant comparison found. This expression always evaluates to true or false.";

pub fn check_redundant_comparison(
    db: &dyn SemanticGroup,
    if_expr: &ExprIf,
    diagnostics: &mut Vec<PluginDiagnostic>,
    arenas: &Arenas,
) {
    let Condition::BoolExpr(bool_expr) = if_expr.condition else {
        println!("\nThis condition is not a bool expr.\n");
        return;
    };
    println!("\nBOOL EXPR {:?}\n", bool_expr);

    let new_expr = &arenas.exprs[bool_expr];
    println!("\nNEW EXPR {:?}\n", new_expr);

    let Expr::FunctionCall(logical_op) = &arenas.exprs[bool_expr] else {
        println!("\nThis condition is not a logic expr.\n");
        return;
    };
    println!("\nLOGICAL OP {:?}\n", logical_op);

    let new_expr = &logical_op.args[0];

    println!("\n2 NEW EXPR {:?}\n", new_expr);

    // if left_expr == right_expr {
    //     diagnostics.push(PluginDiagnostic {
    //         stable_ptr: if_expr.stable_ptr.into(),
    //         message: REDUNDANT_COMPARISON.to_string(),
    //         severity: Severity::Warning,
    //     });
    // }
}
