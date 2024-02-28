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
    StaticTable,
    OutputEnum,
    OutputEnumStruct,
    CFn,
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
    let child_lines_rs = process_children(
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
                child_lines_rs.join(", ")
            );
            lines.push(formatted_line);
        }
        MethodFlavor::Struct => {
            let formatted_line = format!(
                r#"{snake_fn_name}: extern "C" fn(spi: *mut {}Fat{}) ,"#,
                configs.record_name,
                child_lines_rs.join(", ")
            );
            lines.push(formatted_line);
        }
        MethodFlavor::StaticTable => {
            let formatted_line = format!(r#"{snake_fn_name}: spi_{snake_fn_name},"#,);
            lines.push(formatted_line);
        }
        MethodFlavor::OutputEnum => {
            let formatted_line = format!(
                r#"{snake_fn_name}({}{snake_fn_name}Packet),"#,
                configs.record_name
            );
            lines.push(formatted_line);
        }
        MethodFlavor::OutputEnumStruct => {
            lines.push(format!(
                r#"
#[derive(Clone, Debug)]
pub struct {}{snake_fn_name}Packet {{
"#,
                configs.record_name
            ));
            lines.extend(
                child_lines_rs
                    .iter()
                    .map(|arg| format!("pub {},", arg.replace("&", "").clone())),
            );
            lines.push(format!(
                r#"
}}"#
            ));
        }
        MethodFlavor::CFn => {
            let child_lines_c = process_children(
                entity,
                handlers,
                &mut HandlerConfigs {
                    // ask function handler to output trait style code
                    parameter_flavor: ParameterFlavor::C,
                    ..configs.clone()
                },
            );
            let formatted_line = format!(
                r#"extern "C" fn spi_{}(spi: *mut {}Fat, {}) {{
                    unsafe {{
                        (*(*spi).md_spi_ptr).{}({})
                    }}
                }}"#,
                snake_fn_name,
                configs.record_name,
                child_lines_rs.join(","),
                snake_fn_name,
                child_lines_c.join(","),
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
