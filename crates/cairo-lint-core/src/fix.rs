use cairo_lang_compiler::db::RootDatabase;
use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_filesystem::span::TextSpan;
use cairo_lang_semantic::diagnostic::SemanticDiagnosticKind;
use cairo_lang_semantic::SemanticDiagnostic;
use cairo_lang_syntax::node::ast::{
    BlockOrIf, Condition, Expr, ExprBinary, ExprIf, ExprLoop, ExprMatch, OptionElseClause,
    OptionPatternEnumInnerPattern, Pattern, Statement,
};
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::kind::SyntaxKind;
use cairo_lang_syntax::node::{SyntaxNode, TypedSyntaxNode};
use cairo_lang_utils::Upcast;
use if_chain::if_chain;
use log::debug;

use crate::lints::bool_comparison::generate_fixed_text_for_comparison;
use crate::lints::double_comparison;
use crate::lints::single_match::is_expr_unit;
use crate::plugin::{diagnostic_kind_from_message, CairoLintKind};

mod import_fixes;
pub use import_fixes::{apply_import_fixes, collect_unused_imports, ImportFix};
mod helper;
use helper::{invert_condition, remove_break_from_block, remove_break_from_else_clause};

/// Represents a fix for a diagnostic, containing the span of code to be replaced
/// and the suggested replacement.
#[derive(Debug, Clone)]
pub struct Fix {
    pub span: TextSpan,
    pub suggestion: String,
}

fn indent_snippet(input: &str, initial_indentation: usize) -> String {
    let mut indented_code = String::new();
    let mut indentation_level = initial_indentation;
    let indent = "    "; // 4 spaces for each level of indentation
    let mut lines = input.split('\n').peekable();
    while let Some(line) = lines.next() {
        let trim = line.trim();
        // Decrease indentation level if line starts with a closing brace
        if trim.starts_with('}') && indentation_level > 0 {
            indentation_level -= 1;
        }

        // Add current indentation level to the line
        if !trim.is_empty() {
            indented_code.push_str(&indent.repeat(indentation_level));
        }
        indented_code.push_str(trim);
        if lines.peek().is_some() {
            indented_code.push('\n');
        }
        // Increase indentation level if line ends with an opening brace
        if trim.ends_with('{') {
            indentation_level += 1;
        }
    }

    indented_code
}

/// Attempts to fix a semantic diagnostic.
///
/// This function is the entry point for fixing semantic diagnostics. It examines the
/// diagnostic kind and delegates to specific fix functions based on the diagnostic type.
///
/// # Arguments
///
/// * `db` - A reference to the RootDatabase
/// * `diag` - A reference to the SemanticDiagnostic to be fixed
///
/// # Returns
///
/// An `Option<(SyntaxNode, String)>` where the `SyntaxNode` represents the node to be
/// replaced, and the `String` is the suggested replacement. Returns `None` if no fix
/// is available for the given diagnostic.
pub fn fix_semantic_diagnostic(db: &RootDatabase, diag: &SemanticDiagnostic) -> Option<(SyntaxNode, String)> {
    match diag.kind {
        SemanticDiagnosticKind::PluginDiagnostic(ref plugin_diag) => Fixer.fix_plugin_diagnostic(db, plugin_diag),
        SemanticDiagnosticKind::UnusedImport(_) => {
            debug!("Unused imports should be handled in preemptively");
            None
        }
        _ => {
            debug!("No fix available for diagnostic: {:?}", diag.kind);
            None
        }
    }
}

