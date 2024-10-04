use cairo_lang_defs::ids::NamedLanguageElementId;
use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_semantic::db::SemanticGroup;
use cairo_lang_semantic::{
    Arenas, Expr, ExprBlock, ExprId, ExprLoop, ExprMatch, Pattern, PatternEnumVariant, Statement,
};
use cairo_lang_syntax::node::helpers::QueryAttrs;
use cairo_lang_syntax::node::{TypedStablePtr, TypedSyntaxNode};

pub const LOOP_MATCH_POP_FRONT: &str =
    "you seem to be trying to use `loop` for iterating over a span. Consider using `for in`";

const SPAN_MATCH_POP_FRONT: &str = "\"SpanImpl::pop_front\"";

pub const ALLOWED: [&str; 1] = [LINT_NAME];
const LINT_NAME: &str = "loop_match_pop_front";

pub fn check_loop_match_pop_front(
    db: &dyn SemanticGroup,
    loop_expr: &ExprLoop,
    diagnostics: &mut Vec<PluginDiagnostic>,
    arenas: &Arenas,
) {
    let mut current_node = loop_expr.stable_ptr.lookup(db.upcast()).as_syntax_node();
    while let Some(node) = current_node.parent() {
        if node.has_attr_with_arg(db.upcast(), "allow", LINT_NAME) {
            return;
        }
        current_node = node;
    }

    if !loop_expr.ty.is_unit(db) {
        return;
    }
    let Expr::Block(expr_block) = &arenas.exprs[loop_expr.body] else {
        return;
    };
    if expr_block.statements.is_empty()
        && let Some(tail) = &expr_block.tail
        && let Expr::Match(expr_match) = &arenas.exprs[*tail]
        && let Expr::FunctionCall(func_call) = &arenas.exprs[expr_match.matched_expr]
        && func_call.function.name(db) == SPAN_MATCH_POP_FRONT
    {
        if !check_single_match(db, expr_match, arenas) {
            return;
        }
        diagnostics.push(PluginDiagnostic {
            stable_ptr: loop_expr.stable_ptr.into(),
            message: LOOP_MATCH_POP_FRONT.to_owned(),
            severity: Severity::Warning,
        });
        return;
    }
    if !expr_block.statements.is_empty()
        && let Statement::Expr(stmt_expr) = &arenas.statements[expr_block.statements[0]]
        && let Expr::Match(expr_match) = &arenas.exprs[stmt_expr.expr]
    {
        if !check_single_match(db, expr_match, arenas) {
            return;
        }
        let Expr::FunctionCall(func_call) = &arenas.exprs[expr_match.matched_expr] else {
            return;
        };
        if func_call.function.name(db) == SPAN_MATCH_POP_FRONT {
            diagnostics.push(PluginDiagnostic {
                stable_ptr: loop_expr.stable_ptr.into(),
                message: LOOP_MATCH_POP_FRONT.to_owned(),
                severity: Severity::Warning,
            })
        }
    }
}

const OPTION_TYPE: &str = "core::option::Option::<";
const SOME_VARIANT: &str = "Some";
const NONE_VARIANT: &str = "None";

pub fn check_single_match(db: &dyn SemanticGroup, match_expr: &ExprMatch, arenas: &Arenas) -> bool {
    let arms = &match_expr.arms;

    // Check that we're in a setup with 2 arms that return unit
    if arms.len() == 2 && match_expr.ty.is_unit(db) {
        let first_arm = &arms[0];
        let second_arm = &arms[1];
        let is_first_arm_correct = if let Some(pattern) = first_arm.patterns.first() {
            match &arenas.patterns[*pattern] {
                // If the first arm is `_ => smth` it's incorrect
                Pattern::Otherwise(_) => false,
                // Check if the variant is of type option and if it's `None` checks that it only contains `{ break; }`
                // without comments`
                Pattern::EnumVariant(enum_pat) => check_enum_pattern(db, enum_pat, arenas, first_arm.expression),
                _ => false,
            }
        } else {
            false
        };
        let is_second_arm_correct = if let Some(pattern) = second_arm.patterns.first() {
            match &arenas.patterns[*pattern] {
                // If the 2nd arm is `_ => smth`, checks that smth is `{ break; }`
                Pattern::Otherwise(_) => {
                    if let Expr::Block(expr_block) = &arenas.exprs[second_arm.expression] {
                        check_block_is_break(db, expr_block, arenas)
                    } else {
                        return false;
                    }
                }
                // Check if the variant is of type option and if it's `None` checks that it only contains `{ break; }`
                // without comments`
                Pattern::EnumVariant(enum_pat) => check_enum_pattern(db, enum_pat, arenas, second_arm.expression),
                _ => false,
            }
        } else {
            false
        };
        is_first_arm_correct && is_second_arm_correct
    } else {
        false
    }
}
fn check_enum_pattern(
    db: &dyn SemanticGroup,
    enum_pat: &PatternEnumVariant,
    arenas: &Arenas,
    arm_expression: ExprId,
) -> bool {
    // Checks that the variant is from the option type.
    if !enum_pat.ty.format(db.upcast()).starts_with(OPTION_TYPE) {
        return false;
    }
    // Check if the variant is the None variant
    if enum_pat.variant.id.name(db.upcast()) == NONE_VARIANT
    // Get the expression of the None variant and checks if it's a block expression. 
        && let Expr::Block(expr_block) = &arenas.exprs[arm_expression]
        // If it's a block expression checks that it only contains `break;`
        && check_block_is_break(db, expr_block, arenas)
    {
        true
    } else {
        enum_pat.variant.id.name(db.upcast()) == SOME_VARIANT
    }
}
/// Checks that the block only contains `break;` without comments
fn check_block_is_break(db: &dyn SemanticGroup, expr_block: &ExprBlock, arenas: &Arenas) -> bool {
    if expr_block.statements.len() == 1
        && let Statement::Break(break_stmt) = &arenas.statements[expr_block.statements[0]]
    {
        let break_node = break_stmt.stable_ptr.lookup(db.upcast()).as_syntax_node();
        // Checks that the trimmed text == the text without trivia which would mean that there is no comment
        let break_text = break_node.get_text(db.upcast()).trim().to_string();
        if break_text == break_node.get_text_without_trivia(db.upcast())
            && (break_text == "break;" || break_text == "break ();")
        {
            return true;
        }
    }
    false
}
