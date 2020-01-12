use crate::{
    body::ScopeId, Body, ExprId, ExprScopes, FileId, HirDatabase, ModuleDef, Name, PatId, Path,
    PerNs,
};
use std::sync::Arc;

#[derive(Debug, Clone, Default)]
pub struct Resolver {
    scopes: Vec<Scope>,
}

#[derive(Debug, Clone)]
pub(crate) enum Scope {
    /// All the items and imported names of a module
    ModuleScope(ModuleItemMap),

    /// Local bindings
    ExprScope(ExprScope),
}

#[derive(Debug, Clone)]
pub(crate) struct ModuleItemMap {
    file_id: FileId,
}

#[derive(Debug, Clone)]
pub(crate) struct ExprScope {
    expr_scopes: Arc<ExprScopes>,
    scope_id: ScopeId,
}

impl Resolver {
    pub(crate) fn push_scope(mut self, scope: Scope) -> Resolver {
        self.scopes.push(scope);
        self
    }

    pub(crate) fn push_module_scope(self, file_id: FileId) -> Resolver {
        self.push_scope(Scope::ModuleScope(ModuleItemMap { file_id }))
    }

    pub(crate) fn push_expr_scope(
        self,
        expr_scopes: Arc<ExprScopes>,
        scope_id: ScopeId,
    ) -> Resolver {
        self.push_scope(Scope::ExprScope(ExprScope {
            expr_scopes,
            scope_id,
        }))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Resolution {
    /// An item
    Def(ModuleDef),
    /// A local binding (only value namespace)
    LocalBinding(PatId),
}

impl Resolver {
    pub fn resolve_name(&self, db: &impl HirDatabase, name: &Name) -> PerNs<Resolution> {
        let mut resolution = PerNs::none();
        for scope in self.scopes.iter().rev() {
            resolution = resolution.or(scope.resolve_name(db, name));
            if resolution.is_both() {
                return resolution;
            }
        }
        resolution
    }

    /// Returns the fully resolved path if we were able to resolve it.
    /// otherwise returns `PerNs::none`
    pub fn resolve_path_without_assoc_items(
        &self,
        db: &impl HirDatabase,
        path: &Path,
    ) -> PerNs<Resolution> {
        if let Some(name) = path.as_ident() {
            self.resolve_name(db, name)
        } else {
            PerNs::none()
        }
    }
}

impl Scope {
    fn resolve_name(&self, db: &impl HirDatabase, name: &Name) -> PerNs<Resolution> {
        match self {
            Scope::ModuleScope(m) => db
                .module_scope(m.file_id)
                .get(name)
                .map(|r| r.def)
                .unwrap_or_else(PerNs::none)
                .map(Resolution::Def),
            Scope::ExprScope(e) => {
                let entry = e
                    .expr_scopes
                    .entries(e.scope_id)
                    .iter()
                    .find(|entry| entry.name() == name);
                match entry {
                    Some(e) => PerNs::values(Resolution::LocalBinding(e.pat())),
                    None => PerNs::none(),
                }
            }
        }
    }

    //    fn collect_names(&self, db: &impl HirDatabase, f: &mut dyn FnMut(Name, PerNs<Resolution>)) {
    //        match self {
    //            Scope::ModuleScope(m) => {
    //                // FIXME: should we provide `self` here?
    //                // f(
    //                //     Name::self_param(),
    //                //     PerNs::types(Resolution::Def {
    //                //         def: m.module.into(),
    //                //     }),
    //                // );
    //                m.crate_def_map[m.module_id].scope.entries().for_each(|(name, res)| {
    //                    f(name.clone(), res.def.map(Resolution::Def));
    //                });
    //                m.crate_def_map.extern_prelude().iter().for_each(|(name, def)| {
    //                    f(name.clone(), PerNs::types(Resolution::Def(*def)));
    //                });
    //                if let Some(prelude) = m.crate_def_map.prelude() {
    //                    let prelude_def_map = db.crate_def_map(prelude.krate);
    //                    prelude_def_map[prelude.module_id].scope.entries().for_each(|(name, res)| {
    //                        f(name.clone(), res.def.map(Resolution::Def));
    //                    });
    //                }
    //            }
    //            Scope::GenericParams(gp) => {
    //                for param in &gp.params {
    //                    f(param.name.clone(), PerNs::types(Resolution::GenericParam(param.idx)))
    //                }
    //            }
    //            Scope::ImplBlockScope(i) => {
    //                f(SELF_TYPE, PerNs::types(Resolution::SelfType(*i)));
    //            }
    //            Scope::ExprScope(e) => {
    //                e.expr_scopes.entries(e.scope_id).iter().for_each(|e| {
    //                    f(e.name().clone(), PerNs::values(Resolution::LocalBinding(e.pat())));
    //                });
    //            }
    //        }
    //    }
}

// needs arbitrary_self_types to be a method... or maybe move to the def?
pub fn resolver_for_expr(body: Arc<Body>, db: &impl HirDatabase, expr_id: ExprId) -> Resolver {
    let scopes = db.expr_scopes(body.owner());
    resolver_for_scope(body, db, scopes.scope_for(expr_id))
}

pub(crate) fn resolver_for_scope(
    body: Arc<Body>,
    db: &impl HirDatabase,
    scope_id: Option<ScopeId>,
) -> Resolver {
    let mut r = body.owner().resolver(db);
    let scopes = db.expr_scopes(body.owner());
    let scope_chain = scopes.scope_chain(scope_id).collect::<Vec<_>>();
    for scope in scope_chain.into_iter().rev() {
        r = r.push_expr_scope(Arc::clone(&scopes), scope);
    }
    r
}
