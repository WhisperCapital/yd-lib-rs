use crate::build_utils::{
    config::HandlerConfigs, format_name::get_full_name_of_entity, Handler, HandlerMap,
};
use clang::*;
use inflector::Inflector;

lazy_static! {
    static ref INDENT: String = "    ".to_string();
}

#[derive(Clone)]
pub enum ParameterFlavor {
    /// return c style parameter code
    MethodCallParam,
    /// return rust style parameter code
    Rust,
    /// each param as field in rust struct
    RustStruct,
    ///
    SpiFn,
    /// only add debug log
    None,
}

macro_rules! console_debug {
    ($($tokens: tt)*) => {
        println!("cargo:warning={}", format!($($tokens)*))
    }
}

pub fn insert_function_parameter_handlers(handlers: &mut HandlerMap) {
    let parameter_types_to_handle = [
        TypeKind::Bool,
        TypeKind::Double,
        TypeKind::Elaborated,
        TypeKind::IncompleteArray,
        TypeKind::Int,
        TypeKind::Pointer,
        TypeKind::Typedef,
        TypeKind::UInt,
        TypeKind::ULongLong,
        TypeKind::Enum,
    ];
    for type_kind in parameter_types_to_handle {
        handlers.insert(
            type_kind,
            Handler::FunctionPrototype(Box::new(handle_function_parameter)),
        );
    }
}
pub fn handle_function_parameter(
    entity: &Entity,
    handlers: &HandlerMap,
    configs: &mut HandlerConfigs,
) -> Vec<String> {
    let entity_type = entity.get_type().unwrap();
    let entity_name = if entity.get_name().unwrap() == "type" {
        "type_".to_string()  // Rename "type" to "type_"
    } else {
        Inflector::to_snake_case(&entity.get_name().unwrap())
    };
    // console_debug!(
    //     "handle_function_parameter {:?} {:?}",
    //     entity_name,
    //     entity_type.get_kind()
    // );

    let parameter_str = match entity_type.get_kind() {
        TypeKind::Pointer => {
            let parameter = get_pointer_parameter(&entity_type, configs);
            format_parameter(&entity_name, &parameter, &configs.parameter_flavor)
        }
        TypeKind::Typedef => {
            let parameter = get_typedef_parameter(&entity_type, configs);
            format_parameter(&entity_name, &parameter, &configs.parameter_flavor)
        }
        TypeKind::UInt => format_parameter(
            &entity_name,
            "std::os::raw::c_uint",
            &configs.parameter_flavor,
        ),
        TypeKind::Double => format_parameter(&entity_name, "f64", &configs.parameter_flavor),
        TypeKind::ULongLong => format_parameter(&entity_name, "u64", &configs.parameter_flavor),
        TypeKind::Int => format_parameter(
            &entity_name,
            "std::os::raw::c_int",
            &configs.parameter_flavor,
        ),
        TypeKind::Bool => format_parameter(&entity_name, "bool", &configs.parameter_flavor),
        TypeKind::Enum => {
            let d = entity_type.get_declaration().unwrap();
            let rust_type = get_full_name_of_entity(&d);
            format_parameter(&entity_name, &rust_type, &configs.parameter_flavor)
        }
        TypeKind::IncompleteArray => format_parameter(
            &entity_name,
            "Vec<std::ffi::CString>",
            &configs.parameter_flavor,
        ),
        _ => {
            println!("handle_function_parameter not handling {:?}", entity_type);
            panic!("");
        }
    };

    let is_only_child = configs.num_parent_children_same_handler == 1;
    let is_first_child = configs.index == 0;
    let is_last_child = configs.index == configs.num_parent_children_same_handler - 1;
    if is_only_child {
        match configs.parameter_flavor {
            ParameterFlavor::SpiFn => {
                vec![parameter_str, ",\n".to_string()]
            }
            _ => vec![parameter_str],
        }
    } else {
        match configs.parameter_flavor {
            ParameterFlavor::None => vec!["/* ,*/".to_string(), parameter_str],
            ParameterFlavor::RustStruct | ParameterFlavor::SpiFn => {
                vec![parameter_str, ",\n".to_string()]
            }
            _ => {
                if is_first_child {
                    vec![parameter_str]
                } else {
                    vec![", ".to_string(), parameter_str]
                }
            }
        }
    }
}

