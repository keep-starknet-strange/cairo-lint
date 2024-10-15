use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_semantic::db::SemanticGroup;
use cairo_lang_semantic::{Arenas, Expr, ExprFunctionCall, ExprFunctionCallArg};
use cairo_lang_syntax::node::helpers::QueryAttrs;
use cairo_lang_syntax::node::{TypedStablePtr, TypedSyntaxNode};

pub const INT_GE_PLUS_ONE: &str = "Unnecessary add operation in integer >= comparison. Use simplified comparison.";
pub const INT_GE_MIN_ONE: &str = "Unnecessary sub operation in integer >= comparison. Use simplified comparison.";
pub const INT_LE_PLUS_ONE: &str = "Unnecessary add operation in integer <= comparison. Use simplified comparison.";
pub const INT_LE_MIN_ONE: &str = "Unnecessary sub operation in integer <= comparison. Use simplified comparison.";

pub const ALLOWED: [&str; 1] = [LINT_NAME];
const LINT_NAME: &str = "int_plus_one";

pub fn check_int_plus_one(
    db: &dyn SemanticGroup,
    expr_func: &ExprFunctionCall,
    arenas: &Arenas,
    diagnostics: &mut Vec<PluginDiagnostic>,
) {
    // Checks if the lint is allowed in an upper scope
    let mut current_node = expr_func.stable_ptr.lookup(db.upcast()).as_syntax_node();
    while let Some(node) = current_node.parent() {
        if node.has_attr_with_arg(db.upcast(), "allow", LINT_NAME) {
            return;
        }
        current_node = node;
    }

    // Check if the function call is the bool greater or equal (>=) or lower or equal (<=).
    if !expr_func.function.full_name(db).contains("core::integer::")
        || (!expr_func.function.full_name(db).contains("PartialOrd::ge")
            && !expr_func.function.full_name(db).contains("PartialOrd::le"))
    {
        return;
    }

    let lhs = &expr_func.args[0];
    let rhs = &expr_func.args[1];

    // x >= y + 1
    if check_is_variable(lhs, arenas)
        && check_is_add_or_sub_one(db, rhs, arenas, "::add")
        && expr_func.function.full_name(db).contains("::ge")
    {
        diagnostics.push(PluginDiagnostic {
            stable_ptr: expr_func.stable_ptr.untyped(),
            message: INT_GE_PLUS_ONE.to_string(),
            severity: Severity::Warning,
        });
    }

    // x - 1 >= y
    if check_is_add_or_sub_one(db, lhs, arenas, "::sub")
        && check_is_variable(rhs, arenas)
        && expr_func.function.full_name(db).contains("::ge")
    {
        diagnostics.push(PluginDiagnostic {
            stable_ptr: expr_func.stable_ptr.untyped(),
            message: INT_GE_MIN_ONE.to_string(),
            severity: Severity::Warning,
        });
    }

    // x + 1 <= y
    if check_is_add_or_sub_one(db, lhs, arenas, "::add")
        && check_is_variable(rhs, arenas)
        && expr_func.function.full_name(db).contains("::le")
    {
        diagnostics.push(PluginDiagnostic {
            stable_ptr: expr_func.stable_ptr.untyped(),
            message: INT_LE_PLUS_ONE.to_string(),
            severity: Severity::Warning,
        });
    }

    // x <= y - 1
    if check_is_variable(lhs, arenas)
        && check_is_add_or_sub_one(db, rhs, arenas, "::sub")
        && expr_func.function.full_name(db).contains("::le")
    {
        diagnostics.push(PluginDiagnostic {
            stable_ptr: expr_func.stable_ptr.untyped(),
            message: INT_LE_MIN_ONE.to_string(),
            severity: Severity::Warning,
        });
    }
}

fn check_is_variable(arg: &ExprFunctionCallArg, arenas: &Arenas) -> bool {
    if let ExprFunctionCallArg::Value(val_expr) = arg {
        let Expr::Var(_) = arenas.exprs[*val_expr] else {
            return false;
        };
    };
    true
}

fn check_is_add_or_sub_one(
    db: &dyn SemanticGroup,
    arg: &ExprFunctionCallArg,
    arenas: &Arenas,
    operation: &str,
) -> bool {
    let ExprFunctionCallArg::Value(v) = arg else {
        return false;
    };
    let Expr::FunctionCall(ref func_call) = arenas.exprs[*v] else {
        return false;
    };

    // Check is addition or substraction
    if !func_call.function.full_name(db).contains("core::integer::")
        && !func_call.function.full_name(db).contains(operation)
        || func_call.args.len() != 2
    {
        return false;
    }

    let lhs = &func_call.args[0];
    let rhs = &func_call.args[1];

    // Check lhs is var
    if let ExprFunctionCallArg::Value(v) = lhs {
        let Expr::Var(_) = arenas.exprs[*v] else {
            return false;
        };
    };

    // Check rhs is 1
    if let ExprFunctionCallArg::Value(v) = rhs {
        if let Expr::Literal(ref litteral_expr) = arenas.exprs[*v] {
            if litteral_expr.value != 1.into() {
                return false;
            }
        };
    };

    true
}
