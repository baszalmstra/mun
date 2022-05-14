//! Defines a helper struct `MunArrayValue` which wraps an inkwell value and represents a pointer to
//! a heap allocated Mun array struct.
//!
//! Mun arrays are represented on the heap as:
//!
//! ```c
//! struct Obj {
//!     ArrayValueT *value;
//!     ...
//! }
//!
//! struct ArrayValueT {
//!     usize_t len;
//!     usize_t capacity;
//!     T elements[capacity];
//! }
//! ```

use inkwell::builder::Builder;
use inkwell::types::{BasicTypeEnum, IntType, StructType};
use inkwell::values::{BasicValueEnum, IntValue, PointerValue};
use std::ffi::CStr;

/// A helper struct that wraps a PointerValue which points to an in memory Mun array value.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct MunArrayValue<'ink>(PointerValue<'ink>);

impl<'ink> MunArrayValue<'ink> {
    /// Constructs a new instance from an inkwell PointerValue without checking if this is actually
    /// a pointer to an array.
    pub fn from_ptr_unchecked(ptr: PointerValue<'ink>) -> Self {
        Self(ptr)
    }

    /// Returns the name of the array
    pub fn get_name(&self) -> &CStr {
        self.0.get_name()
    }

    /// Generate code to get to the array value
    fn get_array_ptr(&self, builder: &Builder<'ink>) -> PointerValue<'ink> {
        let value_name = self.0.get_name().to_string_lossy();
        let array_value_ptr = builder
            .build_struct_gep(self.0, 0, &format!("{}.value_ptr", &value_name))
            .expect("could not get array_value_ptr");
        builder
            .build_load(array_value_ptr, &format!("{}.value", &value_name))
            .into_pointer_value()
    }

    /// Generate code to fetch the length of the array.
    pub fn get_length_ptr(&self, builder: &Builder<'ink>) -> PointerValue<'ink> {
        let value_name = self.0.get_name().to_string_lossy();
        let array_ptr = self.get_array_ptr(builder);
        builder
            .build_struct_gep(array_ptr, 0, &format!("{}.length_ptr", &value_name))
            .expect("could not get `length` from array struct")
    }

    /// Generate code to fetch the capacity of the array.
    pub fn get_capacity(&self, builder: &Builder<'ink>) -> IntValue<'ink> {
        let value_name = self.0.get_name().to_string_lossy();
        let length_ptr = builder
            .build_struct_gep(self.0, 1, &format!("{}.capacity_ptr", &value_name))
            .expect("could not get `length` from array struct");
        builder
            .build_load(length_ptr, &format!("{}.capacity", &value_name))
            .into_int_value()
    }

    /// Generate code to a pointer to the elements stored in the array.
    pub fn get_elements(&self, builder: &Builder<'ink>) -> PointerValue<'ink> {
        let value_name = self.0.get_name().to_string_lossy();
        let array_ptr = self.get_array_ptr(builder);
        builder
            .build_struct_gep(array_ptr, 2, &format!("{}.elements_ptr", &value_name))
            .expect("could not get `elements` from array struct")
    }

    /// Returns the type of the `length` field
    pub fn length_ty(&self) -> IntType {
        self.struct_ty()
            .get_field_type_at_index(0)
            .expect("an array must have a second field")
            .into_int_type()
    }

    /// Returns the type of the `length` field
    pub fn capacity_ty(&self) -> IntType {
        self.struct_ty()
            .get_field_type_at_index(1)
            .expect("an array must have a second field")
            .into_int_type()
    }

    /// Returns the type of the elements stored in this array
    pub fn element_ty(&self) -> BasicTypeEnum<'ink> {
        self.struct_ty()
            .get_field_type_at_index(2)
            .expect("an array must have a second field")
    }

    /// Returns the type of the array struct that this instance points to
    fn struct_ty(&self) -> StructType<'ink> {
        self.0
            .get_type()
            .get_element_type()
            .into_struct_type()
            .get_field_type_at_index(0)
            .expect("could not get array value type")
            .into_pointer_type()
            .get_element_type()
            .into_struct_type()
    }
}

impl<'ink> From<MunArrayValue<'ink>> for BasicValueEnum<'ink> {
    fn from(value: MunArrayValue<'ink>) -> Self {
        value.0.into()
    }
}

impl<'ink> From<MunArrayValue<'ink>> for PointerValue<'ink> {
    fn from(value: MunArrayValue<'ink>) -> Self {
        value.0
    }
}
