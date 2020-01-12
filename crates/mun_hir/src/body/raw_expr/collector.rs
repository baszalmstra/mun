use super::{
    super::{Body, BodySourceMap, ExprId, Pat, PatId, PatPtr, RawExpr, RecordPtr},
    ArithOp, BinaryOp, Literal, RecordLitField, Statement,
};
use crate::{
    arena::Arena,
    body::raw_expr::{CmpOp, Ordering},
    code_model::DefWithBody,
    in_file::InFile,
    name::AsName,
    type_ref::{TypeRefBuilder, TypeRefId},
    FileId, HirDatabase, Name, Path,
};
use either::Either;
use mun_syntax::{
    ast,
    ast::{ArgListOwner, BinOp, LoopBodyOwner, NameOwner, TypeAscriptionOwner},
    AstNode, AstPtr, T,
};
use std::mem;

pub(crate) struct RawExprCollector<DB> {
    db: DB,
    owner: DefWithBody,
    exprs: Arena<ExprId, RawExpr>,
    pats: Arena<PatId, Pat>,
    source_map: BodySourceMap,
    params: Vec<(PatId, TypeRefId)>,
    body_expr: Option<ExprId>,
    ret_type: Option<TypeRefId>,
    type_ref_builder: TypeRefBuilder,
    current_file_id: FileId,
}

