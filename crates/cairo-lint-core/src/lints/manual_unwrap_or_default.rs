use cairo_lang_semantic::db::SemanticGroup;
use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_semantic::{Expr, ExprIf, ExprMatch, Arenas};

pub const MANUAL_UNWRAP_OR_DEFAULT: &str = "This can be done in one call with `.unwrap_or_default()`";

/// Enum to represent an `if let` or `match` expression.
pub enum IfLetOrMatch {
    If(ExprIf),
    Match (ExprMatch),
}

impl IfLetOrMatch {
  /// Parses and extracts the branches of an `if let` or `match` expression.
  pub fn parse_and_extract(
      expr: &Expr,
      arenas: &Arenas
  ) -> Option<(Expr, Option<Expr>)> {
      match expr {
          Expr::If(expr_if) => {
              // Extract the then Expr from the block of the if
              let then_expr = &arenas.exprs[expr_if.if_block];

              // Handle the Option<Expr> for the else block
              let else_expr = match expr_if.else_block {
                  Some(else_block_id) => {
                      let else_block_expr = &arenas.exprs[else_block_id];
                      Some(else_block_expr.clone())
                  },
                  None => None,
              };

              // Return the tuple of the then block and the optional else block
              Some((then_expr.clone(), else_expr))
          },
          Expr::Match(expr_match) => {
              // Extract the arms from `match`
              let arms = &expr_match.arms;

              // Ensure that there are exactly 2 arms
              if arms.len() == 2 {
                  let some_arm_expr = &arenas.exprs[arms[0].expression];
                  let none_arm_expr = &arenas.exprs[arms[1].expression];

                  Some((some_arm_expr.clone(), Some(none_arm_expr.clone())))
              } else {
                  None
              }
          },
          _ => None,
      }
  }
}

fn is_default_expr(
  db: &dyn SemanticGroup, 
  expr: &Expr, 
) -> bool {
  match expr {
    // Check if the expression is a function call
    Expr::FunctionCall(call_expr) => {
        let func_name = &call_expr.function.name(db);
        // Check if the function name is `Default::default` or `default`
        func_name.as_str() == "Default::default" || func_name.as_str() == "default"
    },
    // Other patterns are not considered default expressions
    _ => false,
  }
}

// / Function to check if a pattern matches a `Some` case and the other arm is `Default::default()`.
fn is_manual_unwrap_or_default(
    db: &dyn SemanticGroup, 
    some_block: &Expr, 
    none_expr: &Expr
) -> bool {
    // Add check that the `Some(x) => x` doesn't do anything apart "returning" the value in `Some`.
    // let is_some = matches!(some_block, Expr::Some(_));

    // Check if the "none" branch returns Default::default()
    let is_default = is_default_expr(db, none_expr);

    // is_some && is_default
    is_default
}

/// check rule that detects manual `unwrap_or_default` patterns.
pub fn check_manual_unwrap_or_default(
    db: &dyn SemanticGroup,
    expr: &Expr,
    diagnostics: &mut Vec<PluginDiagnostic>,
    arenas: &Arenas,
) {
    if let Some((some_block, Some(none_expr))) = IfLetOrMatch::parse_and_extract(expr, arenas) {
        if is_manual_unwrap_or_default(db, &some_block, &none_expr) {
            diagnostics.push(PluginDiagnostic {
                stable_ptr: expr.stable_ptr().into(),
                message: "This can be simplified using `.unwrap_or_default()`.".to_string(),
                severity: Severity::Warning,
            });
        }
    }
}
