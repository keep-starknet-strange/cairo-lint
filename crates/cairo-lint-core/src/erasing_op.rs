use cairo_lang_compiler::lint::{Lint, LintContext, LintPass};
use cairo_lang_syntax::node::ast::{self, BinaryOp};
use cairo_lang_diagnostics::level;



macro_rules! declare_lint{
    ($name:ident, $level:ident, $desc:expr) => {
        pub const $name: Lint = Lint {
            name: stringify!($name),
            desc: $desc,
            default_level: Level::$level,
        };
    };
}


declare_lint! {
    ERASE_OP,
    Warn,
    "detect operations that always result in zero"
}

#[derive(Default)]
pub struct EraseOp;

impl LintPass for EraseOp{
    
    fn name(&self) -> & 'static str{
        "EraseOp"
    }
    
    fn check_expr(&mut self, cx: &LintContext, expr: &ast::Expr){
        if let Some((left, right, op)) = expr.as_binary_expr(){
            if (left.is_zero_literal() || right.is_zero_literal()) && matches! (op, BinaryOp::Mul | BinaryOp::Div | BinaryOp::BitAnd) {
                cx.span_lint(ERASE_OP, expr.span(), "operation can be simplified to zero")
            }
        }
    }
}

impl EraseOp{
    pub fn new() -> Self{
        Self
    }
}

