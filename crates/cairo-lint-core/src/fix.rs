use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_filesystem::span::TextSpan;
use cairo_lang_semantic::diagnostic::SemanticDiagnosticKind;
use cairo_lang_semantic::SemanticDiagnostic;
use cairo_lang_syntax::node::ast::{BinaryOperator, Expr, ExprBinary, ExprMatch, Pattern};
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::{SyntaxNode, TypedSyntaxNode};
use cairo_lang_utils::Upcast;

use crate::db::AnalysisDatabase;
use crate::lints::single_match::is_expr_unit;
use crate::plugin::{diagnostic_kind_from_message, CairoLintKind};

#[derive(Default)]
pub struct Fix {
    pub span: TextSpan,
    pub suggestion: String,
}

pub fn fix_semantic_diagnostic(db: &AnalysisDatabase, diag: &SemanticDiagnostic) -> String {
    match diag.kind {
        SemanticDiagnosticKind::UnusedVariable => {
            format!("_{}", diag.stable_location.syntax_node(db.upcast()).get_text(db.upcast()))
        }
        SemanticDiagnosticKind::PluginDiagnostic(ref diag) => Fixer.fix_plugin_diagnostic(db, diag),
        _ => "".to_owned(),
    }
}

#[derive(Default)]
pub struct Fixer;
impl Fixer {
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

    pub fn fix_plugin_diagnostic(&self, db: &AnalysisDatabase, diag: &PluginDiagnostic) -> String {
        match diagnostic_kind_from_message(&diag.message) {
            CairoLintKind::DestructMatch => self.fix_destruct_match(db, diag.stable_ptr.lookup(db.upcast())),
            _ => "".to_owned(),
        }
    }

    pub fn fix_double_comparison(&self, db: &dyn SyntaxGroup, node: SyntaxNode) -> String {
        let expr = Expr::from_syntax_node(db, node.clone());

        if let Expr::Binary(binary_op) = expr {
            let lhs = binary_op.lhs(db);
            let rhs = binary_op.rhs(db);

            if let (Expr::Binary(lhs_inner), Expr::Binary(rhs_inner)) = (&lhs, &rhs) {
                if let (Some(lhs_var), Some(rhs_var)) =
                    (Self::extract_variable(lhs_inner, db), Self::extract_variable(rhs_inner, db))
                {
                    if lhs_var == rhs_var {
                        if matches!(
                            (lhs_inner.op(db), rhs_inner.op(db)),
                            (BinaryOperator::EqEq(_), BinaryOperator::LT(_))
                                | (BinaryOperator::LT(_), BinaryOperator::EqEq(_))
                                | (BinaryOperator::GE(_), BinaryOperator::GT(_))
                                | (BinaryOperator::LE(_), BinaryOperator::LT(_))
                        ) {
                            let simplified_op = match (lhs_inner.op(db), rhs_inner.op(db)) {
                                (BinaryOperator::EqEq(_), BinaryOperator::LT(_))
                                | (BinaryOperator::LT(_), BinaryOperator::EqEq(_))
                                | (BinaryOperator::LE(_), BinaryOperator::LT(_))
                                | (BinaryOperator::GE(_), BinaryOperator::GT(_)) => "<=",
                                _ => return node.get_text(db).to_string(),
                            };
                            return format!(
                                "{}{} {} {}\n",
                                node.get_text(db).chars().take_while(|c| c.is_whitespace()).collect::<String>(),
                                lhs_var,
                                simplified_op,
                                lhs_inner.rhs(db).as_syntax_node().get_text_without_trivia(db)
                            );
                        }
                    }
                }
            }
        }
        node.get_text(db).to_string()
    }

    pub fn extract_variable(binary_expr: &ExprBinary, db: &dyn SyntaxGroup) -> Option<String> {
        let lhs = binary_expr.lhs(db);
        Some(lhs.as_syntax_node().get_text(db).to_string())
    }
}