fn format_parameter(name: &str, parameter: &str, flavor: &ParameterFlavor) -> String {
    match flavor {
        ParameterFlavor::MethodCallParam => format!("{name}"),
        ParameterFlavor::Rust => format!("{name}: {parameter}"),
        ParameterFlavor::SpiFn => {
            format!("{indent}{indent}{indent}{name}: {name}", indent = *INDENT)
        }
        ParameterFlavor::RustStruct => {
            format!("{indent}pub {name}: {parameter}", indent = *INDENT)
        }
        ParameterFlavor::None => format!("/* Param: {name} */"),
    }
}

fn get_pointer_parameter(
    entity_type: &Type,
    configs: &mut HandlerConfigs,
) -> String {
    let pointee_type = entity_type.get_pointee_type().unwrap();
    let flavor = &configs.parameter_flavor;
    match pointee_type.get_kind() {
        TypeKind::CharS => match flavor {
            ParameterFlavor::MethodCallParam => " as *const i8".to_string(),
            ParameterFlavor::Rust | ParameterFlavor::RustStruct => "std::ffi::CString".to_string(),
            ParameterFlavor::SpiFn => "*const std::os::raw::c_char".to_string(),
            ParameterFlavor::None => "/* char* */".to_string(),
        },
        TypeKind::UChar => match flavor {
            ParameterFlavor::MethodCallParam => " as *const u8".to_string(),
            ParameterFlavor::Rust | ParameterFlavor::RustStruct => "u8".to_string(),
            ParameterFlavor::SpiFn => "*const std::os::raw::c_uchar".to_string(),
            ParameterFlavor::None => "/* unsigned char* */".to_string(),
        },
        TypeKind::Pointer => {
            let inner_type = pointee_type.get_pointee_type().unwrap();
            match inner_type.get_kind() {
                TypeKind::CharS => match flavor {
                    ParameterFlavor::MethodCallParam => ".as_ptr() as *const *const i8".to_string(),
                    ParameterFlavor::Rust | ParameterFlavor::RustStruct => "Vec<std::ffi::CString>".to_string(),
                    ParameterFlavor::SpiFn => ".iter().map(|s| s.as_ptr()).collect::<Vec<_>>().as_ptr() as *const *const i8".to_string(),
                    ParameterFlavor::None => "/* char** */".to_string(),
                },
                _ => panic!("Unhandled pointer to pointer type"),
            }
        }
        _ => {
            if let Some(decl) = pointee_type.get_declaration() {
                let type_name = get_full_name_of_entity(&decl);
                match flavor {
                    ParameterFlavor::MethodCallParam => format!(" as *const {}", type_name),
                    ParameterFlavor::Rust | ParameterFlavor::RustStruct => {
                        configs.life_time_on_children = true;
                        format!("&{}{}", configs.life_time, type_name)
                    }
                    ParameterFlavor::SpiFn => format!("&{}", type_name),
                    ParameterFlavor::None => format!("/* {} */", type_name),
                }
            } else if pointee_type.get_kind() == TypeKind::Elaborated {
                let type_name = pointee_type.get_display_name();
                match flavor {
                    ParameterFlavor::MethodCallParam => {
                        if entity_type.is_const_qualified() {
                            format!(" as *const {}", type_name)
                        } else {
                            format!(" as *mut {}", type_name)
                        }
                    }
                    ParameterFlavor::Rust | ParameterFlavor::RustStruct => {
                        if entity_type.is_const_qualified() {
                            format!("&{}", type_name)
                        } else {
                            format!("&mut {}", type_name)
                        }
                    }
                    ParameterFlavor::SpiFn => format!("&{}", type_name),
                    ParameterFlavor::None => format!("/* {} */", type_name),
                }
            } else {
                panic!("Unhandled pointer type: {:?}", pointee_type.get_kind());
            }
        }
    }
}

