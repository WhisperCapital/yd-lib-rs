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
    C,
    /// return rust style parameter code
    Rust,
    /// each param as field in rust struct
    RustStruct,
    ///
    SpiFn,
    /// only add debug log
    None,
}

pub fn insert_function_parameter_handlers(handlers: &mut HandlerMap) {
    let parameter_types_to_handle = [TypeKind::Int, TypeKind::Bool, TypeKind::IncompleteArray];
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
    let entity_name = Inflector::to_snake_case(&entity.get_name().unwrap());

    let parameter_str = match entity_type.get_kind() {
        TypeKind::Pointer => {
            let parameter = get_pointer_parameter(&entity_type, &configs.parameter_flavor);
            format_parameter(&entity_name, &parameter, &configs.parameter_flavor)
        }
        TypeKind::Typedef => {
            let parameter = get_typedef_parameter(&entity_type, &configs.parameter_flavor);
            format_parameter(&entity_name, &parameter, &configs.parameter_flavor)
        }
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
        ParameterFlavor::C => format!("{name}"),
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

fn get_pointer_parameter(entity_type: &Type, flavor: &ParameterFlavor) -> String {
    let pointee_type = entity_type.get_pointee_type().unwrap();
    match pointee_type.get_kind() {
        TypeKind::CharS => match flavor {
            ParameterFlavor::C => " as *const i8".to_string(),
            ParameterFlavor::Rust | ParameterFlavor::RustStruct => "std::ffi::CString".to_string(),
            ParameterFlavor::SpiFn => "*const std::os::raw::c_char".to_string(),
            ParameterFlavor::None => "/* char* */".to_string(),
        },
        TypeKind::Pointer => {
            let inner_type = pointee_type.get_pointee_type().unwrap();
            match inner_type.get_kind() {
                TypeKind::CharS => match flavor {
                    ParameterFlavor::C => ".as_ptr() as *const *const i8".to_string(),
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
                    ParameterFlavor::C => format!(" as *mut {}", type_name),
                    ParameterFlavor::Rust | ParameterFlavor::RustStruct => {
                        format!("&mut {}", type_name)
                    }
                    ParameterFlavor::SpiFn => format!("{}.as_ref()", type_name),
                    ParameterFlavor::None => format!("/* {} */", type_name),
                }
            } else {
                panic!("Unhandled pointer type");
            }
        }
    }
}

fn get_typedef_parameter(entity_type: &Type, flavor: &ParameterFlavor) -> String {
    let underlying_type = entity_type
        .get_declaration()
        .unwrap()
        .get_typedef_underlying_type()
        .unwrap();

    match underlying_type.get_kind() {
        TypeKind::CharS => match flavor {
            ParameterFlavor::C => "*const std::os::raw::c_char".to_string(),
            ParameterFlavor::Rust | ParameterFlavor::RustStruct => {
                "std::os::raw::c_char".to_string()
            }
            ParameterFlavor::SpiFn => "as *const std::os::raw::c_char".to_string(),
            ParameterFlavor::None => "/* c_char */".to_string(),
        },
        TypeKind::Pointer => {
            let pointee_type = underlying_type.get_pointee_type().unwrap();
            match pointee_type.get_kind() {
                TypeKind::CharS => match flavor {
                    ParameterFlavor::C => "*const *const std::os::raw::c_char".to_string(),
                    ParameterFlavor::Rust | ParameterFlavor::RustStruct => {
                        "Vec<std::ffi::CString>".to_string()
                    }
                    ParameterFlavor::SpiFn => "as *const *const std::os::raw::c_char".to_string(),
                    ParameterFlavor::None => "/* char** */".to_string(),
                },
                _ => panic!("Unhandled pointer to pointer type in typedef"),
            }
        }
        _ => panic!("Unhandled typedef type"),
    }
}
