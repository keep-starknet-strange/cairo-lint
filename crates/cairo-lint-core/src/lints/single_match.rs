use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_semantic::db::SemanticGroup;
use cairo_lang_semantic::{Arenas, ExprMatch, Pattern};
use cairo_lang_syntax::node::ast::{Expr as AstExpr, ExprBlock, ExprListParenthesized, Statement};
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::helpers::QueryAttrs;
use cairo_lang_syntax::node::{TypedStablePtr, TypedSyntaxNode};
use if_chain::if_chain;

pub const DESTRUCT_MATCH: &str =
    "you seem to be trying to use `match` for destructuring a single pattern. Consider using `if let`";
pub const MATCH_FOR_EQUALITY: &str =
    "you seem to be trying to use `match` for an equality check. Consider using `if`";

pub const ALLOWED: [&str; 1] = [LINT_NAME];
const LINT_NAME: &str = "single_match";

/// Checks for matches that do something only in 1 arm and can be rewrote as an `if let`
/// ```ignore
/// let var = Option::Some(1_u32);
/// match var {
///     Option::Some(val) => do_smth(val),
///     _ => (),
/// }
/// ```
/// Which can be rewritten as
/// ```ignore
/// if let Option::Some(val) = var {
///     do_smth(val),
/// }
/// ```
pub fn check_single_match(
    db: &dyn SemanticGroup,
    match_expr: &ExprMatch,
    diagnostics: &mut Vec<PluginDiagnostic>,
    arenas: &Arenas,
) {
    // Checks if the lint is allowed in an upper scope.
    let mut current_node = match_expr.stable_ptr.lookup(db.upcast()).as_syntax_node();
    while let Some(node) = current_node.parent() {
        if node.has_attr_with_arg(db.upcast(), "allow", LINT_NAME) {
            return;
        }
        current_node = node;
    }

    let arms = &match_expr.arms;
    let mut is_single_armed = false;
    let mut is_complete = false;
    let mut is_destructuring = false;

    // If the match isn't of unit type it means that both branches return something so it can't be a
    // single match
    if arms.len() != 2 || !match_expr.ty.is_unit(db) {
        return;
    }

    let first_arm = &arms[0];
    let second_arm = &arms[1];
    let mut enum_len = None;
    if let Some(pattern) = first_arm.patterns.first() {
        match &arenas.patterns[*pattern] {
            // If the first arm is `_ => ...` the enum is wrong
            Pattern::Otherwise(_) => return,
            // Get the number of variants in the enum to know if it's comprehensive or not
            Pattern::EnumVariant(enum_pat) => {
                enum_len = Some(
                    db.enum_variants(enum_pat.variant.concrete_enum_id.enum_id(db))
                        .unwrap()
                        .len(),
                );
                // If there's an enum pattern it's a destructuring match
                is_destructuring = enum_pat.inner_pattern.is_some();
            }
            Pattern::Struct(_) => {
                // If it's a struct pattern it's a destructuring match
                is_destructuring = true;
            }
            _ => (),
        };
    };
    if let Some(pattern) = second_arm.patterns.first() {
        match &arenas.patterns[*pattern] {
            // If the second arm is `_ => ...` the match is comprehensive
            Pattern::Otherwise(_) => {
                is_complete = true;
            }
            Pattern::EnumVariant(_) => {
                // And if the 2nd arm is an enum variant check that the number of variants in the enum is 2.
                if enum_len == Some(2) {
                    is_complete = true;
                }
            }
            _ => (),
        };

        // Checks that the second arm doesn't do anything
        is_single_armed = is_expr_unit(
            arenas.exprs[second_arm.expression]
                .stable_ptr()
                .lookup(db.upcast()),
            db.upcast(),
        ) && is_complete;
    };

    match (is_single_armed, is_destructuring) {
        (true, false) => diagnostics.push(PluginDiagnostic {
            stable_ptr: match_expr.stable_ptr.into(),
            message: MATCH_FOR_EQUALITY.to_string(),
            severity: Severity::Warning,
        }),
        (true, true) => diagnostics.push(PluginDiagnostic {
            stable_ptr: match_expr.stable_ptr.into(),
            message: DESTRUCT_MATCH.to_string(),
            severity: Severity::Warning,
        }),
        (_, _) => (),
    }
}

/// Is a tuple expression the unit type.
fn is_expr_list_parenthesised_unit(expr: &ExprListParenthesized, db: &dyn SyntaxGroup) -> bool {
    expr.expressions(db).elements(db).is_empty()
}

/// Is the block empty `{}` or `{ () }` but it shouldn't contain a comment.
fn is_block_expr_unit_without_comment(block_expr: &ExprBlock, db: &dyn SyntaxGroup) -> bool {
    let statements = block_expr.statements(db).elements(db);
    // Check if the block is empty and there's no comment in it
    if statements.is_empty()
        && block_expr
            .rbrace(db)
            .leading_trivia(db)
            .node
            .get_text(db)
            .trim()
            .is_empty()
    {
        return true;
    }

    // If there's statement checks that it's `()` without comment
    if_chain! {
        if statements.len() == 1;
        if let Statement::Expr(statement_expr) = &statements[0];
        if let AstExpr::Tuple(tuple_expr) = statement_expr.expr(db);
        then {
            let tuple_node = tuple_expr.as_syntax_node();
            if tuple_node.span(db).start != tuple_node.span_start_without_trivia(db) {
                return false;
            }
            return is_expr_list_parenthesised_unit(&tuple_expr, db);
        }
    }
    false
}

/// Checks that either the expression is `()` or `{ }` or `{ () }` but none of them should contain a
/// comment.
pub fn is_expr_unit(expr: AstExpr, db: &dyn SyntaxGroup) -> bool {
    match expr {
        AstExpr::Block(block_expr) => is_block_expr_unit_without_comment(&block_expr, db),
        AstExpr::Tuple(tuple_expr) => is_expr_list_parenthesised_unit(&tuple_expr, db),
        _ => false,
    }
}
