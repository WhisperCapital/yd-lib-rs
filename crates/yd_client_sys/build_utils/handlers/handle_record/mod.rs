use crate::build_utils::{
    config::HandlerConfigs, format_name::get_full_name_of_entity, HandlerMap,
};
use clang::*;

pub mod spi;
use spi::*;

#[derive(Clone)]
pub enum RecordFlavor {
    /// Callback, need to generate a trait, stream, v-table...
    SPI,
    /// Generate safe wrapper around C++ API
    API,
    /// Log only
    None,
}

pub fn handle_record(
    entity: &Entity,
    handlers: &HandlerMap,
    configs: &mut HandlerConfigs,
) -> Vec<String> {
    let mut lines: Vec<String> = vec![format!(
        "// Record: {}",
        entity.get_display_name().unwrap_or_default()
    )];
    let full_rust_struct_name = get_full_name_of_entity(&entity);
    if full_rust_struct_name != "YDListener" {
        return lines;
    }
    let vtable_struct_name = format!("{full_rust_struct_name}VTable");
    let full_trait_name = format!("{full_rust_struct_name}_trait");

    lines.extend(handle_trait(
        entity,
        handlers,
        configs,
        &full_rust_struct_name,
        &full_trait_name,
    ));
    lines.extend(handle_vtable(
        entity,
        handlers,
        configs,
        &full_rust_struct_name,
        &vtable_struct_name,
    ));
    lines.extend(handle_spi_output_enum(
        entity,
        handlers,
        configs,
        &full_rust_struct_name,
    ));
    lines.extend(handle_output_enum_struct(entity, handlers, configs));
    lines.extend(handle_static_table(
        entity,
        handlers,
        configs,
        &full_rust_struct_name,
    ));
    lines.extend(handle_c_fn(entity, handlers, configs));
    lines.push(handle_fat_spi(
        entity,
        handlers,
        configs,
        &full_rust_struct_name,
        &vtable_struct_name,
        &full_trait_name,
    ));
    lines.push(handle_spi_stream_code(
        &full_rust_struct_name,
        &format!("{full_rust_struct_name}Output"),
    ));

    lines
}