fn get_typedef_parameter(entity_type: &Type, configs: &mut HandlerConfigs) -> String {
    let underlying_type = entity_type
        .get_declaration()
        .unwrap()
        .get_typedef_underlying_type()
        .unwrap();
    let flavor = &configs.parameter_flavor;
    match underlying_type.get_kind() {
        TypeKind::CharS => match flavor {
            ParameterFlavor::MethodCallParam => "*const std::os::raw::c_char".to_string(),
            ParameterFlavor::Rust | ParameterFlavor::RustStruct => {
                "std::os::raw::c_char".to_string()
            }
            ParameterFlavor::SpiFn => "as *const std::os::raw::c_char".to_string(),
            ParameterFlavor::None => "/* c_char */".to_string(),
        },
        TypeKind::Pointer => get_pointer_parameter(&underlying_type, configs), // Delegate to the pointer handler
        TypeKind::Int => match flavor {
            ParameterFlavor::MethodCallParam
            | ParameterFlavor::Rust
            | ParameterFlavor::RustStruct => "i32".to_string(),
            ParameterFlavor::SpiFn => "as i32".to_string(),
            ParameterFlavor::None => "/* int */".to_string(),
        },
        TypeKind::Bool => match flavor {
            ParameterFlavor::MethodCallParam
            | ParameterFlavor::Rust
            | ParameterFlavor::RustStruct => "bool".to_string(),
            ParameterFlavor::SpiFn => "as bool".to_string(),
            ParameterFlavor::None => "/* bool */".to_string(),
        },
        TypeKind::Elaborated | TypeKind::Record => {
            // Handle user-defined types, structs, and unions
            let decl = underlying_type.get_declaration().unwrap();
            let type_name = get_full_name_of_entity(&decl);
            match flavor {
                ParameterFlavor::MethodCallParam => format!("*mut {}", type_name),
                ParameterFlavor::Rust | ParameterFlavor::RustStruct => type_name,
                ParameterFlavor::SpiFn => format!("as *mut {}", type_name),
                ParameterFlavor::None => format!("/* {} */", type_name),
            }
        }
        TypeKind::ConstantArray => {
            let array_type = underlying_type.get_element_type().unwrap();
            let size = underlying_type.get_size().unwrap();
            match array_type.get_kind() {
                TypeKind::CharS => match flavor {
                    ParameterFlavor::MethodCallParam
                    | ParameterFlavor::Rust
                    | ParameterFlavor::RustStruct => format!("[std::os::raw::c_char; {}]", size),
                    ParameterFlavor::SpiFn => format!("as *const [std::os::raw::c_char; {}]", size),
                    ParameterFlavor::None => "/* char array */".to_string(),
                },
                TypeKind::Int => match flavor {
                    ParameterFlavor::MethodCallParam
                    | ParameterFlavor::Rust
                    | ParameterFlavor::RustStruct => format!("[i32; {}]", size),
                    ParameterFlavor::SpiFn => format!("as *const [i32; {}]", size),
                    ParameterFlavor::None => "/* int array */".to_string(),
                },
                // Add other type cases as needed
                _ => panic!(
                    "Unhandled constant array element type: {:?}",
                    array_type.get_kind()
                ),
            }
        }
        TypeKind::LongLong => match flavor {
            ParameterFlavor::MethodCallParam
            | ParameterFlavor::Rust
            | ParameterFlavor::RustStruct => {
                if underlying_type.is_const_qualified() {
                    "const i64".to_string() // or "const u64" if it's unsigned
                } else {
                    "i64".to_string() // or "u64" if it's unsigned
                }
            }
            ParameterFlavor::SpiFn => "as i64".to_string(), // or "as u64" if it's unsigned
            ParameterFlavor::None => "/* i64 */".to_string(), // or "/* u64 */" if it's unsigned
        },
        _ => panic!("Unhandled typedef type: {:?}", underlying_type.get_kind()),
    }
}
