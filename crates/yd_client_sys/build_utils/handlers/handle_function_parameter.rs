use crate::build_utils::{
    config::HandlerConfigs, format_name::get_full_name_of_entity, Handler, HandlerMap
};
use clang::*;
use inflector::Inflector;

#[derive(Clone)]
pub enum ParameterFlavor {
    /// return c style parameter code
    C,
    /// return rust style parameter code
    Rust,
    /// only add debug log
    None,
}

pub fn insert_function_parameter_handlers(handlers: &mut HandlerMap) {
    let parameter_types_to_handle = [TypeKind::Int, TypeKind::Bool, TypeKind::IncompleteArray];
    for type_kind in parameter_types_to_handle {
        handlers.insert(type_kind, Handler::FunctionPrototype(Box::new(handle_function_parameter)));
    }
}

pub fn handle_function_parameter(
    entity: &Entity,
    handlers: &HandlerMap,
    configs: &mut HandlerConfigs,
) -> Vec<String> {
    let mut lines: Vec<String> = vec![];
    // there will be no child node here
    // transform c++ type to rust type
    let entity_type = entity.get_type().unwrap();
    // TODO: 似乎 SPI 和 API 需要生成不同的参数代码，需要加更多 configs flavor 来区分
    let (mut rust_type, c_type) = match entity_type.get_kind() {
        TypeKind::Pointer => {
            let tp = entity_type.get_pointee_type().unwrap();
            let d = tp.get_declaration();
            if let Some(d) = d {
                (
                    "&mut ".to_string() + &get_full_name_of_entity(&d),
                    " as * mut ".to_string() + &get_full_name_of_entity(&d),
                )
            } else {
                match tp.get_kind() {
                    // 这个是char*, register_front() 会有这样的参数
                    TypeKind::CharS => ("std::ffi::CString".to_string(), ".into_raw()".to_string()),
                    TypeKind::Pointer => {
                        // 这是char**
                        // mdapi.subscribe() 有这样的参数
                        let tp = tp.get_pointee_type().unwrap();
                        match tp.get_kind() {
                            TypeKind::CharS => (
                                "Vec<std::ffi::CString>".to_string(),
                                ".to_char_pp()".to_string(),
                            ),
                            _ => panic!(""),
                        }
                    }
                    _ => (tp.get_display_name(), "".to_string()),
                }
            }
        }
        TypeKind::Typedef => {
            match entity_type
                .get_declaration()
                .unwrap()
                .get_typedef_underlying_type()
                .unwrap()
                .get_kind()
            {
                TypeKind::CharS => ("std::os::raw::c_char".to_string(), "".to_string()),
                _ => {
                    // (tp.get_display_name(), "".to_string())
                    println!("tp={:?}", entity_type);
                    panic!("");
                }
            }
        }
        TypeKind::Int => ("std::os::raw::c_int".to_string(), "".to_string()),
        TypeKind::Bool => ("std::os::raw::c_bool".to_string(), "".to_string()),
        TypeKind::Enum => {
            let d = entity_type.get_declaration().unwrap();
            (get_full_name_of_entity(&d), "".to_string())
        }
        TypeKind::IncompleteArray => (
            "Vec<std::ffi::CString>".to_string(),
            ".iter().map(|cs| cs.as_ptr()).collect::<Vec<_>>().as_mut_ptr() as *mut *mut i8"
                .to_string(),
        ),
        _ => {
            println!("handle_function_parameter not handling {:?}", entity_type);
            panic!("");
        }
    };
    if rust_type == "int" {
        // 或者要转为 std::os::raw::c_int
        rust_type = "std::os::raw::c_int".to_string();
    }
    match configs.parameter_flavor {
        ParameterFlavor::C => {
            lines.push(format!(
                r#", {}{}"#,
                Inflector::to_snake_case(&entity.get_name().unwrap()),
                c_type
            ));
        }
        ParameterFlavor::Rust => {
            lines.push(format!(
                ", {}: {}",
                Inflector::to_snake_case(&entity.get_name().unwrap()),
                rust_type
            ));
        }
        ParameterFlavor::None => lines.push(format!(
            "/* Param: {} */",
            entity.get_display_name().unwrap_or_default()
        )),
    }

    lines
}
