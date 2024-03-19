use crate::build_utils::{
    config::HandlerConfigs, format_name::format_enum_name,
    handle_function_parameter::ParameterFlavor, process_children, HandlerMap,
};
use clang::*;
use inflector::Inflector;

lazy_static! {
    static ref INDENT: String = "    ".to_string();
}

macro_rules! console_debug {
    ($($tokens: tt)*) => {
        println!("cargo:warning={}", format!($($tokens)*))
    }
}

#[derive(Clone)]
pub enum MethodFlavor {
    /// method is in a struct
    Struct,
    /// method is in a trait
    SpiTrait,
    ApiTrait,
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
    let raw_camel_case_name = entity.get_name().unwrap();
    let record_name = configs.record_name.clone();
    let sibling_index = find_previous_sibling_index(entity, configs);
    let method_reload_suffix = if sibling_index > 0 {
        format!("{sibling_index}")
    } else {
        "".to_string()
    };
    let snake_fn_name = format!(
        "{}{method_reload_suffix}",
        Inflector::to_snake_case(&raw_camel_case_name)
    );
    let camel_case_name = Inflector::to_camel_case(&snake_fn_name);

    let enum_name = format!(
        "{}{method_reload_suffix}",
        format_enum_name(&raw_camel_case_name)
    );
    let packet_name_prefix = format!("{record_name}{enum_name}");

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
        MethodFlavor::SpiTrait => {
            lines.push(format!("{}fn {snake_fn_name}(&mut self", *INDENT));
            if !child_lines_rs.is_empty() {
                lines.push(format!(", "));
            }
            lines.extend(child_lines_rs);
            lines.push(format!(") {{}}\n"));
        }
        MethodFlavor::ApiTrait => {
            if raw_camel_case_name.starts_with("~") {
                // TODO: 在别处处理类的析构
                return lines;
            }
            lines.push(format!("{}pub fn {snake_fn_name}(&mut self", *INDENT));
            if !child_lines_rs.is_empty() {
                lines.push(format!(", "));
            }
            lines.extend(child_lines_rs);
            let c_result_type = entity.get_result_type().unwrap().get_display_name();
            let rust_result_type = get_rs_result_type_from_c_result_type(&c_result_type);
            // TODO: 这个可能需要拼一下，不知道对不对
            let full_api_record_name = format!("{record_name}_{camel_case_name}");
            let child_lines_c = process_children(
                entity,
                handlers,
                &mut HandlerConfigs {
                    // ask function handler to output trait style code
                    parameter_flavor: ParameterFlavor::MethodCallParam,
                    ..configs.clone()
                },
            );
            lines.push(format!(
                r#") -> {rust_result_type} {{
        unsafe {{
            ((*(*self).vtable_).{full_api_record_name})(self as *mut {record_name}"#
            ));
            // console_debug!("{full_api_record_name} {:?}", child_lines_c);
            if !child_lines_c.is_empty() {
                lines.push(format!(", "));
            }
            lines.extend(child_lines_c);
            lines.push(format!(
                r#")
        }}
    }}
"#
            ));
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
            let config_for_children = &mut HandlerConfigs {
                // ask function handler to output trait style code
                parameter_flavor: ParameterFlavor::RustStruct,
                ..configs.clone()
            };
            process_children(
                entity,
                handlers,
                config_for_children,
            );
            // life_time_param_on_parent = config_for_children.life_time_on_children ? "<'a>" : "";
            let life_time_param_on_parent = if config_for_children.life_time_on_children {
                "<'a>"
            } else {
                ""
            };
            configs.life_time_on_children = false;
            lines.push(format!("{}{enum_name}({packet_name_prefix}Packet{life_time_param_on_parent}),\n", *INDENT));
        }
        MethodFlavor::OutputEnumStruct => {
            let config_for_children = &mut HandlerConfigs {
                // ask function handler to output trait style code
                parameter_flavor: ParameterFlavor::RustStruct,
                life_time: "'a ".to_string(),
                ..configs.clone()
            };
            let child_lines_rs_struct = process_children(
                entity,
                handlers,
                config_for_children,
            );
            // life_time_param_on_parent = config_for_children.life_time_on_children ? "<'a>" : "";
            let life_time_param_on_parent = if config_for_children.life_time_on_children {
                "<'a>"
            } else {
                ""
            };
            configs.life_time_on_children = false;
            lines.push(format!(
                r#"
#[allow(unused_lifetimes)]
#[derive(Clone, Debug)]
pub struct {packet_name_prefix}Packet{life_time_param_on_parent} {{
"#
            ));
            lines.extend(child_lines_rs_struct);
            lines.push(format!("\n}}\n"));
        }
        MethodFlavor::CFn => {
            let child_lines_c = process_children(
                entity,
                handlers,
                &mut HandlerConfigs {
                    // ask function handler to output trait style code
                    parameter_flavor: ParameterFlavor::MethodCallParam,
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
                "{full_spi_output_enum_name}::{enum_name}({packet_name_prefix}Packet {{\n",
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

pub fn find_previous_sibling_index(entity: &Entity, configs: &HandlerConfigs) -> usize {
    let current_name = entity.get_name().unwrap();
    let mut index = 0;

    if let Some(parent) = entity.get_lexical_parent() {
        let siblings = parent.get_children();

        // Ensure we only iterate up to the current entity's index as specified in configs
        for sibling in siblings.iter().take(configs.index + 1) {
            if sibling.get_kind() == entity.get_kind() {
                if sibling.get_name().unwrap() == current_name {
                    // Increment index for each sibling with the same name found
                    index += 1;
                }
            }
        }
    }
    index
}

fn get_rs_result_type_from_c_result_type(c_result_type: &str) -> String {
    match c_result_type {
        "void" => "()".to_string(),
        "int" => "std::os::raw::c_int".to_string(),
        "bool" => "bool".to_string(),
        "const char *" => "*const std::os::raw::c_char".to_string(),
        "YDQueryResult<char> *" => "*const YDQueryResult".to_string(),
        _ => {
            if c_result_type.starts_with("const ") && c_result_type.ends_with(" *") {
                let t = &c_result_type[6..c_result_type.len() - 2];
                format!("*const {}", t)
            } else {
                format!("/** {} */", c_result_type)
            }
        }
    }
}
