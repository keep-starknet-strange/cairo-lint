use cairo_lang_syntax::node::ast::{
    BlockOrIf, Condition, Expr, ExprBlock, ExprIf, OptionElseClause, OptionPatternEnumInnerPattern, Pattern, Statement,
};
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::TypedSyntaxNode;

pub fn statement_check_func_name(statement: Statement, db: &dyn SyntaxGroup, func_names: &[&str]) -> bool {
    match statement {
        Statement::Expr(statement_expr) => {
            let expr = statement_expr.expr(db);
            if let Expr::FunctionCall(func_call) = expr {
                let func_name = func_call.path(db).as_syntax_node().get_text_without_trivia(db);
                func_names.contains(&func_name.as_str())
            } else {
                false
            }
        }
        _ => false,
    }
}

pub fn pattern_check_enum_arg(pattern: &Pattern, db: &dyn SyntaxGroup, arg_name: String) -> bool {
    match pattern {
        Pattern::Enum(enum_pattern) => {
            let enum_arg = enum_pattern.pattern(db);
            match enum_arg {
                OptionPatternEnumInnerPattern::PatternEnumInnerPattern(inner_pattern) => {
                    inner_pattern.pattern(db).as_syntax_node().get_text_without_trivia(db) == arg_name
                }
                OptionPatternEnumInnerPattern::Empty(_) => false,
            }
        }
        _ => false,
    }
}

pub fn expr_check_inner_pattern_is_if_block_statement(expr: &ExprIf, db: &dyn SyntaxGroup) -> bool {
    if let Condition::Let(condition_let) = expr.condition(db) {
        match &condition_let.patterns(db).elements(db)[0] {
            Pattern::Enum(enum_pattern) => {
                let enum_arg = enum_pattern.pattern(db);
                match enum_arg {
                    OptionPatternEnumInnerPattern::PatternEnumInnerPattern(inner_pattern) => {
                        inner_pattern.pattern(db).as_syntax_node().get_text_without_trivia(db)
                            == expr.if_block(db).statements(db).as_syntax_node().get_text_without_trivia(db)
                    }
                    OptionPatternEnumInnerPattern::Empty(_) => false,
                }
            }
            _ => false,
        }
    } else {
        false
    }
}

pub fn arm_expr_check_func_name(arm_expression: Expr, db: &dyn SyntaxGroup, func_name: &str) -> bool {
    if let Expr::FunctionCall(func_call) = arm_expression {
        func_call.path(db).as_syntax_node().get_text(db) == func_name
    } else {
        false
    }
}

pub fn get_else_expr_block(else_clause: OptionElseClause, db: &dyn SyntaxGroup) -> Option<ExprBlock> {
    match else_clause {
        OptionElseClause::Empty(_) => None,
        OptionElseClause::ElseClause(else_clause) => match else_clause.else_block_or_if(db) {
            BlockOrIf::Block(expr_block) => Some(expr_block),
            _ => None,
        },
    }
}
