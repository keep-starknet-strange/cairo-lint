use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_semantic::db::SemanticGroup;
use cairo_lang_semantic::{Arenas, Expr, ExprFunctionCall, ExprFunctionCallArg};
use cairo_lang_syntax::node::helpers::QueryAttrs;
use cairo_lang_syntax::node::{TypedStablePtr, TypedSyntaxNode};
use num_bigint::BigInt;

use super::{function_trait_name_from_fn_id, AND};
use crate::lints::{DIV, MUL};

pub const ERASING_OPERATION: &str = "This operation results in the value being erased (e.g., multiplication by 0). \
                                     Consider replacing the entire expression with 0.";

pub const ALLOWED: [&str; 1] = [LINT_NAME];
const LINT_NAME: &str = "erasing_op";

pub fn check_erasing_operation(
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
    let func = function_trait_name_from_fn_id(db, &expr_func.function);

    let is_erasing_operation = match func.as_str() {
        MUL | AND => is_zero(&expr_func.args[0], arenas) || is_zero(&expr_func.args[1], arenas),
        DIV => is_zero(&expr_func.args[0], arenas),
        _ => false,
    };
    if is_erasing_operation {
        diagnostics.push(PluginDiagnostic {
            stable_ptr: expr_func.stable_ptr.untyped(),
            message: ERASING_OPERATION.to_string(),
            severity: Severity::Warning,
        });
    }
}
fn is_zero(arg: &ExprFunctionCallArg, arenas: &Arenas) -> bool {
    match arg {
        ExprFunctionCallArg::Value(expr) => match &arenas.exprs[*expr] {
            Expr::Literal(val) => val.value == BigInt::ZERO,
            _ => false,
        },
        _ => false,
    }
}
