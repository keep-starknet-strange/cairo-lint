use cairo_lang_syntax::node::ast::{
    BlockOrIf, Condition, Expr, ExprBlock, ExprIf, OptionElseClause, OptionPatternEnumInnerPattern, Pattern, Statement,
};
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::TypedSyntaxNode;

/// Checks if the input statement is a `FunctionCall` then checks if the function name is one of the
/// func_names input list.
///
/// # Arguments
/// * `statement` - A statement that may contain a `FunctionCall`.
/// * `db` - Reference to the `SyntaxGroup` for syntax tree access.
/// * `func_names` - A list of target function names.
///
/// # Returns
/// * `true` if the function name matches any of the input names, otherwise `false`.
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

/// Checks if the inner_pattern in the input `Pattern::Enum` matches the given argument name.
///
/// # Arguments
/// * `pattern` - The pattern to check.
/// * `db` - Reference to the `SyntaxGroup` for syntax tree access.
/// * `arg_name` - The target name.
///
/// # Returns
/// * `true` if the argument name matches, otherwise `false`.
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

/// Checks if the given `Expr::FunctionCall` expression matches the input `enum_name` and
/// verifies that the first argument of the function call corresponds to pattern enum arg.
///
/// # Arguments
/// * `expr` - The expression to check, expected to be a function call.
/// * `pattern` - The pattern to match against the function's first argument.
/// * `db` - Reference to the `SyntaxGroup` for accessing the syntax tree.
/// * `enum_name` - The name of the enum to match against the function call.
///
/// # Returns
/// * `true` if the expression is a the input `enum_name` enum and the first argument matches the
///   input pattern enum arg, otherwise `false`.
pub fn pattern_check_enum_arg_is_expression(
    expr: Expr,
    pattern: Pattern,
    db: &dyn SyntaxGroup,
    enum_name: String,
) -> bool {
    if let Expr::FunctionCall(func_call) = expr {
        if func_call.path(db).as_syntax_node().get_text(db) == enum_name {
            pattern_check_enum_arg(
                &pattern,
                db,
                func_call.arguments(db).arguments(db).elements(db)[0].as_syntax_node().get_text(db),
            )
        } else {
            false
        }
    } else {
        false
    }
}

/// Checks if the inner_pattern in the input `Pattern::Enum` matches the given expr
///
/// # Arguments
/// * `pattern` - The pattern to check.
/// * `db` - Reference to the `SyntaxGroup` for syntax tree access.
/// * `expr` - the expr.
///
/// # Returns
/// * `true` if the expr matches, otherwise `false`.
pub fn pattern_check_enum_expr(pattern: &Pattern, db: &dyn SyntaxGroup, expr: &Expr) -> bool {
    if let Pattern::Enum(enum_pattern) = pattern {
        if let OptionPatternEnumInnerPattern::PatternEnumInnerPattern(x) = enum_pattern.pattern(db) {
            let pattern_text = x.pattern(db).as_syntax_node().get_text_without_trivia(db);

            return match expr {
                Expr::Block(expr_block) => {
                    expr_block.statements(db).elements(db).first().map_or(false, |statement| {
                        statement.as_syntax_node().get_text_without_trivia(db) == pattern_text
                    })
                }
                Expr::Path(_) => expr.as_syntax_node().get_text_without_trivia(db) == pattern_text,
                _ => false,
            };
        }
    }
    false
}

/// Checks that the condition expression contains an `Enum` that contains an inner pattern that is
/// the same as the statement in the if block
///
/// # Arguments
/// * `expr` - The ExprIf expression to check.
/// * `db` - Reference to the `SyntaxGroup` for syntax tree access.
///
/// # Returns
/// * `true` if the pattern matches the if block statement, otherwise `false`.
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

/// Checks if the input `Expr` is a `FunctionCall` with the specified function name.
///
/// # Arguments
/// * `arm_expression` - The expression to check.
/// * `db` - Reference to the `SyntaxGroup` for syntax tree access.
/// * `func_name` - The target function name.
///
/// # Returns
/// * `true` if the expression is a function call and the function name matches, otherwise `false`.
pub fn arm_expr_check_func_name(arm_expression: Expr, db: &dyn SyntaxGroup, func_name: &str) -> bool {
    if let Expr::FunctionCall(func_call) = arm_expression {
        func_call.path(db).as_syntax_node().get_text(db) == func_name
    } else {
        false
    }
}