#[derive(Default)]
pub struct Fixer;
impl Fixer {
    /// Fixes a destructuring match by converting it to an if-let expression.
    ///
    /// This method handles matches with two arms, where one arm is a wildcard (_)
    /// and the other is either an enum or struct pattern.
    ///
    /// # Arguments
    ///
    /// * `db` - A reference to the SyntaxGroup
    /// * `node` - The SyntaxNode representing the match expression
    ///
    /// # Returns
    ///
    /// A `String` containing the if-let expression that replaces the match.
    ///
    /// # Panics
    ///
    /// Panics if the diagnostic is incorrect (i.e., the match doesn't have the expected structure).
    pub fn fix_destruct_match(&self, db: &dyn SyntaxGroup, node: SyntaxNode) -> Option<(SyntaxNode, String)> {
        let match_expr = ExprMatch::from_syntax_node(db, node.clone());
        let arms = match_expr.arms(db).elements(db);
        let first_arm = &arms[0];
        let second_arm = &arms[1];
        let (pattern, first_expr) =
            match (&first_arm.patterns(db).elements(db)[0], &second_arm.patterns(db).elements(db)[0]) {
                (Pattern::Underscore(_), Pattern::Enum(pat)) => (pat.as_syntax_node(), second_arm),
                (Pattern::Enum(pat), Pattern::Underscore(_)) => (pat.as_syntax_node(), first_arm),
                (Pattern::Underscore(_), Pattern::Struct(pat)) => (pat.as_syntax_node(), second_arm),
                (Pattern::Struct(pat), Pattern::Underscore(_)) => (pat.as_syntax_node(), first_arm),
                (Pattern::Enum(pat1), Pattern::Enum(pat2)) => {
                    if is_expr_unit(second_arm.expression(db), db) {
                        (pat1.as_syntax_node(), first_arm)
                    } else {
                        (pat2.as_syntax_node(), second_arm)
                    }
                }
                (_, _) => panic!("Incorrect diagnostic"),
            };
        let mut pattern_span = pattern.span(db);
        pattern_span.end = pattern.span_start_without_trivia(db);
        let indent = node.get_text(db).chars().take_while(|c| c.is_whitespace()).collect::<String>();
        let trivia = pattern.clone().get_text_of_span(db, pattern_span);
        Some((
            node,
            indent_snippet(
                &format!(
                    "{trivia}{indent}if let {} = {} {{\n{}\n}}",
                    pattern.get_text_without_trivia(db),
                    match_expr.expr(db).as_syntax_node().get_text_without_trivia(db),
                    first_expr.expression(db).as_syntax_node().get_text_without_trivia(db),
                ),
                indent.len() / 4,
            ),
        ))
    }

    /// Fixes a plugin diagnostic by delegating to the appropriate Fixer method.
    ///
    /// # Arguments
    ///
    /// * `db` - A reference to the RootDatabase
    /// * `diag` - A reference to the SemanticDiagnostic
    /// * `plugin_diag` - A reference to the PluginDiagnostic
    ///
    /// # Returns
    ///
    /// An `Option<(SyntaxNode, String)>` containing the node to be replaced and the
    /// suggested replacement.
    pub fn fix_plugin_diagnostic(
        &self,
        db: &RootDatabase,
        plugin_diag: &PluginDiagnostic,
    ) -> Option<(SyntaxNode, String)> {
        match diagnostic_kind_from_message(&plugin_diag.message) {
            CairoLintKind::DoubleParens => {
                self.fix_double_parens(db.upcast(), plugin_diag.stable_ptr.lookup(db.upcast()))
            }
            CairoLintKind::DestructMatch => self.fix_destruct_match(db, plugin_diag.stable_ptr.lookup(db.upcast())),
            CairoLintKind::DoubleComparison => {
                self.fix_double_comparison(db.upcast(), plugin_diag.stable_ptr.lookup(db.upcast()))
            }
            CairoLintKind::EquatableIfLet => self.fix_equatable_if_let(db, plugin_diag.stable_ptr.lookup(db.upcast())),
            CairoLintKind::BreakUnit => self.fix_break_unit(db, plugin_diag.stable_ptr.lookup(db.upcast())),
            CairoLintKind::CollapsibleIf => self.fix_collapsible_if(db, plugin_diag.stable_ptr.lookup(db.upcast())),
            CairoLintKind::BoolComparison => self.fix_bool_comparison(
                db,
                ExprBinary::from_syntax_node(db.upcast(), plugin_diag.stable_ptr.lookup(db.upcast())),
            ),
            CairoLintKind::CollapsibleIfElse => self.fix_collapsible_if_else(
                db,
                &ExprIf::from_syntax_node(db.upcast(), plugin_diag.stable_ptr.lookup(db.upcast())),
            ),
            CairoLintKind::LoopMatchPopFront => {
                self.fix_loop_match_pop_front(db, plugin_diag.stable_ptr.lookup(db.upcast()))
            }
            CairoLintKind::ManualUnwrapOrDefault => {
                self.fix_manual_unwrap_or_default(db, plugin_diag.stable_ptr.lookup(db.upcast()))
            }
            CairoLintKind::LoopForWhile => self.fix_loop_break(db.upcast(), plugin_diag.stable_ptr.lookup(db.upcast())),
            CairoLintKind::ManualOkOr => self.fix_manual_ok_or(db, plugin_diag.stable_ptr.lookup(db.upcast())),
            CairoLintKind::ManualOk => self.fix_manual_ok(db, plugin_diag.stable_ptr.lookup(db.upcast())),
            CairoLintKind::ManualErr => self.fix_manual_err(db, plugin_diag.stable_ptr.lookup(db.upcast())),
            CairoLintKind::ManualIsSome => self.fix_manual_is_some(db, plugin_diag.stable_ptr.lookup(db.upcast())),
            CairoLintKind::ManualExpect => self.fix_manual_expect(db, plugin_diag.stable_ptr.lookup(db.upcast())),
            CairoLintKind::ManualExpectErr => {
                self.fix_manual_expect_err(db, plugin_diag.stable_ptr.lookup(db.upcast()))
            }
            CairoLintKind::ManualIsNone => self.fix_manual_is_none(db, plugin_diag.stable_ptr.lookup(db.upcast())),
            CairoLintKind::ManualIsOk => self.fix_manual_is_ok(db, plugin_diag.stable_ptr.lookup(db.upcast())),
            CairoLintKind::ManualIsErr => self.fix_manual_is_err(db, plugin_diag.stable_ptr.lookup(db.upcast())),
            CairoLintKind::IntGePlusOne => self.fix_int_ge_plus_one(
                db,
                ExprBinary::from_syntax_node(db.upcast(), plugin_diag.stable_ptr.lookup(db.upcast())),
            ),
            CairoLintKind::IntGeMinOne => self.fix_int_ge_min_one(
                db,
                ExprBinary::from_syntax_node(db.upcast(), plugin_diag.stable_ptr.lookup(db.upcast())),
            ),
            CairoLintKind::IntLePlusOne => self.fix_int_le_plus_one(
                db,
                ExprBinary::from_syntax_node(db.upcast(), plugin_diag.stable_ptr.lookup(db.upcast())),
            ),
            CairoLintKind::IntLeMinOne => self.fix_int_le_min_one(
                db,
                ExprBinary::from_syntax_node(db.upcast(), plugin_diag.stable_ptr.lookup(db.upcast())),
            ),
            _ => None,
        }
    }

