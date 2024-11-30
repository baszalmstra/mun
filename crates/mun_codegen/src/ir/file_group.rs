use std::collections::BTreeMap;

use inkwell::{module::Module, types::PointerType, values::UnnamedAddress, AddressSpace};
use mun_hir::HasVisibility;
use rustc_hash::FxHashSet;

use super::{
    dispatch_table::{DispatchTable, DispatchTableBuilder},
    intrinsics,
    type_table::{TypeTable, TypeTableBuilder},
};
use crate::{
    code_gen::CodeGenContext,
    module_group::ModuleGroup,
    value::{IrTypeContext, IrValueContext},
};

/// The IR generated for a group of files. It is used to generate IR for all of
/// the group's files and the resulting `Assembly`'s symbols.
#[derive(Debug, PartialEq, Eq)]
pub struct FileGroupIr<'ink> {
    /// The LLVM module that contains the IR
    pub(crate) llvm_module: Module<'ink>,
    /// The dispatch table
    pub(crate) dispatch_table: DispatchTable<'ink>,
    /// The type table
    pub(crate) type_table: TypeTable<'ink>,
    /// The allocator handle, if it exists
    pub(crate) allocator_handle_type: Option<PointerType<'ink>>,
    /// The modules that contain code that was referenced from this group of
    /// modules
    pub(crate) referenced_modules: FxHashSet<mun_hir::Module>,
}

/// Generates IR that is shared among the group's files.
pub(crate) fn gen_file_group_ir<'ink>(
    code_gen: &CodeGenContext<'_, 'ink>,
    module_group: &ModuleGroup,
) -> FileGroupIr<'ink> {
    let llvm_module = code_gen
        .context
        .create_module(&format!("filegroupir:{}", &module_group.name));

    // Use a `BTreeMap` to guarantee deterministically ordered output.
    let mut intrinsics_map = BTreeMap::new();
    let mut needs_alloc = false;

    // Collect all intrinsic functions, wrapper function, and generate struct
    // declarations.
    for fun in module_group.all_functions(code_gen.db) {
        // We don't need to generate code for extern functions
        if fun.is_extern(code_gen.db) {
            continue;
        }

        // Collect intrinsics used within the function
        // TODO: Move this into a database query??
        intrinsics::collect_fn_body(
            code_gen.context,
            code_gen.target_machine.get_target_data(),
            code_gen.db,
            &mut intrinsics_map,
            &mut needs_alloc,
            &fun.body(code_gen.db),
            &fun.infer(code_gen.db),
        );

        // Collect intrinsics for wrapper functions for functions that are not
        // marshallable
        let fn_sig = fun.ty(code_gen.db).callable_sig(code_gen.db).unwrap();
        if fun.visibility(code_gen.db).is_externally_visible() && !fn_sig.marshallable(code_gen.db)
        {
            intrinsics::collect_wrapper_body(
                code_gen.context,
                code_gen.target_machine.get_target_data(),
                &mut intrinsics_map,
                &mut needs_alloc,
            );
        }
    }

    // Collect all exposed functions' bodies.
    let mut dispatch_table_builder = DispatchTableBuilder::new(
        code_gen.context,
        code_gen.target_machine.get_target_data(),
        code_gen.db,
        &llvm_module,
        &intrinsics_map,
        &code_gen.hir_types,
        module_group,
    );
    for fun in module_group.all_functions(code_gen.db) {
        // Find all functions that must be present in the dispatch table
        if !fun.is_extern(code_gen.db) {
            let body = fun.body(code_gen.db);
            let infer = fun.infer(code_gen.db);
            dispatch_table_builder.collect_body(&body, &infer);
        }
    }

    let (dispatch_table, referenced_modules) = dispatch_table_builder.build();

    let target_data = code_gen.target_machine.get_target_data();
    let type_context = IrTypeContext {
        context: code_gen.context,
        target_data: &target_data,
        struct_types: &code_gen.rust_types,
    };
    let value_context = IrValueContext {
        type_context: &type_context,
        context: code_gen.context,
        module: &llvm_module,
    };
    let mut type_table_builder = TypeTableBuilder::new(
        code_gen.db,
        &value_context,
        intrinsics_map.keys(),
        &dispatch_table,
        &code_gen.hir_types,
        module_group,
    );

    for fun in module_group.all_functions(code_gen.db) {
        type_table_builder.collect_fn(fun);
    }
    for struct_def in module_group.struct_defs(code_gen.db) {
        type_table_builder.collect_struct(struct_def);
    }

    let type_table = type_table_builder.build();

    // Create the allocator handle global value
    let allocator_handle_type = if needs_alloc {
        let allocator_handle_type = code_gen.context.i8_type().ptr_type(AddressSpace::default());
        let global = llvm_module.add_global(allocator_handle_type, None, "allocatorHandle");
        global.set_initializer(&allocator_handle_type.const_null());
        global.set_unnamed_address(UnnamedAddress::Global);
        Some(allocator_handle_type)
    } else {
        None
    };

    FileGroupIr {
        llvm_module,
        dispatch_table,
        type_table,
        allocator_handle_type,
        referenced_modules,
    }
}
