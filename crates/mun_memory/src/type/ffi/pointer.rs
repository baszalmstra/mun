use std::ffi::c_void;
use std::mem::ManuallyDrop;
use std::ops::Deref;
use std::sync::Arc;

use capi_utils::{mun_error_try, try_deref_mut, ErrorHandle};

use crate::r#type::{PointerInfo, Type as RustType, TypeDataStore};

use super::Type;

/// Additional information of a pointer [`Type`].
///
/// Ownership of this type lies with the [`Type`] that created this instance. As long as the
/// original type is not released through [`mun_type_release`] this type stays alive.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct PointerType(pub(super) *const c_void, pub(super) *const c_void);

impl<'t> From<super::super::PointerType<'t>> for PointerType {
    fn from(ty: super::super::PointerType<'t>) -> Self {
        PointerType(
            (ty.inner as *const PointerInfo).cast(),
            (&ty.store as *const &Arc<TypeDataStore>).cast(),
        )
    }
}

impl PointerType {
    /// Returns the store associated with this instance
    unsafe fn store(&self) -> Result<ManuallyDrop<Arc<TypeDataStore>>, String> {
        if self.1.is_null() {
            return Err(String::from("null pointer"));
        }

        Ok(ManuallyDrop::new(Arc::from_raw(
            self.1 as *const TypeDataStore,
        )))
    }

    /// Returns the pointer ino associated with the Type
    unsafe fn inner(&self) -> Result<&PointerInfo, String> {
        match (self.0 as *const PointerInfo).as_ref() {
            Some(store) => Ok(store),
            None => Err(String::from("null pointer")),
        }
    }

    /// Converts from C FFI type to a Rust type.
    unsafe fn to_rust<'a>(self) -> Result<super::super::PointerType<'a>, String> {
        match (
            (self.0 as *const PointerInfo).as_ref(),
            (self.1 as *const Arc<TypeDataStore>).as_ref(),
        ) {
            (Some(inner), Some(store)) => Ok(super::super::PointerType { inner, store }),
            _ => Err(String::from("null pointer")),
        }
    }
}

/// Returns the type that this instance points to. Ownership is transferred if this function returns
/// successfully.
///
/// # Safety
///
/// This function results in undefined behavior if the passed in `PointerType` has been deallocated
/// by a previous call to [`mun_type_release`].
pub unsafe extern "C" fn mun_pointer_type_pointee(
    ty: PointerType,
    pointee: *mut Type,
) -> ErrorHandle {
    let store = mun_error_try!(ty
        .store()
        .map_err(|e| format!("invalid argument 'ty': {e}")));
    let ty = mun_error_try!(ty
        .inner()
        .map_err(|e| format!("invalid argument 'ty': {e}")));
    let pointee = try_deref_mut!(pointee);
    *pointee = RustType::new_unchecked(ty.pointee, ManuallyDrop::deref(&store).clone()).into();
    ErrorHandle::default()
}

/// Returns true if this is a mutable pointer.
///
/// # Safety
///
/// This function results in undefined behavior if the passed in `PointerType` has been deallocated
/// by a previous call to [`mun_type_release`].
pub unsafe extern "C" fn mun_pointer_is_mutable(
    ty: PointerType,
    mutable: *mut bool,
) -> ErrorHandle {
    let ty = mun_error_try!(ty
        .inner()
        .map_err(|e| format!("invalid argument 'ty': {e}")));
    let mutable = try_deref_mut!(mutable);
    *mutable = ty.mutable;
    ErrorHandle::default()
}

#[cfg(test)]
mod test {
    use std::mem::MaybeUninit;
    use std::ptr;

    use capi_utils::{assert_error_snapshot, assert_getter1, assert_getter2};

    use super::super::{
        mun_type_equal, mun_type_kind, mun_type_pointer_type, mun_type_release,
        pointer::{mun_pointer_is_mutable, mun_pointer_type_pointee, PointerType},
        primitive::{mun_type_primitive, PrimitiveType},
        Type, TypeKind,
    };