    /// Rewrites `break ();` as `break;` given the node text contains it.
    pub fn fix_break_unit(&self, db: &dyn SyntaxGroup, node: SyntaxNode) -> Option<(SyntaxNode, String)> {
        Some((node.clone(), node.get_text(db).replace("break ();", "break;").to_string()))
    }

    /// Rewrites a bool comparison to a simple bool. Ex: `some_bool == false` would be rewritten to
    /// `!some_bool`
    pub fn fix_bool_comparison(&self, db: &dyn SyntaxGroup, node: ExprBinary) -> Option<(SyntaxNode, String)> {
        let lhs = node.lhs(db).as_syntax_node().get_text(db);
        let rhs = node.rhs(db).as_syntax_node().get_text(db);

        let result = generate_fixed_text_for_comparison(db, lhs.as_str(), rhs.as_str(), node.clone());
        Some((node.as_syntax_node(), result))
    }

    /// Rewrites this:
    ///
    /// ```ignore
    /// loop {
    ///     match some_span.pop_front() {
    ///         Option::Some(val) => do_smth(val),
    ///         Option::None => break;
    ///     }
    /// }
    /// ```
    /// to this:
    /// ```ignore
    /// for val in span {
    ///     do_smth(val);
    /// };
    /// ```
    pub fn fix_loop_match_pop_front(&self, db: &dyn SyntaxGroup, node: SyntaxNode) -> Option<(SyntaxNode, String)> {
        let expr_loop = ExprLoop::from_syntax_node(db, node.clone());
        let body = expr_loop.body(db);
        let Statement::Expr(expr) = &body.statements(db).elements(db)[0] else {
            panic!("Wrong statement type. This is probably a bug in the lint detection. Please report it")
        };
        let Expr::Match(expr_match) = expr.expr(db) else {
            panic!("Wrong expression type. This is probably a bug in the lint detection. Please report it")
        };
        let val = expr_match.expr(db);
        let span_name = match val {
            Expr::FunctionCall(func_call) => func_call.arguments(db).arguments(db).elements(db)[0]
                .arg_clause(db)
                .as_syntax_node()
                .get_text_without_trivia(db),
            Expr::Binary(dot_call) => dot_call.lhs(db).as_syntax_node().get_text_without_trivia(db),
            _ => panic!("Wrong expressiin type. This is probably a bug in the lint detection. Please report it"),
        };
        let mut elt_name = "".to_owned();
        let mut some_arm = "".to_owned();
        let arms = expr_match.arms(db).elements(db);

        let mut loop_span = node.span(db);
        loop_span.end = node.span_start_without_trivia(db);
        let indent = node.get_text(db).chars().take_while(|c| c.is_whitespace()).collect::<String>();
        let trivia = node.clone().get_text_of_span(db, loop_span);
        let trivia = if trivia.is_empty() { trivia } else { format!("{indent}{trivia}\n") };
        for arm in arms {
            if_chain! {
                if let Pattern::Enum(enum_pattern) = &arm.patterns(db).elements(db)[0];
                if let OptionPatternEnumInnerPattern::PatternEnumInnerPattern(var) = enum_pattern.pattern(db);
                then {
                    elt_name = var.pattern(db).as_syntax_node().get_text_without_trivia(db);
                    some_arm = if let Expr::Block(block_expr) = arm.expression(db) {
                        block_expr.statements(db).as_syntax_node().get_text(db)
                    } else {
                        arm.expression(db).as_syntax_node().get_text(db)
                    }
                }
            }
        }
        Some((
            node,
            indent_snippet(&format!("{trivia}for {elt_name} in {span_name} {{\n{some_arm}\n}};\n"), indent.len() / 4),
        ))
    }

