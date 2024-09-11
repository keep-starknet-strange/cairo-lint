use std::collections::HashSet;

use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_diagnostics::Severity;
use cairo_lang_semantic::Parameter;

pub const DUPLICATE_UNDERSCORE_ARGS: &str = "duplicate arguments, having another argument having almost the same name \
                                             makes code comprehension and documentation more difficult";

pub fn check_duplicate_underscore_args(params: Vec<Parameter>, diagnostics: &mut Vec<PluginDiagnostic>) {
    let mut registered_names: HashSet<String> = HashSet::new();

    for param in params {
        let param_name = param.name.to_string();
        let stripped_name = param_name.strip_prefix('_').unwrap_or(&param_name);

        if !registered_names.insert(stripped_name.to_string()) {
            diagnostics.push(PluginDiagnostic {
                stable_ptr: param.stable_ptr.0,
                message: DUPLICATE_UNDERSCORE_ARGS.to_string(),
                severity: Severity::Warning,
            });
        }
    }
}
