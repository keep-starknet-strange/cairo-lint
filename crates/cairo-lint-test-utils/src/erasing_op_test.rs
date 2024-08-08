
fn test_erase_op() {
    let source_code = r#"
    fn main() {
        let x = 1;
        let _ = 0 / x;  // Should trigger lint
        let _ = 0 * x;  // Should trigger lint
        let _ = x & 0;  // Should trigger lint
    }
    "#;

    let lint_result = run_lint(source_code);
    assert!(lint_result.contains("operation can be simplified to zero"));
}