    /// Removes unnecessary double parentheses from a syntax node.
    ///
    /// Simplifies an expression by stripping extra layers of parentheses while preserving
    /// the original formatting and indentation.
    ///
    /// # Arguments
    ///
    /// * `db` - Reference to the `SyntaxGroup` for syntax tree access.
    /// * `node` - The `SyntaxNode` containing the expression.
    ///
    /// # Returns
    ///
    /// A `String` with the simplified expression.
    ///
    /// # Example
    ///
    /// Input: `((x + y))`
    /// Output: `x + y`
    pub fn fix_double_parens(&self, db: &dyn SyntaxGroup, node: SyntaxNode) -> Option<(SyntaxNode, String)> {
        let mut expr = Expr::from_syntax_node(db, node.clone());

        while let Expr::Parenthesized(inner_expr) = expr {
            expr = inner_expr.expr(db);
        }

        Some((
            node.clone(),
            indent_snippet(
                &expr.as_syntax_node().get_text_without_trivia(db),
                node.get_text(db).chars().take_while(|c| c.is_whitespace()).collect::<String>().len() / 4,
            ),
        ))
    }

    /// Transforms nested `if-else` statements into a more compact `if-else if` format.
    ///
    /// Simplifies an expression by converting nested `if-else` structures into a single `if-else
    /// if` statement while preserving the original formatting and indentation.
    ///
    /// # Arguments
    ///
    /// * `db` - Reference to the `SyntaxGroup` for syntax tree access.
    /// * `node` - The `SyntaxNode` containing the expression.
    ///
    /// # Returns
    ///
    /// A `String` with the refactored `if-else` structure.
    pub fn fix_collapsible_if_else(&self, db: &dyn SyntaxGroup, if_expr: &ExprIf) -> Option<(SyntaxNode, String)> {
        let OptionElseClause::ElseClause(else_clause) = if_expr.else_clause(db) else {
            return None;
        };
        if let BlockOrIf::Block(block_expr) = else_clause.else_block_or_if(db) {
            if let Some(Statement::Expr(statement_expr)) = block_expr.statements(db).elements(db).first() {
                if let Expr::If(if_expr) = statement_expr.expr(db) {
                    // Construct the new "else if" expression
                    let condition = if_expr.condition(db).as_syntax_node().get_text(db);
                    let if_body = if_expr.if_block(db).as_syntax_node().get_text(db);
                    let else_body = if_expr.else_clause(db).as_syntax_node().get_text(db);

                    // Preserve original indentation
                    let original_indent = else_clause
                        .as_syntax_node()
                        .get_text(db)
                        .chars()
                        .take_while(|c| c.is_whitespace())
                        .collect::<String>();

                    return Some((
                        else_clause.as_syntax_node(),
                        format!("{}else if {} {} {}", original_indent, condition, if_body, else_body),
                    ));
                }
            }
        }

        // If we can't transform it, return the original text
        None
    }

