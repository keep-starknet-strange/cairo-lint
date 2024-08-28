use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_filesystem::span::TextSpan;
use cairo_lang_semantic::diagnostic::SemanticDiagnosticKind;
use cairo_lang_semantic::SemanticDiagnostic;
use cairo_lang_syntax::node::ast::{ExprIf, Condition, ExprMatch, Pattern};
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
            CairoLintKind::EquatableIfLet => self.fix_equatable_if_let(db, diag.stable_ptr.lookup(db.upcast())),
            _ => "".to_owned(),
        }
    }

    pub fn fix_equatable_if_let(&self, db: &dyn SyntaxGroup, node: SyntaxNode) -> String {
        let expr = ExprIf::from_syntax_node(db, node.clone());
        let condition = expr.condition(db);

        let fixed_condition = match condition {
            Condition::Let(condition_let) => {

                let need_parentheses = match &condition_let.patterns(db).elements(db)[0] {
                    Pattern::Struct(_) => true,
                    _ => false
                };

                format!(
                    " {} == {}{}{} ", 
                    condition_let.expr(db).as_syntax_node().get_text_without_trivia(db),
                    if need_parentheses {"("} else {""},
                    condition_let.patterns(db).as_syntax_node().get_text_without_trivia(db),
                    if need_parentheses {")"} else {""}
                )
            },
            _ => panic!("Incorrect diagnostic")
        };

        let if_block = format!(
            "{}{}{}",
            expr.if_block(db).lbrace(db).as_syntax_node().get_text(db),
            expr.if_block(db).statements(db).as_syntax_node().get_text(db),
            expr.if_block(db).rbrace(db).as_syntax_node().get_text(db)
            );
       
        format!(
            "{}{}{}{}",
            node.get_text(db).chars().take_while(|c| c.is_whitespace()).collect::<String>(),
            expr.if_kw(db).as_syntax_node().get_text_without_trivia(db),
            fixed_condition,
            if_block,
        )
    }
}