use cairo_lang_compiler::db::RootDatabase;
use cairo_lang_defs::ids::UseId;
use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_filesystem::span::TextSpan;
use cairo_lang_semantic::diagnostic::SemanticDiagnosticKind;
use cairo_lang_semantic::SemanticDiagnostic;
use cairo_lang_syntax::node::ast::{Expr, ExprMatch, Pattern};
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::kind::SyntaxKind;
use cairo_lang_syntax::node::{SyntaxNode, TypedStablePtr, TypedSyntaxNode};
use cairo_lang_utils::Upcast;
use log::{debug, warn};

use crate::lints::single_match::is_expr_unit;
use crate::plugin::{diagnostic_kind_from_message, CairoLintKind};

/// Represents a fix for a diagnostic, containing the span of code to be replaced
/// and the suggested replacement.
#[derive(Debug, Clone)]
pub struct Fix {
    pub span: TextSpan,
    pub suggestion: String,
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
        SemanticDiagnosticKind::UnusedImport(ref id) => Fixer.fix_unused_import(db, id),
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
        let trivia = pattern.clone().get_text_of_span(db, pattern_span).trim().to_string();
        let trivia = if trivia.is_empty() { trivia } else { format!("{indent}{trivia}\n") };
        format!(
            "{trivia}{indent}if let {} = {} {{ {} }}",
            pattern.get_text_without_trivia(db),
            match_expr.expr(db).as_syntax_node().get_text_without_trivia(db),
            first_expr.expression(db).as_syntax_node().get_text_without_trivia(db),
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
            _ => return None,
        };

        Some((semantic_diag.stable_location.syntax_node(db.upcast()), new_text))
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

        format!(
            "{}{}",
            node.get_text(db).chars().take_while(|c| c.is_whitespace()).collect::<String>(),
            expr.as_syntax_node().get_text_without_trivia(db),
        )
    }

    /// Attempts to fix an unused import by removing it.
    ///
    /// This function handles both single imports and imports within a use tree.
    /// For multi-import paths, it currently does not provide a fix.
    ///
    /// # Arguments
    ///
    /// * `db` - A reference to the RootDatabase
    /// * `diag` - A reference to the SemanticDiagnostic
    /// * `id` - A reference to the UseId of the unused import
    ///
    /// # Returns
    ///
    /// An `Option<(SyntaxNode, String)>` containing the node to be removed and an empty string
    /// (indicating removal). Returns `None` for multi-import paths.
    pub fn fix_unused_import(&self, db: &RootDatabase, id: &UseId) -> Option<(SyntaxNode, String)> {
        let mut current_node = id.stable_ptr(db).lookup(db.upcast()).as_syntax_node();
        let mut path_to_remove = vec![current_node.clone()];
        let mut remove_entire_statement = true;

        while let Some(parent) = current_node.parent() {
            match parent.kind(db) {
                SyntaxKind::UsePathSingle => {
                    path_to_remove.push(parent.clone());
                    current_node = parent;
                }
                SyntaxKind::UsePathMulti => {
                    path_to_remove.push(parent.clone());
                    remove_entire_statement = false;
                    break;
                }
                SyntaxKind::ItemUse => {
                    if remove_entire_statement {
                        path_to_remove.push(parent.clone());
                    }
                    break;
                }
                _ => current_node = parent,
            }
        }

        if remove_entire_statement {
            Some((path_to_remove.last().unwrap().clone(), String::new()))
        } else {
            warn!("Autofix not supported for multi-import paths: {:?}", id);
            None
        }
    }
}
