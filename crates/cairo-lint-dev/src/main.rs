use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

fn create_new_test(lint_name: &str) -> io::Result<()> {
    let test_content = format!(
        "//! > Test name\n\n//! > cairo_code\nfn main() {{\n    let a: Option<felt252> = Option::Some(1);\n}}\n"
    );

    let test_files_dir = PathBuf::from("crates/cairo-lint-core/tests/test_files");
    if !test_files_dir.exists() {
        fs::create_dir_all(&test_files_dir)?;
    }

    let file_name = test_files_dir.join(lint_name);

    let mut file = fs::File::create(&file_name)?;
    file.write_all(test_content.as_bytes())?;

    println!("Test file created: {}", file_name.display());

    let tests_rs_path = Path::new("crates/cairo-lint-core/tests/tests.rs");

    if !tests_rs_path.exists() {
        eprintln!("Error: tests.rs file not found!");
        return Ok(());
    }

    let new_test_entry = format!(r#"test_file!({}, "Test name");"#, lint_name);

    let mut tests_rs_content = fs::read_to_string(tests_rs_path)?;
    tests_rs_content.push_str("\n");
    tests_rs_content.push_str(&new_test_entry);
    fs::write(tests_rs_path, tests_rs_content)?;

    println!("Test entry added to tests.rs");

    Ok(())
}

fn main() {
    let lint_name = if let Some(arg1) = std::env::args().nth(1) {
        arg1
    } else {
        println!("Enter the name of the lint:");
        let mut lint_name = String::new();
        io::stdin().read_line(&mut lint_name).expect("Failed to read line");
        lint_name.trim().to_string()
    };

    if let Err(e) = create_new_test(&lint_name) {
        eprintln!("Error creating test file: {}", e);
    }
}
