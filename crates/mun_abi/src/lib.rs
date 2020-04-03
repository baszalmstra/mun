//! The Mun ABI
//!
//! The Mun ABI defines the binary format used to communicate between the Mun Compiler and Mun
//! Runtime.
#![warn(missing_docs)]

// Bindings can be manually generated by running `cargo gen-abi`.
mod autogen;
mod autogen_impl;

pub use autogen::*;

/// The Mun ABI prelude
///
/// The *prelude* contains imports that are used almost every time.
pub mod prelude {
    pub use crate::autogen::*;
    pub use crate::{Privacy, StructMemoryKind, TypeGroup};
}

/// Represents the kind of memory management a struct uses.
#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StructMemoryKind {
    /// A garbage collected struct is allocated on the heap and uses reference semantics when passed
    /// around.
    GC,

    /// A value struct is allocated on the stack and uses value semantics when passed around.
    ///
    /// NOTE: When a value struct is used in an external API, a wrapper is created that _pins_ the
    /// value on the heap. The heap-allocated value needs to be *manually deallocated*!
    Value,
}

impl Default for StructMemoryKind {
    fn default() -> Self {
        StructMemoryKind::GC
    }
}

impl From<StructMemoryKind> for u64 {
    fn from(kind: StructMemoryKind) -> Self {
        match kind {
            StructMemoryKind::GC => 0,
            StructMemoryKind::Value => 1,
        }
    }
}

/// Represents the privacy level of modules, functions, or variables.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Privacy {
    /// Publicly (and privately) accessible
    Public = 0,
    /// Privately accessible
    Private = 1,
}

/// Represents a group of types that illicit the same characteristics.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum TypeGroup {
    /// Fundamental types (i.e. `()`, `bool`, `float`, `int`, etc.)
    FundamentalTypes = 0,
    /// Struct types (i.e. record, tuple, or unit structs)
    StructTypes = 1,
    /// Array types (i.e. [int], [bool])
    ArrayTypes = 2,
}

impl TypeGroup {
    /// Returns whether this is a fundamental type.
    pub fn is_fundamental(self) -> bool {
        match self {
            TypeGroup::FundamentalTypes => true,
            _ => false,
        }
    }

    /// Returns whether this is a struct type.
    pub fn is_struct(self) -> bool {
        match self {
            TypeGroup::StructTypes => true,
            _ => false,
        }
    }

    /// Returns whether this is a array type.
    pub fn is_array(self) -> bool {
        match self {
            TypeGroup::ArrayTypes => true,
            _ => false,
        }
    }
}
