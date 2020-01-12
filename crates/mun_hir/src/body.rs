mod raw_expr;
mod scope;
mod validator;

pub use raw_expr::{
    ArithOp, BinaryOp, CmpOp, Literal, LogicOp, Ordering, Pat, RawExpr, RecordLitField, Statement,
};
pub use scope::{ExprScopes, ScopeId};
pub use validator::ExprValidator;

use self::raw_expr::RawExprCollector;
use crate::{
    arena::map::ArenaMap,
    arena::Arena,
    code_model::src::HasSource,
    code_model::DefWithBody,
    in_file::InFile,
    type_ref::{TypeRef, TypeRefId, TypeRefMap, TypeRefSourceMap},
    HirDatabase, RawId,
};
use either::Either;
use mun_syntax::{ast, AstPtr};
use rustc_hash::FxHashMap;
use std::ops::Index;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExprId(RawId);
impl_arena_id!(ExprId);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PatId(RawId);
impl_arena_id!(PatId);

/// The body of an item (function, const etc.).
#[derive(Debug, Eq, PartialEq)]
pub struct Body {
    owner: DefWithBody,
    exprs: Arena<ExprId, RawExpr>,
    pats: Arena<PatId, Pat>,
    type_refs: TypeRefMap,
    /// The patterns for the function's parameters. While the parameter types are part of the
    /// function signature, the patterns are not (they don't change the external type of the
    /// function).
    ///
    /// If this `Body` is for the body of a constant, this will just be empty.
    params: Vec<(PatId, TypeRefId)>,
    /// The `ExprId` of the actual body expression.
    body_expr: ExprId,
    ret_type: TypeRefId,
}

impl Body {
    pub fn params(&self) -> &[(PatId, TypeRefId)] {
        &self.params
    }

    pub fn body_expr(&self) -> ExprId {
        self.body_expr
    }

    pub fn owner(&self) -> DefWithBody {
        self.owner
    }

    pub fn exprs(&self) -> impl Iterator<Item = (ExprId, &RawExpr)> {
        self.exprs.iter()
    }

    pub fn pats(&self) -> impl Iterator<Item = (PatId, &Pat)> {
        self.pats.iter()
    }

    pub fn type_refs(&self) -> &TypeRefMap {
        &self.type_refs
    }

    pub fn ret_type(&self) -> TypeRefId {
        self.ret_type
    }
}

impl Index<ExprId> for Body {
    type Output = RawExpr;

    fn index(&self, expr: ExprId) -> &RawExpr {
        &self.exprs[expr]
    }
}

impl Index<PatId> for Body {
    type Output = Pat;

    fn index(&self, pat: PatId) -> &Pat {
        &self.pats[pat]
    }
}

impl Index<TypeRefId> for Body {
    type Output = TypeRef;

    fn index(&self, type_ref: TypeRefId) -> &TypeRef {
        &self.type_refs[type_ref]
    }
}

type ExprPtr = Either<AstPtr<ast::Expr>, AstPtr<ast::RecordField>>;
type ExprSource = InFile<ExprPtr>;

type PatPtr = AstPtr<ast::Pat>; //Either<AstPtr<ast::Pat>, AstPtr<ast::SelfParam>>;
type PatSource = InFile<PatPtr>;

type RecordPtr = AstPtr<ast::RecordField>;

/// An item body together with the mapping from syntax nodes to HIR expression Ids. This is needed
/// to go from e.g. a position in a file to the HIR expression containing it; but for type
/// inference etc., we want to operate on a structure that is agnostic to the action positions of
/// expressions in the file, so that we don't recompute types whenever some whitespace is typed.
#[derive(Default, Debug, Eq, PartialEq)]
pub struct BodySourceMap {
    expr_map: FxHashMap<ExprPtr, ExprId>,
    expr_map_back: ArenaMap<ExprId, ExprSource>,
    pat_map: FxHashMap<PatPtr, PatId>,
    pat_map_back: ArenaMap<PatId, PatSource>,
    type_refs: TypeRefSourceMap,
    field_map: FxHashMap<(ExprId, usize), RecordPtr>,
}

impl BodySourceMap {
    pub(crate) fn expr_syntax(&self, expr: ExprId) -> Option<ExprSource> {
        self.expr_map_back.get(expr).cloned()
    }

    pub fn type_ref_syntax(&self, type_ref: TypeRefId) -> Option<AstPtr<ast::TypeRef>> {
        self.type_refs.type_ref_syntax(type_ref)
    }

    pub(crate) fn syntax_expr(&self, ptr: ExprPtr) -> Option<ExprId> {
        self.expr_map.get(&ptr).cloned()
    }

    pub(crate) fn node_expr(&self, node: &ast::Expr) -> Option<ExprId> {
        self.expr_map.get(&Either::Left(AstPtr::new(node))).cloned()
    }

    pub(crate) fn pat_syntax(&self, pat: PatId) -> Option<PatSource> {
        self.pat_map_back.get(pat).cloned()
    }

    pub(crate) fn node_pat(&self, node: &ast::Pat) -> Option<PatId> {
        self.pat_map.get(&AstPtr::new(node)).cloned()
    }

    pub fn type_refs(&self) -> &TypeRefSourceMap {
        &self.type_refs
    }

    pub fn field_syntax(&self, expr: ExprId, field: usize) -> RecordPtr {
        self.field_map[&(expr, field)]
    }
}

pub(crate) fn body_with_source_map_query(
    db: &impl HirDatabase,
    def: DefWithBody,
) -> (Arc<Body>, Arc<BodySourceMap>) {
    let mut collector;

    match def {
        DefWithBody::Function(ref f) => {
            let src = f.source(db);
            collector = RawExprCollector::new(def, src.file_id, db);
            collector.collect_fn_body(&src.value)
        }
    }

    let (body, source_map) = collector.finish();
    (Arc::new(body), Arc::new(source_map))
}

pub(crate) fn body_hir_query(db: &impl HirDatabase, def: DefWithBody) -> Arc<Body> {
    db.body_with_source_map(def).0
}
