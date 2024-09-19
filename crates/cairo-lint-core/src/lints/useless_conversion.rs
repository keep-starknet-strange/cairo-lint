use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_syntax::node::ast::ExprFunctionCall;
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::ids::SyntaxStablePtrId;
use cairo_lang_syntax::node::{TypedStablePtr, TypedSyntaxNode};

pub const USELESS_CONVERSION: &str = "This conversion is unnecessary.";

pub fn check_useless_conversion(
    db: &dyn SyntaxGroup,
    function_call: &ExprFunctionCall,
    diagnostics: &mut Vec<PluginDiagnostic>,
) {
    let callee = function_call.as_syntax_node().get_text_without_trivia(db);

    if callee.ends_with(".into()") {
        let argument_type = determine_type_of_expr(function_call, db);
        let expected_type = determine_expected_type(db, function_call);

        // If the type of the expression and the expected type are the same, it's a useless conversion
        if let (Some(arg_type), Some(exp_type)) = (argument_type, expected_type) {
            if arg_type == exp_type {
                diagnostics.push(create_diagnostic(
                    USELESS_CONVERSION,
                    function_call.stable_ptr().untyped(),
                    Severity::Warning,
                ));
            }
        }
    }
}

fn determine_type_of_expr(expr: &ExprFunctionCall, db: &dyn SyntaxGroup) -> Option<String> {    
    Some(expr.as_syntax_node().get_text_without_trivia(db))
}

fn determine_expected_type(db: &dyn SyntaxGroup, function_call: &ExprFunctionCall) -> Option<String> {    
    Some("String".to_string())
}

fn create_diagnostic(message: &str, stable_ptr: SyntaxStablePtrId, severity: Severity) -> PluginDiagnostic {
    PluginDiagnostic { stable_ptr, message: message.to_string(), severity }
}
