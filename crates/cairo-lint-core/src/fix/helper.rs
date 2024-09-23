//! # Helper Functions for Cairo Lint
//!
//! This module provides utility functions to assist in generating fixes for `if-else` conditions
//! within loops, inverting logical conditions, and processing code blocks.
//! 
//! The main tasks of this module include:
//!
//! 1. Processing block and `else` clause content, including nested `if-else` constructs.
//! 2. Inverting logical conditions to their opposite for loop and condition rewriting.
//! 3. Skipping `break` statements when processing blocks to correctly transform loops.
//!
//! These helper functions can be reused in various parts of the Cairo Lint codebase to maintain
//! consistency and modularity when working with blocks and conditions.


use cairo_lang_syntax::node::ast::{BlockOrIf, ElseClause, ExprBlock, Statement};
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::TypedSyntaxNode;

/// Processes a block of code, formatting its content and ignoring any break statements.
/// 
/// # Arguments
/// 
/// * `db` - The syntax group which provides access to the syntax tree.
/// * `block` - The expression block (ExprBlock) to be processed.
/// * `indent` - A string representing the indentation to be applied to the block's content.
/// 
/// # Returns
/// 
/// A string representing the formatted content of the block.
pub fn process_block(db: &dyn SyntaxGroup, block: ExprBlock, indent: &str) -> String {
    let mut block_body = String::new();
    for statement in block.statements(db).elements(db) {
        if !matches!(statement, Statement::Break(_)) {
            block_body.push_str(&format!(
                "{}    {}\n",
                indent,
                statement.as_syntax_node().get_text_without_trivia(db)
            ));
        }
    }
    block_body
}

/// Processes the `else` clause of an if-else statement, handling both `else if` and `else` blocks.
///
/// # Arguments
///
/// * `db` - The syntax group which provides access to the syntax tree.
/// * `else_clause` - The `ElseClause` AST node representing the else clause.
/// * `indent` - A string representing the indentation to be applied to the else clause.
///
/// # Returns
///
/// A string representing the formatted content of the else clause.
pub fn process_else_clause(db: &dyn SyntaxGroup, else_clause: ElseClause, indent: &str) -> String {
    let mut else_body = String::new();
    match else_clause.else_block_or_if(db) {
        BlockOrIf::Block(block) => {
            else_body.push_str(&process_block(db, block, indent));
        }
        BlockOrIf::If(else_if) => {
            else_body.push_str(&format!(
                "{}else if {} {{\n",
                indent,
                else_if.condition(db).as_syntax_node().get_text_without_trivia(db)
            ));
            else_body.push_str(&process_block(db, else_if.if_block(db), indent));
            else_body.push_str(&format!("{}}}\n", indent));
        }
    }
    else_body
}

/// Inverts a logical condition, swapping `&&` for `||` and vice versa.
///
/// # Arguments
///
/// * `condition` - A string representing the logical condition to invert.
///
/// # Returns
///
/// A string representing the inverted condition.
pub fn invert_condition(condition: &str) -> String {
    if condition.contains("&&") {
        condition
            .split("&&")
            .map(|part| invert_simple_condition(part.trim()))
            .collect::<Vec<_>>()
            .join(" || ")
    } else if condition.contains("||") {
        condition
            .split("||")
            .map(|part| invert_simple_condition(part.trim()))
            .collect::<Vec<_>>()
            .join(" && ")
    } else {
        invert_simple_condition(condition)
    }
}

/// Inverts a simple condition like `>=` to `<`, `==` to `!=`, etc.
///
/// # Arguments
///
/// * `condition` - A string representing a simple condition (e.g., `>=`, `==`, `!=`).
///
/// # Returns
///
/// A string representing the inverted condition.
pub fn invert_simple_condition(condition: &str) -> String {
    if condition.contains(">=") {
        condition.replace(">=", "<")
    } else if condition.contains("<=") {
        condition.replace("<=", ">")
    } else if condition.contains(">") {
        condition.replace(">", "<=")
    } else if condition.contains("<") {
        condition.replace("<", ">=")
    } else if condition.contains("==") {
        condition.replace("==", "!=")
    } else if condition.contains("!=") {
        condition.replace("!=", "==")
    } else {
        format!("!({})", condition)
    }
}
