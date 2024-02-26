use clang::*;
use super::HandlerMap;


pub fn handle_record(entity: &Entity, handlers: &HandlerMap) -> Vec<String> {
    let mut lines: Vec<String> = vec![format!("// Record: {}", entity.get_display_name().unwrap_or_default())];
    let child_lines = super::process_children(entity, handlers);
    lines.extend(child_lines);
    lines
}
