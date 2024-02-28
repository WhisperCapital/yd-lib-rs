use clang::*;
use inflector::Inflector;

use crate::build_utils::{
    config::HandlerConfigs, format_name::get_full_name_of_entity,
    handle_function_prototype::MethodFlavor, process_children, HandlerMap,
};

pub fn handle_record(
    entity: &Entity,
    handlers: &HandlerMap,
    configs: &mut HandlerConfigs,
) -> Vec<String> {
    let mut lines: Vec<String> = vec![format!(
        "// Record: {}",
        entity.get_display_name().unwrap_or_default()
    )];
    // building safe rust binding for class
    let full_rust_struct_name = get_full_name_of_entity(&entity);
    // TODO: currently only parse the SPI, do this for more structs like API later
    if full_rust_struct_name != "YDListener" {
        return lines;
    }
    // handle spi callbacks
    // first lines are trait
    let full_trait_name = format!("{full_rust_struct_name}_trait",);
    lines.push(format!(r#"pub trait {full_trait_name}: Send {{"#,));
    // trait methods
    lines.extend(process_children(
        entity,
        handlers,
        &mut HandlerConfigs {
            // ask function handler to output trait style code
            method_flavor: MethodFlavor::Trait,
            ..configs.clone()
        },
    ));
    lines.push("}".to_string());
    // end trait
    // next are virtual tables
    let vtable_struct_name = format!("{full_rust_struct_name}VTable");
    lines.push(format!(
        r#"
#[repr(C)]
#[derive(Debug)]
struct {vtable_struct_name} {{
"#
    ));
    lines.extend(process_children(
        entity,
        handlers,
        &mut HandlerConfigs {
            // ask function handler to output v-table struct style code
            method_flavor: MethodFlavor::Struct,
            ..configs.clone()
        },
    ));
    lines.push("}".to_string());
    // end virtual tables
    // next are spi output enum
    let full_spi_output_enum_name = format!("{full_rust_struct_name}Output");
    let full_static_vtable_var_name =
        Inflector::to_snake_case(&full_rust_struct_name).to_uppercase() + "_VTABLE";

    lines
}