impl<'a, DB> RawExprCollector<&'a DB>
where
    DB: HirDatabase,
{
    pub fn new(owner: DefWithBody, file_id: FileId, db: &'a DB) -> Self {
        RawExprCollector {
            owner,
            db,
            exprs: Arena::default(),
            pats: Arena::default(),
            source_map: BodySourceMap::default(),
            params: Vec::new(),
            body_expr: None,
            ret_type: None,
            type_ref_builder: TypeRefBuilder::default(),
            current_file_id: file_id,
        }
    }

    fn alloc_pat(&mut self, pat: Pat, ptr: PatPtr) -> PatId {
        let id = self.pats.alloc(pat);
        self.source_map.pat_map.insert(ptr, id);
        self.source_map
            .pat_map_back
            .insert(id, InFile::new(self.current_file_id, ptr));
        id
    }

    fn alloc_expr(&mut self, expr: RawExpr, ptr: AstPtr<ast::Expr>) -> ExprId {
        let ptr = Either::Left(ptr);
        let id = self.exprs.alloc(expr);
        self.source_map.expr_map.insert(ptr, id);
        self.source_map
            .expr_map_back
            .insert(id, InFile::new(self.current_file_id, ptr));
        id
    }

    fn alloc_expr_field_shorthand(&mut self, expr: RawExpr, ptr: RecordPtr) -> ExprId {
        let ptr = Either::Right(ptr);
        let id = self.exprs.alloc(expr);
        self.source_map.expr_map.insert(ptr, id);
        self.source_map
            .expr_map_back
            .insert(id, InFile::new(self.current_file_id, ptr));
        id
    }

    fn missing_expr(&mut self) -> ExprId {
        self.exprs.alloc(RawExpr::Missing)
    }

    pub fn collect_fn_body(&mut self, node: &ast::FunctionDef) {
        if let Some(param_list) = node.param_list() {
            for param in param_list.params() {
                let pat = if let Some(pat) = param.pat() {
                    pat
                } else {
                    continue;
                };
                let param_pat = self.collect_pat(pat);
                let param_type = self
                    .type_ref_builder
                    .alloc_from_node_opt(param.ascribed_type().as_ref());
                self.params.push((param_pat, param_type));
            }
        }

        let body = self.collect_block_opt(node.body());
        self.body_expr = Some(body);

        let ret_type = if let Some(type_ref) = node.ret_type().and_then(|rt| rt.type_ref()) {
            self.type_ref_builder.alloc_from_node(&type_ref)
        } else {
            self.type_ref_builder.unit()
        };
        self.ret_type = Some(ret_type);
    }

    fn collect_block_opt(&mut self, block: Option<ast::BlockExpr>) -> ExprId {
        if let Some(block) = block {
            self.collect_block(block)
        } else {
            self.exprs.alloc(RawExpr::Missing)
        }
    }

    fn collect_block(&mut self, block: ast::BlockExpr) -> ExprId {
        let syntax_node_ptr = AstPtr::new(&block.clone().into());
        let statements = block
            .statements()
            .map(|s| match s.kind() {
                ast::StmtKind::LetStmt(stmt) => {
                    let pat = self.collect_pat_opt(stmt.pat());
                    let type_ref = stmt
                        .ascribed_type()
                        .map(|t| self.type_ref_builder.alloc_from_node(&t));
                    let initializer = stmt.initializer().map(|e| self.collect_expr(e));
                    Statement::Let {
                        pat,
                        type_ref,
                        initializer,
                    }
                }
                ast::StmtKind::ExprStmt(stmt) => {
                    Statement::Expr(self.collect_expr_opt(stmt.expr()))
                }
            })
            .collect();
        let tail = block.expr().map(|e| self.collect_expr(e));
        self.alloc_expr(RawExpr::Block { statements, tail }, syntax_node_ptr)
    }

    fn collect_pat_opt(&mut self, pat: Option<ast::Pat>) -> PatId {
        if let Some(pat) = pat {
            self.collect_pat(pat)
        } else {
            self.pats.alloc(Pat::Missing)
        }
    }

    fn collect_expr_opt(&mut self, expr: Option<ast::Expr>) -> ExprId {
        if let Some(expr) = expr {
            self.collect_expr(expr)
        } else {
            self.exprs.alloc(RawExpr::Missing)
        }
    }

    fn collect_expr(&mut self, expr: ast::Expr) -> ExprId {
        let syntax_ptr = AstPtr::new(&expr.clone());
        match expr.kind() {
            ast::ExprKind::LoopExpr(expr) => self.collect_loop(expr),
            ast::ExprKind::WhileExpr(expr) => self.collect_while(expr),
            ast::ExprKind::ReturnExpr(r) => self.collect_return(r),
            ast::ExprKind::BreakExpr(r) => self.collect_break(r),
            ast::ExprKind::BlockExpr(b) => self.collect_block(b),
            ast::ExprKind::Literal(e) => {
                let lit = match e.kind() {
                    ast::LiteralKind::Bool => Literal::Bool(e.token().kind() == T![true]),
                    ast::LiteralKind::IntNumber => {
                        Literal::Int(e.syntax().text().to_string().parse().unwrap())
                    }
                    ast::LiteralKind::FloatNumber => {
                        Literal::Float(e.syntax().text().to_string().parse().unwrap())
                    }
                    ast::LiteralKind::String => Literal::String(Default::default()),
                };
                self.alloc_expr(RawExpr::Literal(lit), syntax_ptr)
            }
            ast::ExprKind::PrefixExpr(e) => {
                let expr = self.collect_expr_opt(e.expr());
                if let Some(op) = e.op_kind() {
                    self.alloc_expr(RawExpr::UnaryOp { expr, op }, syntax_ptr)
                } else {
                    self.alloc_expr(RawExpr::Missing, syntax_ptr)
                }
            }
            ast::ExprKind::BinExpr(e) => {
                let op = e.op_kind();
                if let Some(op) = op {
                    match op {
                        op @ BinOp::Add
                        | op @ BinOp::Subtract
                        | op @ BinOp::Divide
                        | op @ BinOp::Multiply
                        | op @ BinOp::Equals
                        | op @ BinOp::NotEquals
                        | op @ BinOp::Less
                        | op @ BinOp::LessEqual
                        | op @ BinOp::Greater
                        | op @ BinOp::GreatEqual
                        //| op @ BinOp::Remainder
                        //| op @ BinOp::Power
                        => {
                            let op = match op {
                                BinOp::Add => BinaryOp::ArithOp(ArithOp::Add),
                                BinOp::Subtract => BinaryOp::ArithOp(ArithOp::Subtract),
                                BinOp::Divide => BinaryOp::ArithOp(ArithOp::Divide),
                                BinOp::Multiply => BinaryOp::ArithOp(ArithOp::Multiply),
                                BinOp::Equals => BinaryOp::CmpOp(CmpOp::Eq { negated: false }),
                                BinOp::NotEquals => BinaryOp::CmpOp(CmpOp::Eq { negated: true }),
                                BinOp::Less => BinaryOp::CmpOp(CmpOp::Ord { ordering: Ordering::Less, strict: true } ),
                                BinOp::LessEqual => BinaryOp::CmpOp(CmpOp::Ord { ordering: Ordering::Less, strict: false } ),
                                BinOp::Greater => BinaryOp::CmpOp(CmpOp::Ord { ordering: Ordering::Greater, strict: true } ),
                                BinOp::GreatEqual => BinaryOp::CmpOp(CmpOp::Ord { ordering: Ordering::Greater, strict: false } ),
                                //BinOp::Remainder => BinaryOp::ArithOp(ArithOp::Remainder),
                                //BinOp::Power => BinaryOp::ArithOp(ArithOp::Power),
                                _ => unreachable!(),
                            };
                            let lhs = self.collect_expr_opt(e.lhs());
                            let rhs = self.collect_expr_opt(e.rhs());
                            self.alloc_expr(
                                RawExpr::BinaryOp {
                                    lhs,
                                    rhs,
                                    op: Some(op),
                                },
                                syntax_ptr,
                            )
                        }
                        op @ BinOp::Assign
                        | op @ BinOp::AddAssign
                        | op @ BinOp::SubtractAssign
                        | op @ BinOp::MultiplyAssign
                        | op @ BinOp::DivideAssign => {

                            let assign_op = match op {
                                BinOp::Assign => None,
                                BinOp::AddAssign => Some(ArithOp::Add),
                                BinOp::SubtractAssign => Some(ArithOp::Subtract),
                                BinOp::MultiplyAssign => Some(ArithOp::Multiply),
                                BinOp::DivideAssign => Some(ArithOp::Divide),
                                _ => unreachable!("invalid assignment operator")
                            } ;

                            let lhs = self.collect_expr_opt(e.lhs());
                            let rhs = self.collect_expr_opt(e.rhs());
                            self.alloc_expr(
                                RawExpr::BinaryOp {
                                    lhs,
                                    rhs,
                                    op: Some(BinaryOp::Assignment { op: assign_op }),
                                },
                                syntax_ptr,
                            )
                        }
                    }
                } else {
                    let lhs = self.collect_expr_opt(e.lhs());
                    let rhs = self.collect_expr_opt(e.rhs());
                    self.alloc_expr(RawExpr::BinaryOp { lhs, rhs, op: None }, syntax_ptr)
                }
            }
            ast::ExprKind::PathExpr(e) => {
                let path = e
                    .path()
                    .and_then(Path::from_ast)
                    .map(RawExpr::Path)
                    .unwrap_or(RawExpr::Missing);
                self.alloc_expr(path, syntax_ptr)
            }
            ast::ExprKind::RecordLit(e) => {
                let path = e.path().and_then(Path::from_ast);
                let mut field_ptrs = Vec::new();
                let record_lit = if let Some(r) = e.record_field_list() {
                    let fields = r
                        .fields()
                        .inspect(|field| field_ptrs.push(AstPtr::new(field)))
                        .map(|field| RecordLitField {
                            name: field
                                .name_ref()
                                .map(|nr| nr.as_name())
                                .unwrap_or_else(Name::missing),
                            expr: if let Some(e) = field.expr() {
                                self.collect_expr(e)
                            } else if let Some(nr) = field.name_ref() {
                                self.alloc_expr_field_shorthand(
                                    RawExpr::Path(Path::from_name_ref(&nr)),
                                    AstPtr::new(&field),
                                )
                            } else {
                                self.missing_expr()
                            },
                        })
                        .collect();
                    let spread = r.spread().map(|s| self.collect_expr(s));
                    RawExpr::RecordLit {
                        path,
                        fields,
                        spread,
                    }
                } else {
                    RawExpr::RecordLit {
                        path,
                        fields: Vec::new(),
                        spread: None,
                    }
                };

                let res = self.alloc_expr(record_lit, syntax_ptr);
                for (idx, ptr) in field_ptrs.into_iter().enumerate() {
                    self.source_map.field_map.insert((res, idx), ptr);
                }
                res
            }
            ast::ExprKind::FieldExpr(e) => {
                let expr = self.collect_expr_opt(e.expr());
                let name = match e.field_access() {
                    Some(kind) => kind.as_name(),
                    None => Name::missing(),
                };
                self.alloc_expr(RawExpr::Field { expr, name }, syntax_ptr)
            }
            ast::ExprKind::IfExpr(e) => {
                let then_branch = self.collect_block_opt(e.then_branch());

                let else_branch = e.else_branch().map(|b| match b {
                    ast::ElseBranch::Block(it) => self.collect_block(it),
                    ast::ElseBranch::IfExpr(elif) => {
                        let expr = ast::Expr::cast(elif.syntax().clone()).unwrap();
                        self.collect_expr(expr)
                    }
                });

                let condition = self.collect_condition_opt(e.condition());

                self.alloc_expr(
                    RawExpr::If {
                        condition,
                        then_branch,
                        else_branch,
                    },
                    syntax_ptr,
                )
            }
            ast::ExprKind::ParenExpr(e) => {
                let inner = self.collect_expr_opt(e.expr());
                // make the paren expr point to the inner expression as well
                let src = Either::Left(syntax_ptr);
                self.source_map.expr_map.insert(src, inner);
                inner
            }
            ast::ExprKind::CallExpr(e) => {
                let callee = self.collect_expr_opt(e.expr());
                let args = if let Some(arg_list) = e.arg_list() {
                    arg_list.args().map(|e| self.collect_expr(e)).collect()
                } else {
                    Vec::new()
                };
                self.alloc_expr(RawExpr::Call { callee, args }, syntax_ptr)
            }
        }
    }

    fn collect_condition_opt(&mut self, cond: Option<ast::Condition>) -> ExprId {
        if let Some(cond) = cond {
            self.collect_condition(cond)
        } else {
            self.exprs.alloc(RawExpr::Missing)
        }
    }

    fn collect_condition(&mut self, cond: ast::Condition) -> ExprId {
        match cond.pat() {
            None => self.collect_expr_opt(cond.expr()),
            _ => unreachable!("patterns in conditions are not yet supported"),
        }
    }

    fn collect_pat(&mut self, pat: ast::Pat) -> PatId {
        let pattern = match pat.kind() {
            ast::PatKind::BindPat(bp) => {
                let name = bp
                    .name()
                    .map(|nr| nr.as_name())
                    .unwrap_or_else(Name::missing);
                Pat::Bind { name }
            }
            ast::PatKind::PlaceholderPat(_) => Pat::Wild,
        };
        let ptr = AstPtr::new(&pat);
        self.alloc_pat(pattern, ptr)
    }

    fn collect_return(&mut self, expr: ast::ReturnExpr) -> ExprId {
        let syntax_node_ptr = AstPtr::new(&expr.clone().into());
        let expr = expr.expr().map(|e| self.collect_expr(e));
        self.alloc_expr(RawExpr::Return { expr }, syntax_node_ptr)
    }

    fn collect_break(&mut self, expr: ast::BreakExpr) -> ExprId {
        let syntax_node_ptr = AstPtr::new(&expr.clone().into());
        let expr = expr.expr().map(|e| self.collect_expr(e));
        self.alloc_expr(RawExpr::Break { expr }, syntax_node_ptr)
    }

    fn collect_loop(&mut self, expr: ast::LoopExpr) -> ExprId {
        let syntax_node_ptr = AstPtr::new(&expr.clone().into());
        let body = self.collect_block_opt(expr.loop_body());
        self.alloc_expr(RawExpr::Loop { body }, syntax_node_ptr)
    }

    fn collect_while(&mut self, expr: ast::WhileExpr) -> ExprId {
        let syntax_node_ptr = AstPtr::new(&expr.clone().into());
        let condition = self.collect_condition_opt(expr.condition());
        let body = self.collect_block_opt(expr.loop_body());
        self.alloc_expr(RawExpr::While { condition, body }, syntax_node_ptr)
    }

    pub fn finish(mut self) -> (Body, BodySourceMap) {
        let (type_refs, type_ref_source_map) = self.type_ref_builder.finish();
        let body = Body {
            owner: self.owner,
            exprs: self.exprs,
            pats: self.pats,
            params: self.params,
            body_expr: self.body_expr.expect("A body should have been collected"),
            type_refs,
            ret_type: self
                .ret_type
                .expect("A body should have return type collected"),
        };
        mem::replace(&mut self.source_map.type_refs, type_ref_source_map);
        (body, self.source_map)
    }
}