    /// Rewrites a double comparison. Ex: `a > b || a == b` to `a >= b`
    pub fn fix_double_comparison(&self, db: &dyn SyntaxGroup, node: SyntaxNode) -> Option<(SyntaxNode, String)> {
        let expr = Expr::from_syntax_node(db, node.clone());

        if let Expr::Binary(binary_op) = expr {
            let lhs = binary_op.lhs(db);
            let rhs = binary_op.rhs(db);
            let middle_op = binary_op.op(db);

            if let (Some(lhs_op), Some(rhs_op)) = (
                double_comparison::extract_binary_operator_expr(&lhs, db),
                double_comparison::extract_binary_operator_expr(&rhs, db),
            ) {
                let simplified_op = double_comparison::determine_simplified_operator(&lhs_op, &rhs_op, &middle_op);

                if let Some(simplified_op) = simplified_op {
                    if let Some(operator_to_replace) = double_comparison::operator_to_replace(lhs_op) {
                        let lhs_text = lhs.as_syntax_node().get_text(db).replace(operator_to_replace, simplified_op);
                        return Some((node, lhs_text.to_string()));
                    }
                }
            }
        }

        None
    }

    /// Rewrites a useless `if let` to a simple `if`
    pub fn fix_equatable_if_let(&self, db: &dyn SyntaxGroup, node: SyntaxNode) -> Option<(SyntaxNode, String)> {
        let expr = ExprIf::from_syntax_node(db, node.clone());
        let condition = expr.condition(db);

        let fixed_condition = match condition {
            Condition::Let(condition_let) => {
                format!(
                    "{} == {} ",
                    condition_let.expr(db).as_syntax_node().get_text_without_trivia(db),
                    condition_let.patterns(db).as_syntax_node().get_text_without_trivia(db),
                )
            }
            _ => panic!("Incorrect diagnostic"),
        };

        Some((
            node,
            format!(
                "{}{}{}",
                expr.if_kw(db).as_syntax_node().get_text(db),
                fixed_condition,
                expr.if_block(db).as_syntax_node().get_text(db),
            ),
        ))
    }
    /// Rewrites manual unwrap or default to use unwrap_or_default
    pub fn fix_manual_unwrap_or_default(&self, db: &dyn SyntaxGroup, node: SyntaxNode) -> Option<(SyntaxNode, String)> {
        // Check if the node is a general expression
        let expr = Expr::from_syntax_node(db, node.clone());

        let matched_expr = match expr {
            // Handle the case where the expression is a match expression
            Expr::Match(expr_match) => expr_match.expr(db).as_syntax_node(),

            // Handle the case where the expression is an if-let expression
            Expr::If(expr_if) => {
                // Extract the condition from the if-let expression
                let condition = expr_if.condition(db);

                match condition {
                    Condition::Let(condition_let) => {
                        // Extract and return the syntax node for the matched expression
                        condition_let.expr(db).as_syntax_node()
                    }
                    _ => panic!("Expected an `if let` expression."),
                }
            }
            // Handle unsupported expressions
            _ => panic!("The expression cannot be simplified to `.unwrap_or_default()`."),
        };

        let indent = node.get_text(db).chars().take_while(|c| c.is_whitespace()).collect::<String>();
        Some((node, format!("{indent}{}.unwrap_or_default()", matched_expr.get_text_without_trivia(db))))
    }
    /// Converts a `loop` with a conditionally-breaking `if` statement into a `while` loop.
    ///
    /// This function transforms loops that have a conditional `if` statement
    /// followed by a `break` into a `while` loop, which can simplify the logic
    /// and improve readability.
    ///
    /// # Arguments
    ///
    /// * `db` - Reference to the `SyntaxGroup` for syntax tree access.
    /// * `node` - The `SyntaxNode` representing the loop expression.
    ///
    /// # Returns
    ///
    /// A `String` containing the transformed loop as a `while` loop, preserving
    /// the original formatting and indentation.
    ///
    /// # Example
    ///
    /// ```
    /// let mut x = 0;
    /// loop {
    ///     if x > 5 {
    ///         break;
    ///     }
    ///     x += 1;
    /// }
    /// ```
    ///
    /// Would be converted to:
    ///
    /// ```
    /// let mut x = 0;
    /// while x <= 5 {
    ///     x += 1;
    /// }
    /// ```
    pub fn fix_loop_break(&self, db: &dyn SyntaxGroup, node: SyntaxNode) -> Option<(SyntaxNode, String)> {
        let loop_expr = ExprLoop::from_syntax_node(db, node.clone());
        let indent = node.get_text(db).chars().take_while(|c| c.is_whitespace()).collect::<String>();
        let mut condition_text = String::new();
        let mut loop_body = String::new();

        if let Some(Statement::Expr(expr_statement)) = loop_expr.body(db).statements(db).elements(db).first() {
            if let Expr::If(if_expr) = expr_statement.expr(db) {
                condition_text = invert_condition(&if_expr.condition(db).as_syntax_node().get_text_without_trivia(db));

                loop_body.push_str(&remove_break_from_block(db, if_expr.if_block(db), &indent));

                if let OptionElseClause::ElseClause(else_clause) = if_expr.else_clause(db) {
                    loop_body.push_str(&remove_break_from_else_clause(db, else_clause, &indent));
                }
            }
        }

        for statement in loop_expr.body(db).statements(db).elements(db).iter().skip(1) {
            loop_body.push_str(&format!("{}    {}\n", indent, statement.as_syntax_node().get_text_without_trivia(db)));
        }

        Some((node, format!("{}while {} {{\n{}{}}}\n", indent, condition_text, loop_body, indent)))
    }

