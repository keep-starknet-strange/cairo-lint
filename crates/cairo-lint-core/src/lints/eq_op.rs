use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_syntax::node::TypedSyntaxNode;
use cairo_lang_syntax::node::ast::{BinaryOperator, Expr, ExprBinary};
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::ids::SyntaxStablePtrId;

const DIV_EQ_OP: &str = "Division with identical operands, this operation always results in one (except for zero) and \
                         may indicate a logic error";
const EQ_COMP_OP: &str =
    "Comparison with identical operands, this operation always results in true and may indicate a logic error";
const EQ_DIFF_OP: &str =
    "Subtraction with identical operands, this operation always results in zero and may indicate a logic error";
const EQ_BITWISE_OP: &str = "Bitwise operation with identical operands, this operation always results in the same \
                             value and may indicate a logic error";
const EQ_LOGICAL_OP: &str = "Logical operation with identical operands, this operation always results in the same \
                             value and may indicate a logic error";

pub fn check_eq_op(db: &dyn SyntaxGroup, node: &ExprBinary, diagnostics: &mut Vec<PluginDiagnostic>) {
    let lhs = node.lhs(db);
    let op = node.op(db);
    let rhs = node.rhs(db);

    println!("is method call: {}", lhs.as_syntax_node().kind(db));
    if are_operands_equal(db, &lhs, &rhs) && !is_method_call(db, &lhs) && !is_method_call(db, &rhs) {
        if let Some(message) = get_diagnostic_message(&op) {
            diagnostics.push(create_diagnostic(node, message));
        }
    }
}

fn are_operands_equal(db: &dyn SyntaxGroup, lhs: &Expr, rhs: &Expr) -> bool {
    let lhs_text = lhs.as_syntax_node().get_text_without_trivia(db);
    println!("lhs text: {}", lhs_text);
    let rhs_text = rhs.as_syntax_node().get_text_without_trivia(db);
    lhs_text == rhs_text
}

// check if the expression is a method call: something like `foo.bar()`
fn is_method_call(db: &dyn SyntaxGroup, expr: &Expr) -> bool {
    match expr {
        Expr::Binary(e) => {
            let op = e.op(db);
            matches!(op, BinaryOperator::Dot(_) | BinaryOperator::DotDot(_))
        }
        _ => false,
    }
}

fn get_diagnostic_message(op: &BinaryOperator) -> Option<&'static str> {
    match op {
        BinaryOperator::EqEq(_) | BinaryOperator::Neq(_) => Some(EQ_COMP_OP),
        BinaryOperator::And(_) | BinaryOperator::Or(_) => Some(EQ_LOGICAL_OP),
        BinaryOperator::Xor(_) | BinaryOperator::Not(_) => Some(EQ_BITWISE_OP),
        BinaryOperator::Minus(_) => Some(EQ_DIFF_OP),
        BinaryOperator::Div(_) => Some(DIV_EQ_OP),
        _ => None,
    }
}

fn create_diagnostic(node: &ExprBinary, message: &str) -> PluginDiagnostic {
    PluginDiagnostic {
        stable_ptr: node.as_syntax_node().stable_ptr(),
        message: message.to_owned(),
        severity: Severity::Warning,
    }
}
