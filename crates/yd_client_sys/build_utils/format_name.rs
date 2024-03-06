use clang::{Entity, EntityKind};
use inflector::Inflector;

pub fn format_enum_name(name: &str) -> String {
    let clean_name = if name.starts_with('~') {
        name[1..].to_string()
    } else {
        name.to_string()
    };
    clean_name.to_pascal_case()
}

pub fn get_full_name_of_entity(e: &Entity) -> String {
    let mut v = vec![e.get_name().expect("/* name_error */")];
    let mut xe = Box::new(e.clone());
    while let Some(e) = xe.get_lexical_parent() {
        if e.get_kind() == EntityKind::TranslationUnit || e.get_kind() == EntityKind::NotImplemented
        {
            break;
        }
        v.push(e.get_name().expect("/* name_error */"));
        xe = Box::new(e);
    }
    v.reverse();
    v.join("_")
}