    /// Rewrites a manual implementation of ok_or
    pub fn fix_manual_ok_or(&self, db: &dyn SyntaxGroup, node: SyntaxNode) -> Option<(SyntaxNode, String)> {
        let fix = match node.kind(db) {
            SyntaxKind::ExprMatch => {
                let expr_match = ExprMatch::from_syntax_node(db, node.clone());

                let (option_var_name, none_arm_err) = expr_match_get_var_name_and_err(expr_match, db, 1);

                format!("{option_var_name}.ok_or({none_arm_err})")
            }
            SyntaxKind::ExprIf => {
                let expr_if = ExprIf::from_syntax_node(db, node.clone());

                let (option_var_name, err) = expr_if_get_var_name_and_err(expr_if, db);

                format!("{option_var_name}.ok_or({err})")
            }
            _ => panic!("SyntaxKind should be either ExprIf or ExprMatch"),
        };
        Some((node, fix))
    }

    /// Rewrites a manual implementation of is_some
    pub fn fix_manual_is_some(&self, db: &dyn SyntaxGroup, node: SyntaxNode) -> Option<(SyntaxNode, String)> {
        Some((node.clone(), fix_manual("is_some", db, node)))
    }

    // Rewrites a manual implementation of is_none
    pub fn fix_manual_is_none(&self, db: &dyn SyntaxGroup, node: SyntaxNode) -> Option<(SyntaxNode, String)> {
        Some((node.clone(), fix_manual("is_none", db, node)))
    }

    /// Rewrites a manual implementation of is_ok
    pub fn fix_manual_is_ok(&self, db: &dyn SyntaxGroup, node: SyntaxNode) -> Option<(SyntaxNode, String)> {
        Some((node.clone(), fix_manual("is_ok", db, node)))
    }

    /// Rewrites a manual implementation of is_err
    pub fn fix_manual_is_err(&self, db: &dyn SyntaxGroup, node: SyntaxNode) -> Option<(SyntaxNode, String)> {
        Some((node.clone(), fix_manual("is_err", db, node)))
    }

    /// Rewrites a manual implementation of ok
    pub fn fix_manual_ok(&self, db: &dyn SyntaxGroup, node: SyntaxNode) -> Option<(SyntaxNode, String)> {
        Some((node.clone(), fix_manual("ok", db, node)))
    }

    /// Rewrites a manual implementation of err
    pub fn fix_manual_err(&self, db: &dyn SyntaxGroup, node: SyntaxNode) -> Option<(SyntaxNode, String)> {
        Some((node.clone(), fix_manual("err", db, node)))
    }

    /// Rewrites a manual implementation of expect
    pub fn fix_manual_expect(&self, db: &dyn SyntaxGroup, node: SyntaxNode) -> Option<(SyntaxNode, String)> {
        let fix = match node.kind(db) {
            SyntaxKind::ExprMatch => {
                let expr_match = ExprMatch::from_syntax_node(db, node.clone());

                let (option_var_name, none_arm_err) = expr_match_get_var_name_and_err(expr_match, db, 1);

                format!("{option_var_name}.expect({none_arm_err})")
            }
            SyntaxKind::ExprIf => {
                let expr_if = ExprIf::from_syntax_node(db, node.clone());

                let (option_var_name, err) = expr_if_get_var_name_and_err(expr_if, db);

                format!("{option_var_name}.expect({err})")
            }
            _ => panic!("SyntaxKind should be either ExprIf or ExprMatch"),
        };
        Some((node, fix))
    }