/// Retrieves the else `ExprBlock` from an `OptionElseClause` clause if it's non-empty.
///
/// # Arguments
/// * `else_clause` - The `OptionElseClause` to extract the block from.
/// * `db` - Reference to the `SyntaxGroup` for syntax tree access.
///
/// # Returns
/// * `Some(ExprBlock)` if an `else` block exists, otherwise `None`.
pub fn get_else_expr_block(else_clause: OptionElseClause, db: &dyn SyntaxGroup) -> Option<ExprBlock> {
    match else_clause {
        OptionElseClause::Empty(_) => None,
        OptionElseClause::ElseClause(else_clause) => match else_clause.else_block_or_if(db) {
            BlockOrIf::Block(expr_block) => Some(expr_block),
            _ => None,
        },
    }
}

/// Checks if the condition of the input `ExprIf` expression contains an enum pattern and contains
/// an inner pattern that matches the inner pattern of the enum in the if block.
///
/// # Arguments
/// * `expr` - The `ExprIf` expression containing the condition and the if block to check.
/// * `db` - Reference to the `SyntaxGroup` for accessing the syntax tree.
/// * `enum_name` - The name of the enum to match against the function call in the if block.
///
/// # Example
/// for :
/// if let Enum(x) = res_val {
///     EnumName(x)
/// }
/// checks that x == x
///
/// # Returns
/// * `true` if the inner pattern of the enum in the condition matches the first argument of the
///   enum `enume_name` in the statement of the if block, otherwise `false`.
pub fn expr_check_condition_enum_inner_pattern_is_if_block_enum_inner_pattern(
    expr: &ExprIf,
    db: &dyn SyntaxGroup,
    enum_name: String,
) -> bool {
    let Condition::Let(condition_let) = expr.condition(db) else {
        return false;
    };

    let Pattern::Enum(enum_pattern) = &condition_let.patterns(db).elements(db)[0] else {
        return false;
    };

    let OptionPatternEnumInnerPattern::PatternEnumInnerPattern(inner_pattern) = enum_pattern.pattern(db) else {
        return false;
    };

    let Statement::Expr(statement_expr) = expr.if_block(db).statements(db).elements(db)[0].clone() else {
        return false;
    };

    let Expr::FunctionCall(func_call) = statement_expr.expr(db) else {
        return false;
    };

    if func_call.path(db).as_syntax_node().get_text_without_trivia(db) != enum_name {
        return false;
    }

    func_call.arguments(db).arguments(db).elements(db)[0].as_syntax_node().get_text_without_trivia(db)
        == inner_pattern.pattern(db).as_syntax_node().get_text_without_trivia(db)
}

/// Checks if the input `Expr` is a default of the expr kind.
///
/// # Arguments
/// * `db` - Reference to the `SyntaxGroup` for syntax tree access.
/// * `expr` - The target expr.
///
/// # Returns
/// * `true` if the expression is a default otherwise `false`.
pub fn check_is_default(db: &dyn SyntaxGroup, expr: &Expr) -> bool {
    match expr {
        Expr::FunctionCall(func_call) => {
            let func_name = func_call.path(db).as_syntax_node().get_text_without_trivia(db);
            func_name == "Default::default" || func_name == "ArrayTrait::new"
        }
        Expr::False(expr_false) => !expr_false.boolean_value(),
        Expr::String(expr_str) => {
            if let Some(str) = expr_str.string_value(db) {
                str.is_empty()
            } else {
                false
            }
        }
        Expr::Block(expr_block) => match &expr_block.statements(db).elements(db)[0] {
            Statement::Expr(statement_expr) => check_is_default(db, &statement_expr.expr(db)),
            _ => false,
        },
        Expr::InlineMacro(expr_macro) => expr_macro.as_syntax_node().get_text_without_trivia(db) == "array![]",
        Expr::FixedSizeArray(expr_arr) => expr_arr.exprs(db).elements(db).iter().all(|expr| check_is_default(db, expr)),
        Expr::Literal(expr_literal) => expr_literal.as_syntax_node().get_text_without_trivia(db) == "0",
        Expr::Tuple(expr_tuple) => {
            expr_tuple.expressions(db).elements(db).iter().all(|expr| check_is_default(db, expr))
        }
        _ => false,
    }
}
