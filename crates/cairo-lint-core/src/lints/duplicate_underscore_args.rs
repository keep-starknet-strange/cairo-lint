use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_semantic::Parameter;
use cairo_lang_diagnostics::Severity;
use std::collections::HashSet;
// use cairo_lang_syntax::node::ids::SyntaxStablePtrId;

pub const DUPLICATE_UNDERSCORE_ARGS: &str = "duplicate arguments, having another argument having almost the same name makes code comprehension and documentation more difficult";

pub fn check_duplicate_underscore_args(params: Vec<Parameter>, diagnostics: &mut Vec<PluginDiagnostic>) {
    let mut registered_names: HashSet<String> = HashSet::new();

    for param in params {
        let param_name = param.name.to_string();

        if let Some(stripped_name) = param_name.strip_prefix('_') {
            if registered_names.contains(stripped_name) {
                println!("{:?}", stripped_name);
                diagnostics.push(PluginDiagnostic {
                    stable_ptr: param.stable_ptr.0,
                    message: DUPLICATE_UNDERSCORE_ARGS.to_string(),
                    severity: Severity::Warning,
                });
            } else {
                registered_names.insert(stripped_name.to_string());
            }
        } else {
            registered_names.insert(param_name.clone());
        }
    }
}