    /// Attempts to fix a collapsible if-statement by combining its conditions.
    /// This function detects nested `if` statements where the inner `if` can be collapsed
    /// into the outer one by combining their conditions with `&&`. It reconstructs the
    /// combined condition and the inner block, preserving the indentation and formatting.
    ///
    /// # Arguments
    ///
    /// * `db` - A reference to the `SyntaxGroup`, which provides access to the syntax tree.
    /// * `node` - A `SyntaxNode` representing the outer `if` statement that might be collapsible.
    ///
    /// # Returns
    ///
    /// A `String` containing the fixed code with the combined conditions if a collapsible
    /// `if` is found. If no collapsible `if` is detected, the original text of the node is
    /// returned.
    pub fn fix_collapsible_if(&self, db: &dyn SyntaxGroup, node: SyntaxNode) -> Option<(SyntaxNode, String)> {
        let expr_if = ExprIf::from_syntax_node(db, node.clone());
        let outer_condition = expr_if.condition(db).as_syntax_node().get_text_without_trivia(db);
        let if_block = expr_if.if_block(db);

        let statements = if_block.statements(db).elements(db);
        if statements.len() != 1 {
            return None;
        }

        if let Some(Statement::Expr(inner_expr_stmt)) = statements.first() {
            if let Expr::If(inner_if_expr) = inner_expr_stmt.expr(db) {
                match inner_if_expr.else_clause(db) {
                    OptionElseClause::Empty(_) => {}
                    OptionElseClause::ElseClause(_) => {
                        return None;
                    }
                }

                match expr_if.else_clause(db) {
                    OptionElseClause::Empty(_) => {}
                    OptionElseClause::ElseClause(_) => {
                        return None;
                    }
                }

                let inner_condition = inner_if_expr.condition(db).as_syntax_node().get_text_without_trivia(db);
                let combined_condition = format!("({}) && ({})", outer_condition, inner_condition);
                let inner_if_block = inner_if_expr.if_block(db).as_syntax_node().get_text(db);

                let indent =
                    expr_if.if_kw(db).as_syntax_node().get_text(db).chars().take_while(|c| c.is_whitespace()).count();
                return Some((
                    node,
                    indent_snippet(&format!("if {} {}", combined_condition, inner_if_block,), indent / 4),
                ));
            }
        }
        None
    }

    /// Rewrites a manual implementation of expect err
    pub fn fix_manual_expect_err(&self, db: &dyn SyntaxGroup, node: SyntaxNode) -> Option<(SyntaxNode, String)> {
        let fix = match node.kind(db) {
            SyntaxKind::ExprMatch => {
                let expr_match = ExprMatch::from_syntax_node(db, node.clone());

                let (option_var_name, none_arm_err) = expr_match_get_var_name_and_err(expr_match, db, 0);

                format!("{option_var_name}.expect_err({none_arm_err})")
            }
            SyntaxKind::ExprIf => {
                let expr_if = ExprIf::from_syntax_node(db, node.clone());

                let (option_var_name, err) = expr_if_get_var_name_and_err(expr_if, db);

                format!("{option_var_name}.expect_err({err})")
            }
            _ => panic!("SyntaxKind should be either ExprIf or ExprMatch"),
        };
        Some((node, fix))
    }

    /// Rewrites a manual implementation of int ge plus one x >= y + 1
    pub fn fix_int_ge_plus_one(&self, db: &dyn SyntaxGroup, node: ExprBinary) -> Option<(SyntaxNode, String)> {
        let lhs = node.lhs(db).as_syntax_node().get_text(db);

        let Expr::Binary(rhs_exp) = node.rhs(db) else { panic!("should be addition") };
        let rhs = rhs_exp.lhs(db).as_syntax_node().get_text(db);

        let fix = format!("{} > {} ", lhs.trim(), rhs.trim());
        Some((node.as_syntax_node(), fix))
    }

    /// Rewrites a manual implementation of int ge min one x - 1 >= y
    pub fn fix_int_ge_min_one(&self, db: &dyn SyntaxGroup, node: ExprBinary) -> Option<(SyntaxNode, String)> {
        let Expr::Binary(lhs_exp) = node.lhs(db) else { panic!("should be substraction") };
        let rhs = node.rhs(db).as_syntax_node().get_text(db);

        let lhs = lhs_exp.lhs(db).as_syntax_node().get_text(db);

        let fix = format!("{} > {} ", lhs.trim(), rhs.trim());
        Some((node.as_syntax_node(), fix))
    }

