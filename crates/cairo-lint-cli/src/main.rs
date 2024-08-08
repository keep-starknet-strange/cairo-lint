use cairo_lint_core::erasing_op::EraseOp;
use cairo_lang_compiler::lint::register_lints;
fn main() {
    register_lints! {
        EraseOp::new(),

    }
}
