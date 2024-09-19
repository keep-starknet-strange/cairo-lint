use cairo_lang_compiler::db::RootDatabase;
use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_filesystem::span::TextSpan;
use cairo_lang_semantic::diagnostic::SemanticDiagnosticKind;
use cairo_lang_semantic::SemanticDiagnostic;
use cairo_lang_syntax::node::ast::{
    BlockOrIf, ElseClause, Expr, ExprBinary, ExprLoop, ExprMatch, OptionPatternEnumInnerPattern, Pattern, Statement,
};
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::{SyntaxNode, TypedSyntaxNode};
use cairo_lang_utils::Upcast;
use log::debug;

use crate::lints::bool_comparison::generate_fixed_text_for_comparison;
use crate::lints::double_comparison;
use crate::lints::erasing_op::generate_fixed_text_for_erasing_operation;
use crate::lints::single_match::is_expr_unit;
use crate::plugin::{diagnostic_kind_from_message, CairoLintKind};

mod import_fixes;
pub use import_fixes::{apply_import_fixes, collect_unused_imports, ImportFix};

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
        SemanticDiagnosticKind::UnusedVariable => Fixer.fix_unused_variable(db, diag),
        SemanticDiagnosticKind::PluginDiagnostic(ref plugin_diag) => Fixer.fix_plugin_diagnostic(db, diag, plugin_diag),
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
    /// Fixes an unused variable by prefixing it with an underscore.
    ///
    /// # Arguments
    ///
    /// * `db` - A reference to the RootDatabase
    /// * `diag` - A reference to the SemanticDiagnostic for the unused variable
    ///
    /// # Returns
    ///
    /// An `Option<(SyntaxNode, String)>` containing the node to be replaced and the
    /// suggested replacement (the variable name prefixed with an underscore).
    pub fn fix_unused_variable(&self, db: &RootDatabase, diag: &SemanticDiagnostic) -> Option<(SyntaxNode, String)> {
        let node = diag.stable_location.syntax_node(db.upcast());
        let suggestion = format!("_{}", node.get_text(db.upcast()));
        Some((node, suggestion))
    }

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
    pub fn fix_destruct_match(&self, db: &dyn SyntaxGroup, node: SyntaxNode) -> String {
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
        indent_snippet(
            &format!(
                "{trivia}{indent}if let {} = {} {{\n{}\n}}",
                pattern.get_text_without_trivia(db),
                match_expr.expr(db).as_syntax_node().get_text_without_trivia(db),
                first_expr.expression(db).as_syntax_node().get_text_without_trivia(db),
            ),
            indent.len() / 4,
        )
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
        semantic_diag: &SemanticDiagnostic,
        plugin_diag: &PluginDiagnostic,
    ) -> Option<(SyntaxNode, String)> {
        let new_text = match diagnostic_kind_from_message(&plugin_diag.message) {
            CairoLintKind::DoubleParens => {
                self.fix_double_parens(db.upcast(), plugin_diag.stable_ptr.lookup(db.upcast()))
            }
            CairoLintKind::DestructMatch => self.fix_destruct_match(db, plugin_diag.stable_ptr.lookup(db.upcast())),
            CairoLintKind::DoubleComparison => {
                self.fix_double_comparison(db.upcast(), plugin_diag.stable_ptr.lookup(db.upcast()))
            }
            CairoLintKind::BreakUnit => self.fix_break_unit(db, plugin_diag.stable_ptr.lookup(db.upcast())),
            CairoLintKind::BoolComparison => self.fix_bool_comparison(
                db,
                ExprBinary::from_syntax_node(db.upcast(), plugin_diag.stable_ptr.lookup(db.upcast())),
            ),
            CairoLintKind::ErasingOperation => self.fix_erasing_operation(),
            CairoLintKind::CollapsibleIfElse => self.fix_collapsible_if_else(
                db,
                &ElseClause::from_syntax_node(db.upcast(), plugin_diag.stable_ptr.lookup(db.upcast())),
            ),
            CairoLintKind::LoopMatchPopFront => {
                self.fix_loop_match_pop_front(db, plugin_diag.stable_ptr.lookup(db.upcast()))
            }
            _ => return None,
        };

        Some((semantic_diag.stable_location.syntax_node(db.upcast()), new_text))
    }

    pub fn fix_break_unit(&self, db: &dyn SyntaxGroup, node: SyntaxNode) -> String {
        node.get_text(db).replace("break ();", "break;").to_string()
    }

    pub fn fix_bool_comparison(&self, db: &dyn SyntaxGroup, node: ExprBinary) -> String {
        let lhs = node.lhs(db).as_syntax_node().get_text(db);
        let rhs = node.rhs(db).as_syntax_node().get_text(db);

        let result = generate_fixed_text_for_comparison(db, lhs.as_str(), rhs.as_str(), node.clone());
        result
    }
    pub fn fix_erasing_operation(&self) -> String {
        let result = generate_fixed_text_for_erasing_operation();
        result
    }
    pub fn fix_loop_match_pop_front(&self, db: &dyn SyntaxGroup, node: SyntaxNode) -> String {
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
            if let Pattern::Enum(enum_pattern) = &arm.patterns(db).elements(db)[0]
                && let OptionPatternEnumInnerPattern::PatternEnumInnerPattern(var) = enum_pattern.pattern(db)
            {
                elt_name = var.pattern(db).as_syntax_node().get_text_without_trivia(db);
                some_arm = if let Expr::Block(block_expr) = arm.expression(db) {
                    block_expr.statements(db).as_syntax_node().get_text(db)
                } else {
                    arm.expression(db).as_syntax_node().get_text(db)
                }
            }
        }
        indent_snippet(&format!("{trivia}for {elt_name} in {span_name} {{\n{some_arm}\n}};\n"), indent.len() / 4)
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
    pub fn fix_double_parens(&self, db: &dyn SyntaxGroup, node: SyntaxNode) -> String {
        let mut expr = Expr::from_syntax_node(db, node.clone());

        while let Expr::Parenthesized(inner_expr) = expr {
            expr = inner_expr.expr(db);
        }

        indent_snippet(
            &expr.as_syntax_node().get_text_without_trivia(db),
            node.get_text(db).chars().take_while(|c| c.is_whitespace()).collect::<String>().len() / 4,
        )
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
    pub fn fix_collapsible_if_else(&self, db: &dyn SyntaxGroup, else_clause: &ElseClause) -> String {
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

                    return format!("{}else if {} {} {}", original_indent, condition, if_body, else_body);
                }
            }
        }

        // If we can't transform it, return the original text
        else_clause.as_syntax_node().get_text(db)
    }

    pub fn fix_double_comparison(&self, db: &dyn SyntaxGroup, node: SyntaxNode) -> String {
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
                        return lhs_text.to_string();
                    }
                }
            }
        }

        node.get_text(db).to_string()
    }
}
