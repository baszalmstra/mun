use super::try_convert_any_to_basic;
use crate::type_info::TypeSize;
use crate::{
    type_info::{TypeGroup, TypeInfo},
    CodeGenParams, IrDatabase,
};
use hir::{
    ApplicationTy, CallableDef, FloatBitness, FloatTy, HirDisplay, IntBitness, IntTy, Ty, TypeCtor,
};
use inkwell::types::{AnyTypeEnum, BasicType, BasicTypeEnum, FloatType, IntType, StructType};
use inkwell::AddressSpace;
use mun_target::spec::Target;

/// Given a mun type, construct an LLVM IR type
#[rustfmt::skip]
pub(crate) fn ir_query(db: &impl IrDatabase, ty: Ty, params: CodeGenParams) -> AnyTypeEnum {
    let context = db.context();
    match ty {
        Ty::Empty => AnyTypeEnum::StructType(context.struct_type(&[], false)),
        Ty::Apply(ApplicationTy { ctor, .. }) => match ctor {
            TypeCtor::Float(fty) => float_ty_query(db, fty).into(),
            TypeCtor::Int(ity) => int_ty_query(db, ity).into(),
            TypeCtor::Bool => AnyTypeEnum::IntType(context.bool_type()),

            TypeCtor::FnDef(def @ CallableDef::Function(_)) => {
                let ty = db.callable_sig(def);
                let param_tys: Vec<BasicTypeEnum> = ty
                    .params()
                    .iter()
                    .map(|p| {
                        try_convert_any_to_basic(db.type_ir(p.clone(), params.clone())).unwrap()
                    })
                    .collect();

                let fn_type = match ty.ret() {
                    Ty::Empty => context.void_type().fn_type(&param_tys, false),
                    ty => try_convert_any_to_basic(db.type_ir(ty.clone(), params))
                        .expect("could not convert return value")
                        .fn_type(&param_tys, false),
                };

                AnyTypeEnum::FunctionType(fn_type)
            }
            TypeCtor::Struct(s) => {
                let struct_ty = db.struct_ty(s);
                match s.data(db).memory_kind {
                    hir::StructMemoryKind::GC => struct_ty.ptr_type(AddressSpace::Generic).ptr_type(AddressSpace::Const).into(),
                    hir::StructMemoryKind::Value if params.make_marshallable =>
                            struct_ty.ptr_type(AddressSpace::Generic).ptr_type(AddressSpace::Const).into(),
                    hir::StructMemoryKind::Value => struct_ty.into(),
                }
            }
            TypeCtor::Array => db.array_ty(ty).into(),
            _ => unreachable!(),
        },
        _ => unreachable!("unknown type can not be converted"),
    }
}

/// Returns the LLVM IR type of the specified float type
fn float_ty_query(db: &impl IrDatabase, fty: FloatTy) -> FloatType {
    let context = db.context();
    match fty.resolve(&db.target()).bitness {
        FloatBitness::X64 => context.f64_type(),
        FloatBitness::X32 => context.f32_type(),
        _ => unreachable!(),
    }
}

/// Returns the LLVM IR type of the specified int type
fn int_ty_query(db: &impl IrDatabase, ity: IntTy) -> IntType {
    let context = db.context();
    match ity.resolve(&db.target()).bitness {
        IntBitness::X128 => context.i128_type(),
        IntBitness::X64 => context.i64_type(),
        IntBitness::X32 => context.i32_type(),
        IntBitness::X16 => context.i16_type(),
        IntBitness::X8 => context.i8_type(),
        _ => unreachable!(),
    }
}

/// Builds an LLVM IR type for a array (e.g. [T]). In LLVM we represent this as a struct:
///
/// ```text
/// %array = type { int, [0 x T] }
/// ```
///
/// The size of the struct is the first field of the type. Followed by the elements.
pub fn array_ty_query(db: &impl IrDatabase, s: hir::Ty) -> StructType {
    let inner_ty = match s {
        hir::Ty::Apply(hir::ApplicationTy {
            ctor: TypeCtor::Array,
            parameters: st,
        }) => st.as_single().clone(),
        _ => panic!(
            "cannot get array LLVM type for non-array type: {}",
            s.display(db)
        ),
    };

    let usize_type = db.target_data().ptr_sized_int_type(None);
    let inner_ir_type = try_convert_any_to_basic(db.type_ir(inner_ty, CodeGenParams::default()))
        .expect("cannot create array of a non-basic type");
    db.context().struct_type(
        &[usize_type.into(), inner_ir_type.array_type(0).into()],
        false,
    )
}

/// Returns the LLVM IR type of the specified struct
pub fn struct_ty_query(db: &(impl IrDatabase + salsa::Database), s: hir::Struct) -> StructType {
    let name = s.name(db).to_string();
    for field in s.fields(db).iter() {
        // Ensure that salsa's cached value incorporates the struct fields
        let _field_type_ir = db.type_ir(
            field.ty(db),
            CodeGenParams {
                make_marshallable: false,
            },
        );
    }
    //let query = db.salsa_runtime().active_query().unwrap();

    db.context().opaque_struct_type(&name)
}

/// Constructs the `TypeInfo` for the specified HIR type
pub fn type_info_query(db: &impl IrDatabase, ty: Ty) -> TypeInfo {
    let target = db.target_data();
    match ty {
        Ty::Apply(ctor) => match ctor.ctor {
            TypeCtor::Float(ty) => {
                let ir_ty = float_ty_query(db, ty);
                let type_size = TypeSize::from_ir_type(&ir_ty, target.as_ref());
                TypeInfo::new(
                    format!("core::{}", ty.resolve(&db.target())),
                    TypeGroup::FundamentalTypes,
                    type_size,
                )
            }
            TypeCtor::Int(ty) => {
                let ir_ty = int_ty_query(db, ty);
                let type_size = TypeSize::from_ir_type(&ir_ty, target.as_ref());
                TypeInfo::new(
                    format!("core::{}", ty.resolve(&db.target())),
                    TypeGroup::FundamentalTypes,
                    type_size,
                )
            }
            TypeCtor::Bool => {
                let ir_ty = db.context().bool_type();
                let type_size = TypeSize::from_ir_type(&ir_ty, target.as_ref());
                TypeInfo::new("core::bool", TypeGroup::FundamentalTypes, type_size)
            }
            TypeCtor::Struct(s) => {
                let ir_ty = db.struct_ty(s);
                let type_size = TypeSize::from_ir_type(&ir_ty, target.as_ref());
                TypeInfo::new(s.name(db).to_string(), TypeGroup::StructTypes(s), type_size)
            }
            _ => unreachable!("{:?} unhandled", ctor),
        },
        _ => unreachable!("{:?} unhandled", ty),
    }
}

trait ResolveBitness {
    fn resolve(&self, _target: &Target) -> Self;
}

impl ResolveBitness for FloatTy {
    fn resolve(&self, _target: &Target) -> Self {
        let bitness = match self.bitness {
            FloatBitness::Undefined => FloatBitness::X64,
            bitness => bitness,
        };
        FloatTy { bitness }
    }
}

impl ResolveBitness for IntTy {
    fn resolve(&self, _target: &Target) -> Self {
        let bitness = match self.bitness {
            IntBitness::Undefined => IntBitness::X64,
            IntBitness::Xsize => IntBitness::X64,
            bitness => bitness,
        };
        IntTy {
            bitness,
            signedness: self.signedness,
        }
    }
}
