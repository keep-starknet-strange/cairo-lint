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

#[derive(Debug, Clone)]
pub struct ImportFix {
    // The node that contains the imports to be fixed.
    pub node: SyntaxNode,
    // The items to remove from the imports.
    pub items_to_remove: Vec<String>,
}

impl ImportFix {
    pub fn new(node: SyntaxNode) -> Self {
        ImportFix { node, items_to_remove: vec![] }
    }
}

use crate::fix::Fix;

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

fn process_unused_import(db: &RootDatabase, id: &UseId, fixes: &mut HashMap<SyntaxNode, ImportFix>) {
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

pub fn apply_import_fixes(db: &RootDatabase, fixes: &HashMap<SyntaxNode, ImportFix>) -> Vec<Fix> {
    fixes
        .iter()
        .flat_map(|(_, import_fix)| {
            let span = import_fix.node.span(db);

            if import_fix.items_to_remove.is_empty() {
                // Single import case: remove entire import
                vec![Fix { span, suggestion: String::new() }]
            } else {
                // Multi-import case
                handle_multi_import(db, &import_fix.node, &import_fix.items_to_remove)
            }
        })
        .collect()
}

fn handle_multi_import(db: &RootDatabase, node: &SyntaxNode, items_to_remove: &[String]) -> Vec<Fix> {
    if all_descendants_removed(db, node, items_to_remove) {
        remove_entire_import(db, node)
    } else {
        remove_specific_items(db, node, items_to_remove)
    }
}

fn all_descendants_removed(db: &RootDatabase, node: &SyntaxNode, items_to_remove: &[String]) -> bool {
    node.descendants(db)
        .filter(|child| child.kind(db) == SyntaxKind::UsePathLeaf)
        .all(|child| items_to_remove.contains(&child.get_text_without_trivia(db)))
}

fn remove_entire_import(db: &RootDatabase, node: &SyntaxNode) -> Vec<Fix> {
    let mut current_node = node.clone();
    while let Some(parent) = current_node.parent() {
        // Go up until we find a UsePathList on the path - then, we can remove the current node from that
        // list.
        if matches!(parent.kind(db), SyntaxKind::UsePathList) {
            // To remove the current node from the UsePathList, we need to:
            // 1. Get the text of the current node, which becomes "to remove"
            // 2. Rewrite the UsePathList with the current node text removed.
            let items_to_remove = vec![current_node.get_text_without_trivia(db)];
            let parent = parent.parent().unwrap().clone();
            let fix = handle_multi_import(db, &parent, &items_to_remove);
            return fix;
        }
        if matches!(parent.kind(db), SyntaxKind::ItemUse) {
            current_node = parent.clone();
            break;
        }
        current_node = parent;
    }
    vec![Fix { span: current_node.span(db), suggestion: String::new() }]
}

fn remove_specific_items(db: &RootDatabase, node: &SyntaxNode, items_to_remove: &[String]) -> Vec<Fix> {
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
    let mut items: Vec<_> = children.iter().map(|child| child.get_text(db).trim().to_string()).collect();
    items.retain(|item| !items_to_remove.contains(&item.to_string()));

    let text = if items.len() == 1 { items[0].to_string() } else { format!("{{ {} }}", items.join(", ")) };

    vec![Fix { span: node.span(db), suggestion: text }]
}

fn find_use_path_list(db: &RootDatabase, node: &SyntaxNode) -> SyntaxNode {
    node.descendants(db)
        .find(|descendant| descendant.kind(db) == SyntaxKind::UsePathList)
        .unwrap_or_else(|| node.clone())
}
