use crate::build_utils::{
    config::HandlerConfigs, handle_function_parameter::ParameterFlavor, process_children,
    HandlerMap,
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
    let snake_fn_name = Inflector::to_snake_case(&entity.get_name().unwrap());
    let record_name = configs.record_name.clone();

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
            lines.extend(child_lines_rs);
            lines.push(format!(" ),\n"));
        }
        MethodFlavor::StaticTable => {
            lines.push(format!(r"{snake_fn_name}: spi_{snake_fn_name},\n"));
        }
        MethodFlavor::OutputEnum => {
            lines.push(format!(
                "{}{snake_fn_name}({record_name}{snake_fn_name}Packet),\n",
                *INDENT
            ));
        }
        MethodFlavor::OutputEnumStruct => {
            lines.push(format!(
                r#"
#[derive(Clone, Debug)]
pub struct {record_name}{snake_fn_name}Packet {{
"#
            ));
            lines.extend(
                child_lines_rs
                    .iter()
                    .map(|arg| format!("{}pub {},\n", arg.replace("&", "").clone(), *INDENT)),
            );
            lines.push(format!("}}\n"));
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
                r#"extern "C" fn spi_{snake_fn_name}(spi: *mut {record_name}Fat"#
            ));
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
            let child_lines_c = process_children(
                entity,
                handlers,
                &mut HandlerConfigs {
                    // ask function handler to output trait style code
                    parameter_flavor: ParameterFlavor::C,
                    ..configs.clone()
                },
            );
            let trait_line_front = format!(
                "fn {snake_fn_name}(&mut self{}) \n",
                child_lines_rs.join(", ")
            );
            let full_spi_output_enum_name = format!("{record_name}Output");
            let mut pushed_lines: Vec<String> = vec![];
            pushed_lines.push(format!("{full_spi_output_enum_name}::{snake_fn_name}( {record_name}{snake_fn_name}Packet {{\n"));
            pushed_lines.extend(child_lines_rs);
            pushed_lines.push(format!("}}\n"));
            lines.push(format!(
                r#"{trait_line_front} {{
    self.inner.lock().unwrap().push("#
            ));
            lines.extend(pushed_lines);
            lines.push(format!(")\n}}"));
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
