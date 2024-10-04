use std::path::PathBuf;

use cairo_lang_compiler::db::RootDatabase;
use cairo_lang_defs::db::DefsGroup;
use cairo_lang_defs::ids::ModuleId;
use cairo_lang_diagnostics::Diagnostics;
use cairo_lang_filesystem::db::{init_dev_corelib, FilesGroup};
use cairo_lang_filesystem::ids::{CrateId, FileLongId};
use cairo_lang_semantic::db::SemanticGroup;
use cairo_lang_semantic::SemanticDiagnostic;
use cairo_lang_test_utils::parse_test_file::Test;
use cairo_lang_utils::ordered_hash_map::OrderedHashMap;
use cairo_lang_utils::LookupIntern;

pub struct Tests {
    pub tests: OrderedHashMap<String, Test>,
    pub should_fix: bool,
}
pub fn get_diags(crate_id: CrateId, db: &mut RootDatabase) -> Vec<Diagnostics<SemanticDiagnostic>> {
    init_dev_corelib(db, PathBuf::from(std::env::var("CORELIB_PATH").unwrap()));
    let mut diagnostics = Vec::new();
    let module_file = db.module_main_file(ModuleId::CrateRoot(crate_id)).unwrap();
    if db.file_content(module_file).is_none() {
        match module_file.lookup_intern(db) {
            FileLongId::OnDisk(_path) => {}
            FileLongId::Virtual(_) => panic!("Missing virtual file."),
            FileLongId::External(_) => (),
        }
    }

    for module_id in &*db.crate_modules(crate_id) {
        diagnostics.push(db.module_semantic_diagnostics(*module_id).unwrap());
    }
    diagnostics
}
#[macro_export]
macro_rules! test_file {
    ($lint_group: ident, $file_path:ident, $($test_name:expr),*) => {

        paste ! {

            const [<TEST_FILENAME_ $file_path:upper>]: &str = concat!("tests/test_files/", stringify!($lint_group), "/", stringify!($file_path));
            static [<PARSED_TEST_FILE_ $file_path:upper>]: LazyLock<OrderedHashMap<String, Test>> =
                LazyLock::new(|| parse_test_file(Path::new([<TEST_FILENAME_ $file_path:upper>])).unwrap());
            static [<FIXED_TEST_FILE_ $file_path:upper>]: LazyLock<Mutex<Tests>> =
                LazyLock::new(|| Mutex::new(Tests { tests: OrderedHashMap::default(), should_fix: false }));


            #[dtor]
            fn [<fix_ $lint_group $file_path _test_file>]() {
                let val = [<FIXED_TEST_FILE_ $file_path:upper>].lock().unwrap();
                let res = OrderedHashMap::<String, Test>::from_iter(val.tests.clone().into_iter().sorted_by_key(|kv| kv.0.clone()));
                if val.should_fix {
                    dump_to_test_file(res, [<TEST_FILENAME_ $file_path:upper>]).unwrap();
                }
            }

            $(#[test_case($test_name; $test_name)])*
            fn [<$lint_group _ $file_path>](test_name: &str) {
                let test =  [<PARSED_TEST_FILE_ $file_path:upper>].get(test_name).expect("Couldn't get test");
                let is_fix_mode = std::env::var("FIX_TESTS") == Ok("1".into());
                let mut file = test.attributes.get("cairo_code").expect("Couldn't get cairo code").clone();
                let mut db = RootDatabase::builder()
                    .with_plugin_suite(get_default_plugin_suite())
                    .with_plugin_suite(test_plugin_suite())
                    .with_plugin_suite(cairo_lint_plugin_suite())
                    .build()
                    .unwrap();

                let diags = get_diags(setup_test_crate_ex(db.upcast(), &file, Some(CRATE_CONFIG)), &mut db);
                // Transform Vec<Diagnostics<Semantic>> into Vec<Semantic>
                let semantic_diags: Vec<_> = diags.clone().into_iter().flat_map(|diag| diag.get_all()).collect();
                let unused_imports: HashMap<FileId, HashMap<SyntaxNode, ImportFix>> =
                    collect_unused_imports(&db, &semantic_diags);
                let mut fixes = if unused_imports.keys().len() > 0 {
                    let current_file_id = unused_imports.keys().next().unwrap();
                    apply_import_fixes(&db, unused_imports.get(&current_file_id).unwrap())
                } else {
                    Vec::new()
                };

                // Handle other types of fixes
                for diag in diags.iter().flat_map(|diags| diags.get_all()) {
                    if !matches!(diag.kind, SemanticDiagnosticKind::UnusedImport(_)) {
                        if let Some((fix_node, fix)) = fix_semantic_diagnostic(&db, &diag) {
                            let span = fix_node.span(db.upcast());
                            fixes.push(Fix { span, suggestion: fix });
                        }
                    }
                }

                fixes.sort_by_key(|v| std::cmp::Reverse(v.span.start));
                if !test_name.contains("nested") {
                    for fix in fixes.iter() {
                        file.replace_range(fix.span.to_str_range(), &fix.suggestion);
                    }
                } else {
                    file = "Contains nested diagnostics can't fix it".to_string();
                }
                let renderer = Renderer::plain();
                let formatted_diags =
                    diags.into_iter().flat_map(|diags| diags.get_all().iter().map(|diag| format_diagnostic(diag, &db, &renderer)).collect::<Vec<_>>()).collect::<String>().trim().to_string();
                if is_fix_mode {
                    let mut new_test = test.clone();
                    new_test.attributes.insert("diagnostics".to_string(), formatted_diags.clone());
                    new_test.attributes.insert("fixed".to_string(), file.clone());
                    let mut new_tests = [<FIXED_TEST_FILE_ $file_path:upper>].lock().unwrap();
                    new_tests.should_fix = true;
                    new_tests.tests.insert(test_name.to_string(), new_test);
                }
                assert_eq!(&formatted_diags, test.attributes.get("diagnostics").expect("Couldn't get expected diagnostics"));
                assert_eq!(&file, test.attributes.get("fixed").expect("Couldn't get expected fix"));
            }
        }
    };
}
