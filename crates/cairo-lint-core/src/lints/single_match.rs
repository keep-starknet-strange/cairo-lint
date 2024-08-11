use cairo_lang_defs::ids::{FileIndex, ModuleFileId, ModuleId};
use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::{DiagnosticsBuilder, Severity};
use cairo_lang_semantic::db::SemanticGroup;
use cairo_lang_semantic::diagnostic::NotFoundItemType;
use cairo_lang_semantic::expr::inference::InferenceId;
use cairo_lang_semantic::resolve::{AsSegments, ResolvedGenericItem, Resolver};
use cairo_lang_syntax::node::ast::{Expr, ExprBlock, ExprListParenthesized, ExprMatch, Pattern, Statement};
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::{TypedStablePtr, TypedSyntaxNode};
use cairo_lang_utils::try_extract_matches;

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
        && let Expr::Tuple(tuple_expr) = statement_expr.expr(db)
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

pub fn is_expr_unit(expr: Expr, db: &dyn SyntaxGroup) -> bool {
    match expr {
        Expr::Block(block_expr) => is_block_expr_unit_without_comment(&block_expr, db),
        Expr::Tuple(tuple_expr) => is_expr_list_parenthesised_unit(&tuple_expr, db),
        _ => false,
    }
}

pub fn check_single_match(
    db: &dyn SemanticGroup,
    match_expr: &ExprMatch,
    diagnostics: &mut Vec<PluginDiagnostic>,
    module_id: &ModuleId,
) {
    let syntax_db = db.upcast();
    let arms = match_expr.arms(syntax_db).elements(syntax_db);
    let mut is_single_armed = false;
    let mut is_complete = false;
    let mut is_destructuring = false;
    if arms.len() == 2 {
        let first_arm = &arms[0];
        let second_arm = &arms[1];
        let mut enum_len = None;
        let mut is_first_arm_unit = false;
        if let Some(pattern) = first_arm.patterns(syntax_db).elements(syntax_db).first() {
            match pattern {
                Pattern::Underscore(_) => return,
                Pattern::Enum(pat) => {
                    let mut diagnostics = DiagnosticsBuilder::default();
                    let path = pat.path(syntax_db).to_segments(syntax_db);
                    let item = Resolver::new(db, ModuleFileId(*module_id, FileIndex(0)), InferenceId::NoContext)
                        .resolve_generic_path(&mut diagnostics, path, NotFoundItemType::Identifier)
                        .unwrap();
                    let generic_variant = try_extract_matches!(item, ResolvedGenericItem::Variant).unwrap();
                    enum_len = Some(db.enum_variants(generic_variant.enum_id).unwrap().len());
                    is_destructuring = true;
                }
                Pattern::Struct(_) => {
                    is_destructuring = true;
                }
                _ => (),
            }
            is_first_arm_unit = is_expr_unit(first_arm.expression(syntax_db), syntax_db)
        };
        if let Some(pattern) = second_arm.patterns(syntax_db).elements(syntax_db).first() {
            match pattern {
                Pattern::Underscore(_) => {
                    is_complete = true;
                }
                Pattern::Enum(_) => {
                    if enum_len == Some(2) {
                        is_complete = true;
                    }
                }
                _ => (),
            }
            is_single_armed =
                is_expr_unit(second_arm.expression(syntax_db), syntax_db) && is_complete || is_first_arm_unit;
        };
    };
    match (is_single_armed, is_destructuring) {
        (true, false) => diagnostics.push(PluginDiagnostic {
            stable_ptr: match_expr.stable_ptr().untyped(),
            message: MATCH_FOR_EQUALITY.to_string(),
            severity: Severity::Warning,
        }),
        (true, true) => diagnostics.push(PluginDiagnostic {
            stable_ptr: match_expr.stable_ptr().untyped(),
            message: DESTRUCT_MATCH.to_string(),
            severity: Severity::Warning,
        }),
        (_, _) => (),
    }
}
