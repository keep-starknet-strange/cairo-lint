use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_semantic::corelib::unit_ty;
use cairo_lang_semantic::db::SemanticGroup;
use cairo_lang_semantic::{Arenas, ExprMatch, Pattern};
use cairo_lang_syntax::node::ast::{Expr as AstExpr, ExprBlock, ExprListParenthesized, Statement};
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::{TypedStablePtr, TypedSyntaxNode};

pub const DESTRUCT_MATCH: &str =
    "you seem to be trying to use `match` for destructuring a single pattern. Consider using `if let`";
pub const MATCH_FOR_EQUALITY: &str = "you seem to be trying to use `match` for an equality check. Consider using `if`";

fn is_expr_list_parenthesised_unit(expr: &ExprListParenthesized, db: &dyn SyntaxGroup) -> bool {
    expr.expressions(db).elements(db).is_empty()
}

fn is_block_expr_unit_without_comment(block_expr: &ExprBlock, db: &dyn SyntaxGroup) -> bool {
    let statements = block_expr.statements(db).elements(db);
    if statements.is_empty() {
        return true;
    }
    if statements.len() == 1
        && let Statement::Expr(statement_expr) = &statements[0]
        && let AstExpr::Tuple(tuple_expr) = statement_expr.expr(db)
    {
        let tuple_node = tuple_expr.as_syntax_node();
        if tuple_node.span(db).start != tuple_node.span_start_without_trivia(db) {
            return false;
        }
        is_expr_list_parenthesised_unit(&tuple_expr, db)
    } else {
        false
    }
}

pub fn is_expr_unit(expr: AstExpr, db: &dyn SyntaxGroup) -> bool {
    match expr {
        AstExpr::Block(block_expr) => is_block_expr_unit_without_comment(&block_expr, db),
        AstExpr::Tuple(tuple_expr) => is_expr_list_parenthesised_unit(&tuple_expr, db),
        _ => false,
    }
}

pub fn check_single_match(
    db: &dyn SemanticGroup,
    match_expr: &ExprMatch,
    diagnostics: &mut Vec<PluginDiagnostic>,
    arenas: &Arenas,
) {
    let arms = &match_expr.arms;
    let mut is_single_armed = false;
    let mut is_complete = false;
    let mut is_destructuring = false;

    if arms.len() == 2 && match_expr.ty == unit_ty(db) {
        let first_arm = &arms[0];
        let second_arm = &arms[1];
        let mut enum_len = None;
        if let Some(pattern) = first_arm.patterns.first() {
            match &arenas.patterns[*pattern] {
                Pattern::Otherwise(_) => return,
                Pattern::EnumVariant(enum_pat) => {
                    enum_len = Some(db.enum_variants(enum_pat.variant.concrete_enum_id.enum_id(db)).unwrap().len());
                    is_destructuring = true;
                }
                Pattern::Struct(_) => {
                    is_destructuring = true;
                }
                _ => (),
            };
        };
        if let Some(pattern) = second_arm.patterns.first() {
            match &arenas.patterns[*pattern] {
                Pattern::Otherwise(_) => {
                    is_complete = true;
                }
                Pattern::EnumVariant(_) => {
                    if enum_len == Some(2) {
                        is_complete = true;
                    }
                }
                _ => (),
            };

            is_single_armed =
                is_expr_unit(arenas.exprs[second_arm.expression].stable_ptr().lookup(db.upcast()), db.upcast())
                    && is_complete;
        };
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
