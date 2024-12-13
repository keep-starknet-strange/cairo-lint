//! # Import Fixes for Cairo Lint
//!
//! This module provides functionality to detect and fix unused imports in Cairo code.
//! The process involves three main steps:
//!
//! 1. Collecting unused imports: Analyze semantic diagnostics to identify unused imports.
//! 2. Creating import fixes: Generate `ImportFix` structures for each unused import.
//! 3. Applying fixes: Remove or modify the imports based on the collected fixes.
//!
//! The module handles both single imports and multi-imports, ensuring that only unused
//! items are removed while preserving the structure of the import statements.

use std::collections::HashMap;

use cairo_lang_compiler::db::RootDatabase;
use cairo_lang_defs::ids::UseId;
use cairo_lang_diagnostics::DiagnosticEntry;
use cairo_lang_filesystem::ids::FileId;
use cairo_lang_semantic::diagnostic::SemanticDiagnosticKind;
use cairo_lang_semantic::SemanticDiagnostic;
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::kind::SyntaxKind;
use cairo_lang_syntax::node::{SyntaxNode, TypedStablePtr, TypedSyntaxNode};
use cairo_lang_utils::Upcast;

/// Represents a fix for unused imports in a specific syntax node.
#[derive(Debug, Clone)]
pub struct ImportFix {
    /// The node that contains the imports to be fixed.
    pub node: SyntaxNode,
    /// The items to remove from the imports.
    pub items_to_remove: Vec<String>,
}

impl ImportFix {
    /// Creates a new `ImportFix` for the given syntax node.
    pub fn new(node: SyntaxNode) -> Self {
        ImportFix {
            node,
            items_to_remove: vec![],
        }
    }
}

use crate::fix::Fix;

/// Collects unused imports from semantic diagnostics.
///
/// # Arguments
///
/// * `db` - The root database containing the project information.
/// * `diags` - A vector of semantic diagnostics.
///
/// # Returns
///
/// A HashMap where keys are FileIds and values are HashMaps of SyntaxNodes to ImportFixes.
pub fn collect_unused_imports(
    db: &RootDatabase,
    diags: &Vec<SemanticDiagnostic>,
) -> HashMap<FileId, HashMap<SyntaxNode, ImportFix>> {
    let mut file_fixes = HashMap::new();

    for diag in diags {
        if let SemanticDiagnosticKind::UnusedImport(id) = &diag.kind {
            let file_id = diag.location(db.upcast()).file_id;

            let local_fixes = file_fixes.entry(file_id).or_insert_with(HashMap::new);
            process_unused_import(db, id, local_fixes);
        }
    }

    file_fixes
}

/// Processes an unused import and updates the fixes HashMap.
///
/// # Arguments
///
/// * `db` - The root database containing the project information.
/// * `id` - The UseId of the unused import.
/// * `fixes` - A mutable reference to the HashMap of fixes.
fn process_unused_import(
    db: &RootDatabase,
    id: &UseId,
    fixes: &mut HashMap<SyntaxNode, ImportFix>,
) {
    let unused_node = id.stable_ptr(db).lookup(db.upcast()).as_syntax_node();
    let mut current_node = unused_node.clone();

    while let Some(parent) = current_node.parent() {
        match parent.kind(db) {
            SyntaxKind::UsePathMulti => {
                fixes
                    .entry(parent.clone())
                    .or_insert_with(|| ImportFix::new(parent.clone()))
                    .items_to_remove
                    .push(unused_node.get_text_without_trivia(db));
                break;
            }
            SyntaxKind::ItemUse => {
                fixes.insert(parent.clone(), ImportFix::new(parent.clone()));
                break;
            }
            _ => current_node = parent,
        }
    }
}

/// Applies the collected import fixes to generate a list of Fix objects.
///
/// # Arguments
///
/// * `db` - The root database containing the project information.
/// * `fixes` - A HashMap of SyntaxNodes to ImportFixes.
///
/// # Returns
///
/// A vector of Fix objects representing the applied fixes.
pub fn apply_import_fixes(db: &RootDatabase, fixes: &HashMap<SyntaxNode, ImportFix>) -> Vec<Fix> {
    fixes
        .iter()
        .flat_map(|(_, import_fix)| {
            let span = import_fix.node.span(db);

            if import_fix.items_to_remove.is_empty() {
                // Single import case: remove entire import
                vec![Fix {
                    span,
                    suggestion: String::new(),
                }]
            } else {
                // Multi-import case
                handle_multi_import(db, &import_fix.node, &import_fix.items_to_remove)
            }
        })
        .collect()
}

