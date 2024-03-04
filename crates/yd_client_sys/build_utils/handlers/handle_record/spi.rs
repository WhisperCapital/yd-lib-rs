use clang::*;
use inflector::Inflector;

use crate::build_utils::{
    config::HandlerConfigs, handle_function_prototype::MethodFlavor, process_children, HandlerMap,
};

pub fn handle_spi_record(
    entity: &Entity,
    handlers: &HandlerMap,
    configs: &mut HandlerConfigs,
    full_rust_struct_name: &str,
) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let vtable_struct_name = format!("{full_rust_struct_name}VTable");
    let full_trait_name = format!("{full_rust_struct_name}_trait");

    lines.extend(handle_trait(
        entity,
        handlers,
        configs,
        full_rust_struct_name,
        &full_trait_name,
    ));
    lines.extend(handle_vtable(
        entity,
        handlers,
        configs,
        full_rust_struct_name,
        &vtable_struct_name,
    ));
    lines.extend(handle_spi_output_enum(
        entity,
        handlers,
        configs,
        full_rust_struct_name,
    ));
    lines.extend(handle_output_enum_struct(entity, handlers, configs));
    lines.extend(handle_static_table(
        entity,
        handlers,
        configs,
        full_rust_struct_name,
    ));
    lines.extend(handle_c_fn(entity, handlers, configs));
    lines.push(handle_fat_spi(
        entity,
        handlers,
        configs,
        full_rust_struct_name,
        &vtable_struct_name,
        &full_trait_name,
    ));
    lines.push(handle_spi_stream_code(
        full_rust_struct_name,
        &format!("{full_rust_struct_name}Output"),
    ));

    lines
}


