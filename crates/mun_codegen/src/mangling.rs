use mun_hir::HirDatabase;

fn mangle_symbol(db: &dyn HirDatabase, ty: mun_hir::Ty) -> String {
    ty.def
}