    /// Rewrites a manual implementation of int le plus one x + 1 <= y
    pub fn fix_int_le_plus_one(&self, db: &dyn SyntaxGroup, node: ExprBinary) -> Option<(SyntaxNode, String)> {
        let Expr::Binary(lhs_exp) = node.lhs(db) else { panic!("should be addition") };
        let rhs = node.rhs(db).as_syntax_node().get_text(db);

        let lhs = lhs_exp.lhs(db).as_syntax_node().get_text(db);

        let fix = format!("{} < {} ", lhs.trim(), rhs.trim());
        Some((node.as_syntax_node(), fix))
    }

    /// Rewrites a manual implementation of int le min one x <= y -1
    pub fn fix_int_le_min_one(&self, db: &dyn SyntaxGroup, node: ExprBinary) -> Option<(SyntaxNode, String)> {
        let lhs = node.lhs(db).as_syntax_node().get_text(db);

        let Expr::Binary(rhs_exp) = node.rhs(db) else { panic!("should be substraction") };
        let rhs = rhs_exp.lhs(db).as_syntax_node().get_text(db);

        let fix = format!("{} < {} ", lhs.trim(), rhs.trim());
        Some((node.as_syntax_node(), fix))
    }
}

fn expr_match_get_var_name_and_err(expr_match: ExprMatch, db: &dyn SyntaxGroup, arm_index: usize) -> (String, String) {
    let option_var_name = expr_match.expr(db).as_syntax_node().get_text_without_trivia(db);

    let arms = expr_match.arms(db).elements(db);
    if arms.len() != 2 {
        panic!("Expected exactly two arms in the match expression");
    }

    if arm_index > 1 {
        panic!("Invalid arm index. Expected 0 for first arm or 1 for second arm.");
    }

    let Expr::FunctionCall(func_call) = &arms[arm_index].expression(db) else {
        panic!("Expected a function call expression");
    };

    let args = func_call.arguments(db).arguments(db).elements(db);
    let arg = args.first().expect("Should have arg");

    let none_arm_err = arg.as_syntax_node().get_text_without_trivia(db).to_string();

    (option_var_name, none_arm_err)
}

fn expr_if_get_var_name_and_err(expr_if: ExprIf, db: &dyn SyntaxGroup) -> (String, String) {
    let Condition::Let(condition_let) = expr_if.condition(db) else {
        panic!("Expected a ConditionLet condition");
    };
    let option_var_name = condition_let.expr(db).as_syntax_node().get_text_without_trivia(db);

    let OptionElseClause::ElseClause(else_clause) = expr_if.else_clause(db) else {
        panic!("Expected a non-empty else clause");
    };

    let BlockOrIf::Block(expr_block) = else_clause.else_block_or_if(db) else {
        panic!("Expected a BlockOrIf block in else clause");
    };

    let Statement::Expr(statement_expr) = expr_block.statements(db).elements(db)[0].clone() else {
        panic!("Expected a StatementExpr statement");
    };

    let Expr::FunctionCall(func_call) = statement_expr.expr(db) else {
        panic!("Expected a function call expression");
    };

    let args = func_call.arguments(db).arguments(db).elements(db);
    let arg = args.first().expect("Should have arg");
    let err = arg.as_syntax_node().get_text_without_trivia(db).to_string();

    (option_var_name, err)
}

pub fn fix_manual(func_name: &str, db: &dyn SyntaxGroup, node: SyntaxNode) -> String {
    match node.kind(db) {
        SyntaxKind::ExprMatch => {
            let expr_match = ExprMatch::from_syntax_node(db, node.clone());

            let option_var_name = expr_match.expr(db).as_syntax_node().get_text_without_trivia(db);

            format!("{option_var_name}.{func_name}()")
        }
        SyntaxKind::ExprIf => {
            let expr_if = ExprIf::from_syntax_node(db, node.clone());

            let var_name = if let Condition::Let(condition_let) = expr_if.condition(db) {
                condition_let.expr(db).as_syntax_node().get_text_without_trivia(db)
            } else {
                panic!("Expected an ConditionLet condition")
            };

            format!("{var_name}.{func_name}()")
        }
        _ => panic!("SyntaxKind should be either ExprIf or ExprMatch"),
    }
}
