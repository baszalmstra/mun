extern crate core;

pub use r#type::{
    Field, FieldInfo, HasStaticType, PointerType, StructType, StructTypeBuilder, Type, TypeKind,
};

mod cast;
pub mod diff;
pub mod gc;
pub mod mapping;
mod r#type;
pub mod type_table;
use thiserror::Error;

pub mod prelude {
    pub use crate::diff::{diff, Diff, FieldDiff, FieldEditKind};
    pub use crate::mapping::{Action, FieldMapping};
    pub use crate::r#type::{Field, PointerType, StructType, Type, TypeKind};
}

/// An error that can occur when trying to convert from an abi type to an internal type.
#[derive(Debug, Error)]
pub enum TryFromAbiError<'a> {
    #[error("unknown TypeId '{0}'")]
    UnknownTypeId(abi::TypeId<'a>),
}
