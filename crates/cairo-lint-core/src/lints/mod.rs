use cairo_lang_defs::ids::{FunctionWithBodyId, TopLevelLanguageElementId};
use cairo_lang_semantic::db::SemanticGroup;
use cairo_lang_semantic::FunctionId;

pub mod bitwise_for_parity_check;
pub mod bool_comparison;
pub mod breaks;
pub mod collapsible_if;
pub mod double_comparison;
pub mod double_parens;
pub mod duplicate_underscore_args;
pub mod eq_op;
pub mod erasing_op;
pub mod ifs;
pub mod loop_for_while;
pub mod loops;
pub mod manual;
pub mod panic;
pub mod single_match;

pub(crate) const LE: &str = "core::traits::PartialOrd::le";
pub(crate) const GE: &str = "core::traits::PartialOrd::ge";
pub(crate) const LT: &str = "core::traits::PartialOrd::lt";
pub(crate) const GT: &str = "core::traits::PartialOrd::gt";
pub(crate) const EQ: &str = "core::traits::PartialEq::eq";
pub(crate) const NE: &str = "core::traits::PartialEq::ne";
pub(crate) const AND: &str = "core::traits::BitAnd::bitand";
pub(crate) const OR: &str = "core::traits::BitOr::bitor";
pub(crate) const XOR: &str = "core::traits::BitXor::bitxor";
pub(crate) const NOT: &str = "core::traits::BitNot::bitnot";
pub(crate) const DIV: &str = "core::traits::Div::div";
pub(crate) const MUL: &str = "core::traits::Mul::mul";
pub(crate) const SUB: &str = "core::traits::Sub::sub";

pub(crate) fn function_trait_name_from_fn_id(db: &dyn SemanticGroup, function: &FunctionId) -> String {
    let Ok(Some(func_id)) = function.get_concrete(db).body(db) else {
        return String::new();
    };
    // Get the trait function id of the function (if there's none it means it cannot be a call to
    // a corelib trait)
    let trait_fn_id = match func_id.function_with_body_id(db) {
        FunctionWithBodyId::Impl(func) => db.impl_function_trait_function(func).unwrap(),
        FunctionWithBodyId::Trait(func) => func,
        _ => return String::new(),
    };
    // From the trait function id get the trait name and check if it's the corelib `BitAnd`
    trait_fn_id.full_path(db.upcast())
}
