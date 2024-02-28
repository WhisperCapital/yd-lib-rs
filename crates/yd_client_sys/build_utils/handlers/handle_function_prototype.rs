use crate::build_utils::{
    config::HandlerConfigs, handle_function_parameter::ParameterFlavor, process_children,
    HandlerMap,
};
use clang::*;
use inflector::Inflector;

#[derive(Clone)]
pub enum MethodFlavor {
    /// method is in a struct
    Struct,
    /// method is in a trait
    Trait,
    /// only add debug log
    None,
}

pub fn handle_function_prototype(
    entity: &Entity,
    handlers: &HandlerMap,
    configs: &mut HandlerConfigs,
) -> Vec<String> {
    let snake_fn_name = Inflector::to_snake_case(&entity.get_name().unwrap());

    let mut lines: Vec<String> = vec![];
    // get arg from child node if possible
    let child_lines = process_children(
        entity,
        handlers,
        &mut HandlerConfigs {
            // ask function handler to output trait style code
            parameter_flavor: ParameterFlavor::Rust,
            ..configs.clone()
        },
    );
    match configs.method_flavor {
        MethodFlavor::Trait => {
            let formatted_line = format!(
                "fn {snake_fn_name}(&mut self{}) {{}}\n",
                child_lines.join(", ")
            );
            lines.push(formatted_line);
        }
        MethodFlavor::Struct => {
            let formatted_line = format!(
                r#"{snake_fn_name}: extern "C" fn(spi: *mut {}Fat{}) ,"#,
                configs.record_name,
                child_lines.join(", ")
            );
            lines.push(formatted_line);
        }
        MethodFlavor::None => {
            lines.push(format!(
                "// FunctionPrototype: {}",
                entity.get_display_name().unwrap_or_default()
            ));
        }
    }
    lines
}
