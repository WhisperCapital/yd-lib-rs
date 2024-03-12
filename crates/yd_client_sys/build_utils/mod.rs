// build/mod.rs
use clang::*;
use std::collections::HashMap;

mod format_name;
pub mod handlers;
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

fn get_handler<'a>(entity: &'a Entity<'a>, handlers: &'a HandlerMap) -> Option<&'a Handler> {
    entity
        .get_type()
        .and_then(|node_type| handlers.get(&node_type.get_kind()))
}

fn count_children_with_same_handler(
    entity: &Entity,
    child_handler: &Handler,
    handlers: &HandlerMap,
) -> usize {
    entity
        .get_children()
        .into_iter()
        .filter(|c| {
            get_handler(c, handlers).map_or(false, |c_handler| {
                matches!(
                    (c_handler, child_handler),
                    (Handler::Record(_), Handler::Record(_))
                        | (Handler::FunctionPrototype(_), Handler::FunctionPrototype(_))
                )
            })
        })
        .count()
}

fn get_child_index_with_same_handler(
    entity: &Entity,
    child_entity: &Entity,
    child_handler: &Handler,
    handlers: &HandlerMap,
) -> usize {
    let mut count = 0;
    for child in entity.get_children().into_iter() {
        // Stop counting once we reach the child entity
        if &child == child_entity {
            break;
        }

        if let Some(c_handler) = get_handler(&child, handlers) {
            if matches!(
                (c_handler, child_handler),
                (Handler::Record(_), Handler::Record(_))
                    | (Handler::FunctionPrototype(_), Handler::FunctionPrototype(_))
            ) {
                count += 1;
            }
        }
    }
    count
}


pub fn process_children(
    entity: &Entity,
    handlers: &HandlerMap,
    configs: &mut HandlerConfigs,
) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();

    entity.visit_children(|child, _| {
        if let Some(child_handler) = get_handler(&child, handlers) {
            let num_parent_children_same_handler = count_children_with_same_handler(entity, &child_handler, handlers);
            let child_index = get_child_index_with_same_handler(entity, &child, &child_handler, handlers);

            configs.num_parent_children_same_handler = num_parent_children_same_handler.try_into().unwrap_or(0);
            configs.index = child_index;

            match child_handler {
                Handler::Record(h) => lines.extend(h(&child, handlers, configs)),
                Handler::FunctionPrototype(h) => lines.extend(h(&child, handlers, configs)),
            }
        }

        EntityVisitResult::Continue
    });

    lines
}
