// build/mod.rs
use clang::*;
use std::collections::HashMap;

mod format_name;
mod handlers;
pub use self::config::HandlerConfigs;
use handlers::*;

pub enum Handler {
    Record(Box<dyn Fn(&Entity, &HandlerMap, &mut HandlerConfigs) -> Vec<String>>),
    FunctionPrototype(Box<dyn Fn(&Entity, &HandlerMap, &mut HandlerConfigs) -> Vec<String>>),
    // Other handlers
}

pub type HandlerMap = HashMap<TypeKind, Handler>;

pub fn create_handlers() -> HandlerMap {
    let mut handlers: HandlerMap = HashMap::new();
    handlers.insert(
        TypeKind::Record,
        Handler::Record(Box::new(handle_record::handle_record)),
    );
    handlers.insert(
        TypeKind::FunctionPrototype,
        Handler::FunctionPrototype(Box::new(
            handle_function_prototype::handle_function_prototype,
        )),
    );
    // handle all possible param types
    handle_function_parameter::insert_function_parameter_handlers(&mut handlers);
    // Initialize other handlers
    handlers
}

pub fn process_children(
    entity: &Entity,
    handlers: &HandlerMap,
    configs: &mut HandlerConfigs,
) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let mut current_child_index: usize = 0;
    configs.num_parent_children = entity.get_children().len().try_into().unwrap_or(0);
    entity.visit_children(|child, _| {
        if let Some(handler) = child
            .get_type()
            .and_then(|node_type| handlers.get(&node_type.get_kind()))
        {
            configs.index = current_child_index;
            match handler {
                Handler::Record(h) => lines.extend(h(&child, handlers, configs)),
                Handler::FunctionPrototype(h) => lines.extend(h(&child, handlers, configs)),
            }
        }
        current_child_index += 1;
        EntityVisitResult::Continue
    });

    lines
}
