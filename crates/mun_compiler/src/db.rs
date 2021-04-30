use hir::{salsa, HirDatabase, Upcast};
use mun_codegen::{CodeGenDatabase, CodeGenDatabaseStorage, OptimizationLevel};
use mun_target::spec::Target;

/// A compiler database is a salsa database that enables increment compilation.
#[salsa::database(
    hir::SourceDatabaseStorage,
    hir::InternDatabaseStorage,
    hir::AstDatabaseStorage,
    hir::DefDatabaseStorage,
    hir::HirDatabaseStorage,
    CodeGenDatabaseStorage
)]
pub struct CompilerDatabase {
    storage: salsa::Storage<Self>,
}

impl Upcast<dyn hir::AstDatabase> for CompilerDatabase {
    fn upcast(&self) -> &dyn hir::AstDatabase {
        &*self
    }
}

impl Upcast<dyn hir::SourceDatabase> for CompilerDatabase {
    fn upcast(&self) -> &dyn hir::SourceDatabase {
        &*self
    }
}

impl Upcast<dyn hir::DefDatabase> for CompilerDatabase {
    fn upcast(&self) -> &dyn hir::DefDatabase {
        &*self
    }
}

impl Upcast<dyn hir::HirDatabase> for CompilerDatabase {
    fn upcast(&self) -> &dyn hir::HirDatabase {
        &*self
    }
}

impl Upcast<dyn CodeGenDatabase> for CompilerDatabase {
    fn upcast(&self) -> &dyn CodeGenDatabase {
        &*self
    }
}

impl CompilerDatabase {
    /// Constructs a new database
    pub fn new(target: Target, optimization_level: OptimizationLevel) -> Self {
        let mut db = CompilerDatabase {
            storage: Default::default(),
        };

        db.set_target(target);
        db.set_optimization_level(optimization_level);

        db
    }
}

impl salsa::Database for CompilerDatabase {}
