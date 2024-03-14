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
    let mut v = vec![e.get_name().unwrap_or_else(|| "".to_string())];
    let mut xe = Box::new(e.clone());
    while let Some(e) = xe.get_lexical_parent() {
        if e.get_kind() == EntityKind::TranslationUnit || e.get_kind() == EntityKind::NotImplemented {
            break;
        }
        if let Some(name) = e.get_name() {
            v.push(name);
        }
        xe = Box::new(e);
    }
    v.reverse();
    v.iter().filter(|name| !name.is_empty()).cloned().collect::<Vec<_>>().join("_")
}