/// Handles multi-import cases, deciding whether to remove the entire import or specific items.
///
/// # Arguments
///
/// * `db` - The root database containing the project information.
/// * `node` - The syntax node of the import.
/// * `items_to_remove` - A slice of strings representing the items to be removed.
///
/// # Returns
///
/// A vector of Fix objects for the multi-import case.
fn handle_multi_import(
    db: &RootDatabase,
    node: &SyntaxNode,
    items_to_remove: &[String],
) -> Vec<Fix> {
    if all_descendants_removed(db, node, items_to_remove) {
        remove_entire_import(db, node)
    } else {
        remove_specific_items(db, node, items_to_remove)
    }
}

/// Checks if all descendants of a node are to be removed.
///
/// # Arguments
///
/// * `db` - The root database containing the project information.
/// * `node` - The syntax node to check.
/// * `items_to_remove` - A slice of strings representing the items to be removed.
///
/// # Returns
///
/// A boolean indicating whether all descendants should be removed.
fn all_descendants_removed(
    db: &RootDatabase,
    node: &SyntaxNode,
    items_to_remove: &[String],
) -> bool {
    node.descendants(db)
        .filter(|child| child.kind(db) == SyntaxKind::UsePathLeaf)
        .all(|child| items_to_remove.contains(&child.get_text_without_trivia(db)))
}

/// Removes an entire import statement.
///
/// We traverse the parents until we either find a UsePathList on the path - then, we can remove the
/// current node from that list - or we find an ItemUse, in which case we remove the entire import
/// line.
///
/// # Arguments
/// * `db` - The root database containing the project information.
/// * `node` - The syntax node of the import to remove.
///
/// # Returns
///
/// A vector of Fix objects for removing the entire import.
fn remove_entire_import(db: &RootDatabase, node: &SyntaxNode) -> Vec<Fix> {
    let mut current_node = node.clone();
    while let Some(parent) = current_node.parent() {
        // Go up until we find a UsePathList on the path - then, we can remove the current node from that
        // list.
        if parent.kind(db) == SyntaxKind::UsePathList {
            // To remove the current node from the UsePathList, we need to:
            // 1. Get the text of the current node, which becomes "to remove"
            // 2. Rewrite the UsePathList with the current node text removed.
            let items_to_remove = vec![current_node.get_text_without_trivia(db)];
            if let Some(grandparent) = parent.parent() {
                return handle_multi_import(db, &grandparent, &items_to_remove);
            }
        }
        if parent.kind(db) == SyntaxKind::ItemUse {
            current_node = parent;
            break;
        }
        current_node = parent;
    }
    vec![Fix {
        span: current_node.span(db),
        suggestion: String::new(),
    }]
}

/// Removes specific items from a multi-import statement.
///
/// # Arguments
///
/// * `db` - The root database containing the project information.
/// * `node` - The syntax node of the import.
/// * `items_to_remove` - A slice of strings representing the items to be removed.
///
/// # Returns
///
/// A vector of Fix objects for removing specific items from the import.
fn remove_specific_items(
    db: &RootDatabase,
    node: &SyntaxNode,
    items_to_remove: &[String],
) -> Vec<Fix> {
    let use_path_list = find_use_path_list(db, node);
    let children = db.get_children(use_path_list.clone());
    let children: Vec<SyntaxNode> = children
        .iter()
        .filter(|child| {
            let text = child.get_text(db).trim().replace('\n', "");
            !text.is_empty() && !text.eq(",")
        })
        .cloned()
        .collect();
    let mut items: Vec<_> = children
        .iter()
        .map(|child| child.get_text(db).trim().to_string())
        .collect();
    items.retain(|item| !items_to_remove.contains(&item.to_string()));

    let text = if items.len() == 1 {
        items[0].to_string()
    } else {
        format!("{{{}}}", items.join(", "))
    };

    vec![Fix {
        span: node.span(db),
        suggestion: text,
    }]
}

/// Finds the UsePathList node within a given syntax node.
///
/// # Arguments
///
/// * `db` - The root database containing the project information.
/// * `node` - The syntax node to search within.
///
/// # Returns
///
/// The UsePathList syntax node, or the original node if not found.
fn find_use_path_list(db: &RootDatabase, node: &SyntaxNode) -> SyntaxNode {
    node.descendants(db)
        .find(|descendant| descendant.kind(db) == SyntaxKind::UsePathList)
        .unwrap_or_else(|| node.clone())
}
