use cairo_lang_defs::ids::{FunctionWithBodyId, TopLevelLanguageElementId};
use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_semantic::db::SemanticGroup;
use cairo_lang_semantic::{Arenas, Expr, ExprFunctionCall, ExprFunctionCallArg};
use cairo_lang_syntax::node::helpers::QueryAttrs;
use cairo_lang_syntax::node::{TypedStablePtr, TypedSyntaxNode};
use num_bigint::BigInt;

pub const BITWISE_FOR_PARITY: &str =
    "You seem to be trying to use `&` for parity check. Consider using `DivRem::div_rem()` instead.";

pub const ALLOWED: [&str; 1] = [LINT_NAME];
const LINT_NAME: &str = "bitwise_for_parity_check";

pub fn check_bitwise_for_parity(
    db: &dyn SemanticGroup,
    expr_func: &ExprFunctionCall,
    arenas: &Arenas,
    diagnostics: &mut Vec<PluginDiagnostic>,
) {
    let mut current_node = expr_func.stable_ptr.lookup(db.upcast()).as_syntax_node();
    while let Some(node) = current_node.parent() {
        if node.has_attr_with_arg(db.upcast(), "allow", LINT_NAME) {
            return;
        }
        current_node = node;
    }
    let Ok(Some(func_id)) = expr_func.function.get_concrete(db).body(db) else {
        return;
    };
    let trait_fn_id = match func_id.function_with_body_id(db) {
        FunctionWithBodyId::Impl(func) => db.impl_function_trait_function(func).unwrap(),
        FunctionWithBodyId::Trait(func) => func,
        _ => return,
    };
    if trait_fn_id.full_path(db.upcast()) == "core::traits::BitAnd::bitand"
        && let ExprFunctionCallArg::Value(val) = expr_func.args[1]
        && let Expr::Literal(lit) = &arenas.exprs[val]
        && lit.value == BigInt::from(1u8)
    {
        diagnostics.push(PluginDiagnostic {
            stable_ptr: expr_func.stable_ptr.untyped(),
            message: BITWISE_FOR_PARITY.to_string(),
            severity: Severity::Warning,
        });
    }
}
