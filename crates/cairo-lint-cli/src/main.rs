pub mod helpers;

use std::cmp::Reverse;
use std::collections::HashMap;
use std::path::PathBuf;

use annotate_snippets::Renderer;
use anyhow::{anyhow, Result};
use cairo_lang_compiler::db::RootDatabase;
use cairo_lang_compiler::project::{update_crate_root, update_crate_roots_from_project_config};
use cairo_lang_defs::db::DefsGroup;
use cairo_lang_diagnostics::DiagnosticEntry;
use cairo_lang_filesystem::db::{init_dev_corelib, FilesGroup, CORELIB_CRATE_NAME};
use cairo_lang_filesystem::ids::{CrateLongId, FileId};
use cairo_lang_semantic::db::SemanticGroup;
use cairo_lang_semantic::inline_macros::get_default_plugin_suite;
use cairo_lang_starknet::starknet_plugin_suite;
use cairo_lang_test_plugin::test_plugin_suite;
use cairo_lang_utils::{Upcast, UpcastMut};
use cairo_lint_core::diagnostics::format_diagnostic;
use cairo_lint_core::fix::{fix_semantic_diagnostic, Fix};
use cairo_lint_core::plugin::cairo_lint_plugin_suite;
use clap::Parser;
use helpers::*;
use scarb_metadata::MetadataCommand;
use scarb_ui::args::{PackagesFilter, VerbositySpec};
use scarb_ui::components::Status;
use scarb_ui::{OutputFormat, Ui};
use smol_str::SmolStr;

#[derive(Parser, Debug)]
struct Args {
    /// Name of the package.
    #[command(flatten)]
    packages_filter: PackagesFilter,
    /// Path to the file or project to analyze
    path: Option<String>,
    /// Logging verbosity.
    #[command(flatten)]
    pub verbose: VerbositySpec,
    /// Comma separated list of target names to compile.
    #[arg(long, value_delimiter = ',', env = "SCARB_TARGET_NAMES")]
    pub target_names: Vec<String>,
    /// Should lint the tests.
    #[arg(short, long, default_value_t = false)]
    pub test: bool,
    /// Should fix the lint when it can.
    #[arg(short, long, default_value_t = false)]
    pub fix: bool,
}

fn main() -> Result<()> {
    let args: Args = Args::parse();
    let ui = Ui::new(args.verbose.clone().into(), OutputFormat::Text);
    if let Err(err) = main_inner(&ui, args) {
        ui.anyhow(&err);
        std::process::exit(1);
    }
    Ok(())
}

fn main_inner(ui: &Ui, args: Args) -> Result<()> {
    // Get the scarb project metadata
    let metadata = MetadataCommand::new().inherit_stderr().exec()?;
    // Get the corelib package metadata
    let corelib = metadata
        .packages
        .iter()
        .find(|package| package.name == CORELIB_CRATE_NAME)
        .ok_or(anyhow!("Corelib not found"))?;
    // Corelib package id
    let corelib_id = &corelib.id;
    // Corelib path
    let corelib = Into::<PathBuf>::into(corelib.manifest_path.parent().as_ref().unwrap()).join("src");
    // Remove the compilation units that are not requested by the user. If none is specified will lint
    // them all. The test target is a special case and will never be linted unless specified with the
    // `--test` flag
    let compilation_units = metadata.compilation_units.into_iter().filter(|compilation_unit| {
        (args.target_names.is_empty() && compilation_unit.target.kind != targets::TEST)
            || (args.target_names.contains(&compilation_unit.target.kind))
            || (args.test && compilation_unit.target.kind == targets::TEST)
    });
    // Let's lint everything requested
    for compilation_unit in compilation_units {
        // Get the current package metadata
        let package = metadata.packages.iter().find(|package| package.id == compilation_unit.package).unwrap();
        // Print that we're checking this package.
        ui.print(Status::new("Checking", &package.name));
        // Create our db
        let mut db = RootDatabase::builder()
            .with_plugin_suite(get_default_plugin_suite())
            .with_plugin_suite(test_plugin_suite())
            .with_plugin_suite(cairo_lint_plugin_suite())
            .with_plugin_suite(starknet_plugin_suite())
            .with_cfg(to_cairo_cfg(&compilation_unit.cfg))
            .build()?;
        // Setup the corelib
        init_dev_corelib(db.upcast_mut(), corelib.clone());
        // Convert the package edition to a cairo edition. If not specified or not known it will return an
        // error.
        let edition = to_cairo_edition(
            package.edition.as_ref().ok_or(anyhow!("No edition found for package {}", package.name))?,
        )?;
        // Get the package path.
        let package_path = package.root.clone().into();
        // Build the config for this package.
        let config = build_project_config(
            &compilation_unit,
            corelib_id,
            corelib.clone(),
            package_path,
            edition,
            &package.version,
        )?;
        update_crate_roots_from_project_config(&mut db, &config);
        if let Some(corelib) = &config.corelib {
            update_crate_root(&mut db, &config, CORELIB_CRATE_NAME.into(), corelib.clone());
        }
        let crate_id =
            Upcast::<dyn FilesGroup>::upcast(&db).intern_crate(CrateLongId::Real(SmolStr::new(&package.name)));
        // Get all the diagnostics
        let mut diags = Vec::new();
        for module_id in &*db.crate_modules(crate_id) {
            diags.push(db.module_semantic_diagnostics(*module_id).unwrap());
        }
        let renderer = Renderer::styled();
        let formatted_diags = diags
            .iter()
            .flat_map(|diags| {
                diags.get_all().iter().map(|diag| format_diagnostic(diag, &db, &renderer)).collect::<Vec<_>>()
            })
            .collect::<String>();
        ui.print(formatted_diags);
        if args.fix {
            let mut fixes = Vec::with_capacity(diags.len());
            for diag in diags.iter().flat_map(|diags| diags.get_all()) {
                if let Some((fix_node, fix)) = fix_semantic_diagnostic(&db, &diag) {
                    let location = diag.location(db.upcast());
                    fixes.push(Fix { span: fix_node.span(db.upcast()), file_path: location.file_id, suggestion: fix });
                }
            }
            fixes.sort_by_key(|fix| Reverse(fix.span.start));
            let mut fixable_diagnostics = Vec::with_capacity(fixes.len());
            if fixes.len() <= 1 {
                fixable_diagnostics = fixes;
            } else {
                for i in 0..fixes.len() - 1 {
                    let first = fixes[i].span;
                    let second = fixes[i + 1].span;
                    if second.end <= first.start {
                        fixable_diagnostics.push(fixes[i].clone());
                    }
                }
            }

            let mut files: HashMap<FileId, String> = HashMap::default();
            fixable_diagnostics.into_iter().for_each(|fix| {
                let file = files.entry(fix.file_path).or_insert(db.file_content(fix.file_path).unwrap().to_string());
                (*file).replace_range(fix.span.to_str_range(), &fix.suggestion);
            });
            for (file_id, file) in files {
                println!("Fixing {}", file_id.full_path(db.upcast()));
                std::fs::write(file_id.full_path(db.upcast()), file).unwrap()
            }
        }
    }

    Ok(())
}
