use clang::{Entity, EntityKind};

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
