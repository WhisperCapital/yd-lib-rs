use super::HandlerMap;
use clang::*;

pub fn handle_function_prototype(entity: &Entity, handlers: &HandlerMap) -> Vec<String> {
    let mut lines: Vec<String> = vec![format!(
        "// Function prototype: {}",
        entity.get_display_name().unwrap_or_default()
    )];
    let child_lines = super::process_children(entity, handlers);
    lines.extend(child_lines);
    lines
}
