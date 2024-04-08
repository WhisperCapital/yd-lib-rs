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

#[derive(Clone, Debug)]
pub enum MethodFlavor {
    /// method is in a struct
    VTableStruct,
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
    let camel_case_name = Inflector::to_camel_case(&snake_fn_name).replace("Id", "ID");

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
            let config_for_children = &mut HandlerConfigs {
                // ask function handler to output trait style code
                parameter_flavor: ParameterFlavor::Rust,
                life_time: "'a ".to_string(),
                ..configs.clone()
            };
            let child_lines_spi_param = process_children(entity, handlers, config_for_children);
            lines.push(format!("{}fn {snake_fn_name}(&mut self", *INDENT));
            if !child_lines_spi_param.is_empty() {
                lines.push(format!(", "));
            }
            lines.extend(child_lines_spi_param);
            lines.push(format!(") {{}}\n"));
        }
        MethodFlavor::ApiTrait => {
            if raw_camel_case_name.starts_with("~") {
                // TODO: 在别处处理类的析构
                return lines;
            }
            if raw_camel_case_name == "start" {
                /*
                 * 使用我们包装过的 Trait，而不是原生的 Listener 类
                 * 并加入定制的逻辑
                 */
                lines.push(format!(
                    r#"
    pub fn start(&mut self, p_listener: *const dyn YDListenerTrait) -> bool {{
        let p_listener = Box::into_raw(Box::new((&YD_LISTENER_VTABLE, p_listener)));
        unsafe {{
            ((*(*self).vtable_).YDApi_start)(self as *mut YDApi, p_listener as *mut YDListener)
        }}
    }}
"#,));
                return lines;
            }
            if raw_camel_case_name.contains("Multi") {
                /*
                 * 跳过 `insert_multi_orders cancel_multi_orders insert_multi_quotes cancel_multi_quotes` 直到了解正确的类型转换方式
                 */
                lines.push(format!(
                    "{}// {snake_fn_name} // Ignored (MethodFlavor::ApiTrait)\n",
                    *INDENT
                ));
                return lines;
            }
            lines.push(format!("{}pub fn {snake_fn_name}(&mut self", *INDENT));
            if !child_lines_rs.is_empty() {
                lines.push(format!(", "));
            }
            lines.extend(child_lines_rs);
            let c_result_type = entity.get_result_type().unwrap().get_display_name();
            let rust_result_type = get_rs_result_type_from_c_result_type(&c_result_type);
            let full_api_record_name = format!("{record_name}_{camel_case_name}");
            let child_lines_c_method_call_param = process_children(
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
            if !child_lines_c_method_call_param.is_empty() {
                lines.push(format!(", "));
            }
            lines.extend(child_lines_c_method_call_param);
            lines.push(format!(
                r#")
        }}
    }}
"#
            ));
        }
        MethodFlavor::VTableStruct => {
            lines.push(format!(
                r#"{}{snake_fn_name}: extern "C" fn(spi: *mut {record_name}Fat"#,
                *INDENT
            ));
            let child_lines_rs_v_table = process_children(
                entity,
                handlers,
                &mut HandlerConfigs {
                    parameter_flavor: ParameterFlavor::Rust,
                    prefer_pointer: true,
                    ..configs.clone()
                },
            );
            if !child_lines_rs_v_table.is_empty() {
                lines.push(format!(", "));
            }
            lines.extend(child_lines_rs_v_table);
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
            process_children(entity, handlers, config_for_children);
            let life_time_param_on_parent = if config_for_children.life_time_on_children {
                "<'a>"
            } else {
                ""
            };
            configs.life_time_on_children = false;
            lines.push(format!(
                "{}{enum_name}({packet_name_prefix}Packet{life_time_param_on_parent}),\n",
                *INDENT
            ));
        }
        MethodFlavor::OutputEnumStruct => {
            let config_for_children = &mut HandlerConfigs {
                // ask function handler to output trait style code
                parameter_flavor: ParameterFlavor::RustStruct,
                life_time: "'a ".to_string(),
                ..configs.clone()
            };
            let child_lines_rs_struct = process_children(entity, handlers, config_for_children);
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
#[derive(Clone)]
pub struct {packet_name_prefix}Packet{life_time_param_on_parent} {{
"#
            ));
            lines.extend(child_lines_rs_struct);
            lines.push(format!("\n}}\n"));
        }
        MethodFlavor::CFn => {
            let child_lines_rs_c_fn = process_children(
                entity,
                handlers,
                &mut HandlerConfigs {
                    // ask function handler to output trait style code
                    parameter_flavor: ParameterFlavor::Rust,
                    prefer_pointer: true,
                    ..configs.clone()
                },
            );
            lines.push(format!(
                r#"
extern "C" fn spi_{snake_fn_name}(spi: *mut {record_name}Fat"#
            ));
            if !child_lines_rs_c_fn.is_empty() {
                lines.push(format!(", "));
            }
            lines.extend(child_lines_rs_c_fn);
            // add pointer check
            let child_lines_unsafe_c_check = process_children(
                entity,
                handlers,
                &mut HandlerConfigs {
                    // ask function handler to output trait style code
                    parameter_flavor: ParameterFlavor::UnsafeCheck,
                    ..configs.clone()
                },
            );
            lines.push(format!(
                r#") {{
    unsafe {{
        assert!(!spi.is_null());
"#
            ));
            lines.extend(child_lines_unsafe_c_check);
            lines.push(format!(
                r#"
        (*(*spi).md_spi_ptr).{snake_fn_name}("#,
            ));
            let child_lines_c = process_children(
                entity,
                handlers,
                &mut HandlerConfigs {
                    // ask function handler to output trait style code
                    parameter_flavor: ParameterFlavor::MethodCallParam,
                    prefer_pointer: true,
                    ..configs.clone()
                },
            );
            lines.extend(child_lines_c);
            lines.push(format!(
                r#")
    }}
}}
"#
            ));
        }
        MethodFlavor::SpiFn => {
            let config_for_children = &mut HandlerConfigs {
                // ask function handler to output trait style code
                parameter_flavor: ParameterFlavor::Rust,
                life_time: "'a ".to_string(),
                ..configs.clone()
            };
            let child_lines_rs_param = process_children(entity, handlers, config_for_children);
            lines.push(format!("{}fn {snake_fn_name}(&mut self", *INDENT));
            if !child_lines_rs_param.is_empty() {
                lines.push(format!(", "));
            }
            lines.extend(child_lines_rs_param.clone());
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
