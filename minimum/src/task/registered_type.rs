

/// An ID for types, with type name for debug purposes
/// Wrap in a struct just to make it easy to attach debug info (i.e. task name)
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct RegisteredType {
    type_id: core::any::TypeId,
    type_name: &'static str,
}

impl RegisteredType {
    pub fn of<T : 'static>() -> Self {
        RegisteredType {
            type_id: core::any::TypeId::of::<T>(),
            type_name: core::any::type_name::<T>()
        }
    }
}

