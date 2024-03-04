use crate::build_utils::{
    config::HandlerConfigs, format_name::get_full_name_of_entity, HandlerMap,
};
use clang::*;

pub mod spi;
use spi::handle_spi_record;

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
    if full_rust_struct_name == "YDListener" {
        lines.extend(handle_spi_record(entity, handlers, configs, &full_rust_struct_name));
    }
    lines
}
