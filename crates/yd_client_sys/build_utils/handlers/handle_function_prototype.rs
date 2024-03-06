use crate::build_utils::{
    config::HandlerConfigs, format_name::format_enum_name,
    handle_function_parameter::ParameterFlavor, process_children, HandlerMap,
};
use clang::*;
use inflector::Inflector;

lazy_static! {
    static ref INDENT: String = "    ".to_string();
}

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
    SpiFn,
    /// only add debug log
    None,
}

pub fn handle_function_prototype(
    entity: &Entity,
    handlers: &HandlerMap,
    configs: &mut HandlerConfigs,
) -> Vec<String> {
    let camel_case_name = entity.get_name().unwrap();
    let enum_name = format_enum_name(&camel_case_name);
    let snake_fn_name = Inflector::to_snake_case(&camel_case_name);
    let record_name = configs.record_name.clone();
    let packet_name = format!("{record_name}{enum_name}Packet");

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
            lines.push(format!("{}fn {snake_fn_name}(&mut self", *INDENT));
            lines.extend(child_lines_rs);
            lines.push(format!(") {{}}\n"));
        }
        MethodFlavor::Struct => {
            lines.push(format!(
                r#"{}{snake_fn_name}: extern "C" fn(spi: *mut {record_name}Fat"#,
                *INDENT
            ));
            if !child_lines_rs.is_empty() {
                lines.push(format!(", "));
            }
            lines.extend(child_lines_rs);
            lines.push(format!("),\n"));
        }
        MethodFlavor::StaticTable => {
            lines.push(format!(
                "{}{snake_fn_name}: spi_{snake_fn_name},\n",
                *INDENT
            ));
        }
        MethodFlavor::OutputEnum => {
            lines.push(format!("{}{enum_name}({packet_name}Packet),\n", *INDENT));
        }
        MethodFlavor::OutputEnumStruct => {
            lines.push(format!(
                r#"
#[derive(Clone, Debug)]
pub struct {packet_name}Packet {{
"#
            ));
            let child_lines_rs_struct = process_children(
                entity,
                handlers,
                &mut HandlerConfigs {
                    // ask function handler to output trait style code
                    parameter_flavor: ParameterFlavor::RustStruct,
                    ..configs.clone()
                },
            );
            lines.extend(child_lines_rs_struct);
            lines.push(format!("\n}}\n"));
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
            lines.push(format!(
                r#"
extern "C" fn spi_{snake_fn_name}(spi: *mut {record_name}Fat"#
            ));
            if !child_lines_rs.is_empty() {
                lines.push(format!(", "));
            }
            lines.extend(child_lines_rs);
            lines.push(format!(
                r#") {{
    unsafe {{
        (*(*spi).md_spi_ptr).{snake_fn_name}("#,
            ));
            lines.extend(child_lines_c);
            lines.push(format!(
                r#")
    }}
}}
"#
            ));
        }
        MethodFlavor::SpiFn => {
            lines.push(format!("{}fn {snake_fn_name}(&mut self", *INDENT,));
            if !child_lines_rs.is_empty() {
                lines.push(format!(", "));
            }
            lines.extend(child_lines_rs.clone());
            lines.push(format!(
                r#") {{
        self.inner.lock().unwrap().push("#
            ));
            lines.push(format!(
                "{full_spi_output_enum_name}::{snake_fn_name}({packet_name}Packet {{\n",
                full_spi_output_enum_name = format!("{record_name}Output")
            ));
            lines.extend(process_children(
                entity,
                handlers,
                &mut HandlerConfigs {
                    // ask function handler to output trait style code
                    parameter_flavor: ParameterFlavor::SpiFn,
                    ..configs.clone()
                },
            ));
            lines.push(format!(
                "{indent}{indent}}}))\n{indent}}}\n",
                indent = *INDENT
            ));
        }
        MethodFlavor::None => {
            lines.push(format!(
                "// FunctionPrototype: {}\n",
                entity.get_display_name().unwrap_or_default()
            ));
        }
    }
    lines
}
