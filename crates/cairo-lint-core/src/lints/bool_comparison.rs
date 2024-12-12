use cairo_lang_defs::ids::TopLevelLanguageElementId;
use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_semantic::db::SemanticGroup;
use cairo_lang_semantic::{Arenas, Expr, ExprFunctionCall, ExprFunctionCallArg};
use cairo_lang_syntax::node::ast::ExprBinary;
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::helpers::QueryAttrs;
use cairo_lang_syntax::node::kind::SyntaxKind;
use cairo_lang_syntax::node::{TypedStablePtr, TypedSyntaxNode};
use if_chain::if_chain;

pub const BOOL_COMPARISON: &str = "Unnecessary comparison with a boolean value. Use the variable directly.";

pub const ALLOWED: [&str; 1] = [LINT_NAME];
const LINT_NAME: &str = "bool_comparison";

/// Generates the fixed boolean for a boolean comparison. It will transform `x == false` to `!x`
pub fn generate_fixed_text_for_comparison(db: &dyn SyntaxGroup, lhs: &str, rhs: &str, node: ExprBinary) -> String {
    let op_kind = node.op(db).as_syntax_node().kind(db);
    let lhs = lhs.trim();
    let rhs = rhs.trim();

    match (lhs, rhs, op_kind) {
        // lhs
        ("false", _, SyntaxKind::TerminalEqEq | SyntaxKind::TokenEqEq) => format!("!{} ", rhs),
        ("true", _, SyntaxKind::TerminalEqEq | SyntaxKind::TokenEqEq) => format!("{} ", rhs),
        ("false", _, SyntaxKind::TerminalNeq) => format!("!{} ", rhs),
        ("true", _, SyntaxKind::TerminalNeq) => format!("!{} ", rhs),

        // rhs
        (_, "false", SyntaxKind::TerminalEqEq | SyntaxKind::TokenEqEq) => format!("!{} ", lhs),
        (_, "true", SyntaxKind::TerminalEqEq | SyntaxKind::TokenEqEq) => format!("{} ", lhs),
        (_, "false", SyntaxKind::TerminalNeq) => format!("!{} ", lhs),
        (_, "true", SyntaxKind::TerminalNeq) => format!("!{} ", lhs),

        _ => node.as_syntax_node().get_text(db).to_string(),
    }
}

/// Checks for ` a == true`. Bool comparisons are useless and can be rewritten more clearly.
pub fn check_bool_comparison(
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
    // Check if the function call is the bool partial eq function (==).
    if !expr_func.function.full_name(db).contains("core::BoolPartialEq::") {
        return;
    }
    // Extract the args of the function call. This function expects snapshots hence we need to
    // destructure that. Also the boolean type in cairo is an enum hence the enum ctor.
    for arg in &expr_func.args {
        if_chain! {
          if let ExprFunctionCallArg::Value(expr) = arg;
          if let Expr::Snapshot(snap) = &arenas.exprs[*expr];
          if let Expr::EnumVariantCtor(enum_var) = &arenas.exprs[snap.inner];
          if enum_var.variant.concrete_enum_id.enum_id(db).full_path(db.upcast()) == "core::bool";
          then {
            diagnostics.push(PluginDiagnostic {
              stable_ptr: expr_func.stable_ptr.untyped(),
              message: BOOL_COMPARISON.to_string(),
              severity: Severity::Warning,
          });
          }
        }
    }
}
