use crate::build_utils::{
    config::HandlerConfigs, format_name::get_full_name_of_entity, HandlerMap,
};
use clang::*;

pub mod spi;
use spi::handle_spi_record;
pub mod api;
use api::handle_api_record;

#[derive(Clone, Debug)]
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
    let record_name = entity.get_display_name().unwrap_or_default();
    let mut lines: Vec<String> = vec![];
    configs.record_name = record_name;
    let full_rust_struct_name = get_full_name_of_entity(&entity);
    match configs.record_flavor {
        RecordFlavor::SPI => {
            if full_rust_struct_name == "YDListener" {
                lines.extend(handle_spi_record(
                    entity,
                    handlers,
                    configs,
                    &full_rust_struct_name,
                ));
            }
        }
        RecordFlavor::API => {
            if full_rust_struct_name == "YDApi" {
                lines.extend(handle_api_record(
                    entity,
                    handlers,
                    configs,
                    &full_rust_struct_name,
                ));
            }
        }
        RecordFlavor::None => {
            // add format!("// Record: {}\n", record_name)
            lines.push(format!("\n// Record: {}\n", configs.record_name));
        }
    }
    lines
}