pub fn handle_trait(
    entity: &Entity,
    handlers: &HandlerMap,
    configs: &HandlerConfigs,
    full_rust_struct_name: &str,
    full_trait_name: &str,
) -> Vec<String> {
    let mut lines = Vec::new();
    lines.push(format!(r#"pub trait {full_trait_name}: Send {{"#));
    lines.extend(process_children(
        entity,
        handlers,
        &mut HandlerConfigs {
            method_flavor: MethodFlavor::Trait,
            ..configs.clone()
        },
    ));
    lines.push("}".to_string());
    lines
}

pub fn handle_vtable(
    entity: &Entity,
    handlers: &HandlerMap,
    configs: &HandlerConfigs,
    full_rust_struct_name: &str,
    vtable_struct_name: &str,
) -> Vec<String> {
    let mut lines = Vec::new();
    lines.push(format!(
        r#"
#[repr(C)]
#[derive(Debug)]
struct {vtable_struct_name} {{"#
    ));
    lines.extend(process_children(
        entity,
        handlers,
        &mut HandlerConfigs {
            method_flavor: MethodFlavor::Struct,
            ..configs.clone()
        },
    ));
    lines.push("}".to_string());
    lines
}

pub fn handle_spi_output_enum(
    entity: &Entity,
    handlers: &HandlerMap,
    configs: &HandlerConfigs,
    full_rust_struct_name: &str,
) -> Vec<String> {
    let mut lines = Vec::new();
    let full_spi_output_enum_name = format!("{full_rust_struct_name}Output");
    lines.push(format!(
        r#"
#[derive(Clone, Debug)]
pub enum {full_spi_output_enum_name} {{"#
    ));
    lines.extend(process_children(
        entity,
        handlers,
        &mut HandlerConfigs {
            method_flavor: MethodFlavor::OutputEnum,
            ..configs.clone()
        },
    ));
    lines.push("}".to_string());
    lines
}

pub fn handle_output_enum_struct(
    entity: &Entity,
    handlers: &HandlerMap,
    configs: &HandlerConfigs,
) -> Vec<String> {
    process_children(
        entity,
        handlers,
        &mut HandlerConfigs {
            method_flavor: MethodFlavor::OutputEnumStruct,
            ..configs.clone()
        },
    )
}

pub fn handle_static_table(
    entity: &Entity,
    handlers: &HandlerMap,
    configs: &HandlerConfigs,
    full_rust_struct_name: &str,
) -> Vec<String> {
    let mut lines = Vec::new();
    let full_static_vtable_var_name =
        Inflector::to_snake_case(full_rust_struct_name).to_uppercase() + "_VTABLE";
    let vtable_struct_name = format!("{full_rust_struct_name}VTable");
    lines.push(format!(
        r#"
static {full_static_vtable_var_name}: {vtable_struct_name} = {vtable_struct_name} {{"#
    ));
    lines.extend(process_children(
        entity,
        handlers,
        &mut HandlerConfigs {
            method_flavor: MethodFlavor::StaticTable,
            ..configs.clone()
        },
    ));
    lines.push("}".to_string());
    lines
}

pub fn handle_c_fn(
    entity: &Entity,
    handlers: &HandlerMap,
    configs: &HandlerConfigs,
) -> Vec<String> {
    process_children(
        entity,
        handlers,
        &mut HandlerConfigs {
            method_flavor: MethodFlavor::CFn,
            ..configs.clone()
        },
    )
}

pub fn handle_fat_spi(
    entity: &Entity,
    handlers: &HandlerMap,
    configs: &HandlerConfigs,
    full_rust_struct_name: &str,
    vtable_struct_name: &str,
    full_trait_name: &str,
) -> String {
    format!(
        r#"
#[repr(C)]
pub struct {full_rust_struct_name}Fat {{
  vtable: *const {vtable_struct_name},
  pub md_spi_ptr: *mut dyn {full_trait_name},
}}
"#
    )
}

pub fn handle_spi_stream_code(full_spi_name: &str, full_spi_output_enum_name: &str) -> String {
    format!(
        r#"
  use futures::stream::Stream;
  use std::{{
      pin::Pin,
      sync::{{Arc, Mutex}},
      task::Waker,
  }};

  struct {full_spi_name}Inner {{
      buf: std::collections::VecDeque<{full_spi_output_enum_name}>,
      waker: Option<Waker>,
  }}

  impl {full_spi_name}Inner {{
      fn push(&mut self, msg: {full_spi_output_enum_name}) {{
          self.buf.push_back(msg);
          if let Some(ref waker) = &self.waker {{
              waker.clone().wake()
          }}
      }}
  }}

  pub struct {full_spi_name}Stream {{
      inner: Arc<Mutex<{full_spi_name}Inner>>,
  }}

  impl Stream for {full_spi_name}Stream {{
      type Item = {full_spi_output_enum_name};

      fn poll_next(
          self: Pin<&mut Self>,
          cx: &mut futures::task::Context<'_>,
      ) -> futures::task::Poll<Option<Self::Item>> {{
          use futures::task::Poll;
          let mut inner = self.inner.lock().unwrap();
          if let Some(i) = inner.buf.pop_front() {{
              Poll::Ready(Some(i))
          }} else {{
              inner.waker = Some(cx.waker().clone());
              Poll::Pending
          }}
      }}

      fn size_hint(&self) -> (usize, Option<usize>) {{
          (0, None)
      }}
  }}

  pub fn create_spi() -> (Box<{full_spi_name}Stream>, *mut {full_spi_name}Stream) {{
      let i = {full_spi_name}Inner {{
          buf: std::collections::VecDeque::new(),
          waker: None,
      }};
      let xspi = {full_spi_name}Stream {{
          inner: Arc::new(Mutex::new(i)),
      }};
      let myspi = Box::new(xspi);
      let pp = Box::into_raw(myspi);
      let pp2 = pp.clone();
      (unsafe {{ Box::from_raw(pp2) }}, pp)
  }}
  "#,
    )
}

pub fn handle_spi_fn(
  entity: &Entity,
  handlers: &HandlerMap,
  configs: &HandlerConfigs,
  full_trait_name: &str,
  full_rust_struct_name: &str,
) -> Vec<String> {
  let mut lines = Vec::new();
  lines.push(format!(
    r#"impl {full_trait_name} for {full_rust_struct_name}Stream {{"#,
  ));
  lines.extend(process_children(
      entity,
      handlers,
      &mut HandlerConfigs {
          method_flavor: MethodFlavor::SpiFn,
          ..configs.clone()
      },
  ));
  lines.push("}".to_string());
  lines
}
