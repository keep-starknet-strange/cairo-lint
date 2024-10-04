use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_semantic::db::SemanticGroup;
use cairo_lang_semantic::{Arenas, Expr, ExprFunctionCall, ExprFunctionCallArg};
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::helpers::QueryAttrs;
use cairo_lang_syntax::node::{SyntaxNode, TypedStablePtr, TypedSyntaxNode};

use super::double_comparison::function_trait_name_from_fn_id;
use super::{AND, DIV, EQ, GE, GT, LE, LT, NE, NOT, OR, SUB, XOR};

const DIV_EQ_OP: &str = "Division with identical operands, this operation always results in one (except for zero) and \
                         may indicate a logic error";
const EQ_COMP_OP: &str =
    "Comparison with identical operands, this operation always results in true and may indicate a logic error";
const NEQ_COMP_OP: &str =
    "Comparison with identical operands, this operation always results in false and may indicate a logic error";
const EQ_DIFF_OP: &str =
    "Subtraction with identical operands, this operation always results in zero and may indicate a logic error";
const EQ_BITWISE_OP: &str = "Bitwise operation with identical operands, this operation always results in the same \
                             value and may indicate a logic error";
const EQ_LOGICAL_OP: &str = "Logical operation with identical operands, this operation always results in the same \
                             value and may indicate a logic error";

pub const ALLOWED: [&str; 1] = [LINT_NAME];
const LINT_NAME: &str = "eq_op";

pub fn check_eq_op(
    db: &dyn SemanticGroup,
    expr_func: &ExprFunctionCall,
    arenas: &Arenas,
    diagnostics: &mut Vec<PluginDiagnostic>,
) {
    // Checks if the lint is allowed in any upper scope
    let mut current_node = expr_func.stable_ptr.lookup(db.upcast()).as_syntax_node();
    while let Some(node) = current_node.parent() {
        if node.has_attr_with_arg(db.upcast(), "allow", LINT_NAME) {
            return;
        }
        current_node = node;
    }

    // We're looking for binary operations
    if expr_func.args.len() != 2 {
        return;
    }
    // Get lhs syntax node to check the text
    let lhs = match &expr_func.args[0] {
        ExprFunctionCallArg::Reference(val) => val.stable_ptr(),
        ExprFunctionCallArg::Value(val) => {
            let expr = &arenas.exprs[*val];
            // If the operands are funtion calls don't lint because the function might have a side effect
            if matches!(expr, Expr::FunctionCall(_)) {
                return;
            }
            if let Expr::Snapshot(snapshot) = expr
                && matches!(arenas.exprs[snapshot.inner], Expr::FunctionCall(_))
            {
                return;
            }

            expr.stable_ptr()
        }
    }
    .lookup(db.upcast())
    .as_syntax_node();

    // Get rhs syntax node to check the text
    let rhs = match &expr_func.args[1] {
        ExprFunctionCallArg::Reference(val) => val.stable_ptr(),
        ExprFunctionCallArg::Value(val) => {
            let expr = &arenas.exprs[*val];
            // If the operands are funtion calls don't lint because the function might have a side effect
            if matches!(expr, Expr::FunctionCall(_)) {
                return;
            }
            if let Expr::Snapshot(snapshot) = expr
                && matches!(arenas.exprs[snapshot.inner], Expr::FunctionCall(_))
            {
                return;
            }

            expr.stable_ptr()
        }
    }
    .lookup(db.upcast())
    .as_syntax_node();

    let op = function_trait_name_from_fn_id(db, &expr_func.function);

    if are_operands_equal(db.upcast(), lhs, rhs) {
        if let Some(message) = get_diagnostic_message(&op) {
            diagnostics.push(PluginDiagnostic {
                stable_ptr: expr_func.stable_ptr.untyped(),
                message: message.to_owned(),
                severity: Severity::Warning,
            });
        }
    }
}

fn are_operands_equal(db: &dyn SyntaxGroup, lhs: SyntaxNode, rhs: SyntaxNode) -> bool {
    lhs.get_text_without_trivia(db) == rhs.get_text_without_trivia(db)
}

fn get_diagnostic_message(op: &str) -> Option<&'static str> {
    match op {
        EQ | LE | GE => Some(EQ_COMP_OP),
        NE | LT | GT => Some(NEQ_COMP_OP),
        AND | OR => Some(EQ_LOGICAL_OP),
        XOR | NOT => Some(EQ_BITWISE_OP),
        SUB => Some(EQ_DIFF_OP),
        DIV => Some(DIV_EQ_OP),
        _ => None,
    }
}