    /// Returns the pointer type of the specified type. Asserts if that fails.
    unsafe fn pointer_type(ty: Type, mutable: bool) -> (Type, PointerType) {
        assert_getter2!(mun_type_pointer_type(ty, mutable, ptr_ty));

        assert_getter1!(mun_type_kind(ptr_ty, ty_kind));
        let pointer_ty = match ty_kind {
            TypeKind::Pointer(p) => p,
            _ => panic!("invalid type kind for pointer"),
        };

        (ty, pointer_ty)
    }

    #[test]
    fn test_mun_pointer_type_pointee() {
        let ffi_f32 = mun_type_primitive(PrimitiveType::F32);
        let (ffi_f32_ptr, ptr_info) = unsafe { pointer_type(ffi_f32, true) };

        assert_getter1!(mun_pointer_type_pointee(ptr_info, pointee_ty));
        assert!(unsafe { mun_type_equal(pointee_ty, ffi_f32) });

        unsafe { mun_type_release(pointee_ty) };
        unsafe { mun_type_release(ffi_f32_ptr) };
        unsafe { mun_type_release(ffi_f32) };
    }

    #[test]
    fn test_mun_pointer_type_pointee_invalid_null() {
        let mut pointee_ty = MaybeUninit::uninit();
        assert_error_snapshot!(
            unsafe {
                mun_pointer_type_pointee(
                    PointerType(ptr::null(), ptr::null()),
                    pointee_ty.as_mut_ptr(),
                )
            },
            @r###""invalid argument \'ty\': null pointer""###
        );

        let ffi_f32 = mun_type_primitive(PrimitiveType::F32);
        let (ffi_f32_ptr, ptr_info) = unsafe { pointer_type(ffi_f32, true) };
        assert_error_snapshot!(
            unsafe { mun_pointer_type_pointee(ptr_info, ptr::null_mut()) },
            @r###""invalid argument \'pointee\': null pointer""###
        );

        unsafe { mun_type_release(ffi_f32_ptr) };
        unsafe { mun_type_release(ffi_f32) };
    }

    #[test]
    fn test_mun_pointer_type_is_mutable() {
        let ffi_f32 = mun_type_primitive(PrimitiveType::F32);
        let (ffi_f32_immutable_ptr, immutable_ptr_info) = unsafe { pointer_type(ffi_f32, false) };
        let (ffi_f32_mutable_ptr, mutable_ptr_info) = unsafe { pointer_type(ffi_f32, true) };

        assert_getter1!(mun_pointer_is_mutable(immutable_ptr_info, is_mutable));
        assert!(!is_mutable);

        assert_getter1!(mun_pointer_is_mutable(mutable_ptr_info, is_mutable));
        assert!(is_mutable);

        unsafe { mun_type_release(ffi_f32_mutable_ptr) };
        unsafe { mun_type_release(ffi_f32_immutable_ptr) };
        unsafe { mun_type_release(ffi_f32) };
    }

    #[test]
    fn test_mun_pointer_type_is_mutable_invalid_null() {
        let mut is_mutable = MaybeUninit::uninit();
        assert_error_snapshot!(
            unsafe {
                mun_pointer_is_mutable(
                    PointerType(ptr::null(), ptr::null()),
                    is_mutable.as_mut_ptr(),
                )
            },
            @r###""invalid argument \'ty\': null pointer""###
        );

        let ffi_f32 = mun_type_primitive(PrimitiveType::F32);
        let (ffi_f32_ptr, ptr_info) = unsafe { pointer_type(ffi_f32, true) };
        assert_error_snapshot!(
            unsafe { mun_pointer_is_mutable(ptr_info, ptr::null_mut()) },
            @r###""invalid argument \'mutable\': null pointer""###
        );

        unsafe { mun_type_release(ffi_f32_ptr) };
        unsafe { mun_type_release(ffi_f32) };
    }
}
