// build/mod.rs
use clang::*;
use std::collections::HashMap;

pub mod record_handler;
pub mod function_prototype_handler;
// Include other handler modules as needed

pub enum Handler {
    Record(Box<dyn Fn(&Entity, &HandlerMap) -> Vec<String>>),
    FunctionPrototype(Box<dyn Fn(&Entity, &HandlerMap) -> Vec<String>>),
    // Other handlers
}

pub type HandlerMap = HashMap<TypeKind, Handler>;

pub fn create_handlers() -> HandlerMap {
    let mut handlers: HandlerMap = HashMap::new();
    handlers.insert(TypeKind::Record, Handler::Record(Box::new(record_handler::handle_record)));
    handlers.insert(TypeKind::FunctionPrototype, Handler::FunctionPrototype(Box::new(function_prototype_handler::handle_function_prototype)));
    // Initialize other handlers
    handlers
}

pub fn process_children(entity: &Entity, handlers: &HandlerMap) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();

    entity.visit_children(|child, _| {
        if let Some(handler) = child.get_type().and_then(|node_type| handlers.get(&node_type.get_kind())) {
            match handler {
                Handler::Record(h) => lines.extend(h(&child, handlers)), // Corrected line
                Handler::FunctionPrototype(h) => lines.extend(h(&child, handlers)), // Ensure this line is also corrected
                // Handle other types as needed
            }
        }
        EntityVisitResult::Continue
    });

    lines
}
