use crate::body::{ExprId, PatId};
use crate::type_ref::TypeRefId;
use crate::{Name, Path};
pub use mun_syntax::ast::PrefixOp as UnaryOp;

mod collector;
pub(crate) use collector::RawExprCollector;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RecordLitField {
    pub name: Name,
    pub expr: ExprId,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Statement {
    Let {
        pat: PatId,
        type_ref: Option<TypeRefId>,
        initializer: Option<ExprId>,
    },
    Expr(ExprId),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    String(String),
    Bool(bool),
    Int(i64),
    Float(f64),
}

impl Eq for Literal {}

/// A representation of the AST where every expression is mapped to an ExprId and some form of
/// desugaring has happened.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum RawExpr {
    /// Used if the syntax tree does not have a required expression piece
    Missing,
    Call {
        callee: ExprId,
        args: Vec<ExprId>,
    },
    Path(Path),
    If {
        condition: ExprId,
        then_branch: ExprId,
        else_branch: Option<ExprId>,
    },
    UnaryOp {
        expr: ExprId,
        op: UnaryOp,
    },
    BinaryOp {
        lhs: ExprId,
        rhs: ExprId,
        op: Option<BinaryOp>,
    },
    Block {
        statements: Vec<Statement>,
        tail: Option<ExprId>,
    },
    Return {
        expr: Option<ExprId>,
    },
    Break {
        expr: Option<ExprId>,
    },
    Loop {
        body: ExprId,
    },
    While {
        condition: ExprId,
        body: ExprId,
    },
    RecordLit {
        path: Option<Path>,
        fields: Vec<RecordLitField>,
        spread: Option<ExprId>,
    },
    Field {
        expr: ExprId,
        name: Name,
    },
    Literal(Literal),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum BinaryOp {
    LogicOp(LogicOp),
    ArithOp(ArithOp),
    CmpOp(CmpOp),
    Assignment { op: Option<ArithOp> },
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum LogicOp {
    And,
    Or,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum CmpOp {
    Eq { negated: bool },
    Ord { ordering: Ordering, strict: bool },
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Ordering {
    Less,
    Greater,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum ArithOp {
    Add,
    Multiply,
    Subtract,
    Divide,
    //Remainder,
    //Power,
}

impl RawExpr {
    pub fn walk_child_exprs(&self, mut f: impl FnMut(ExprId)) {
        match self {
            RawExpr::Missing => {}
            RawExpr::Path(_) => {}
            RawExpr::Block { statements, tail } => {
                for stmt in statements {
                    match stmt {
                        Statement::Let { initializer, .. } => {
                            if let Some(expr) = initializer {
                                f(*expr);
                            }
                        }
                        Statement::Expr(e) => f(*e),
                    }
                }
                if let Some(expr) = tail {
                    f(*expr);
                }
            }
            RawExpr::Call { callee, args } => {
                f(*callee);
                for arg in args {
                    f(*arg);
                }
            }
            RawExpr::BinaryOp { lhs, rhs, .. } => {
                f(*lhs);
                f(*rhs);
            }
            RawExpr::Field { expr, .. } | RawExpr::UnaryOp { expr, .. } => {
                f(*expr);
            }
            RawExpr::Literal(_) => {}
            RawExpr::If {
                condition,
                then_branch,
                else_branch,
            } => {
                f(*condition);
                f(*then_branch);
                if let Some(else_expr) = else_branch {
                    f(*else_expr);
                }
            }
            RawExpr::Return { expr } => {
                if let Some(expr) = expr {
                    f(*expr);
                }
            }
            RawExpr::Break { expr } => {
                if let Some(expr) = expr {
                    f(*expr);
                }
            }
            RawExpr::Loop { body } => {
                f(*body);
            }
            RawExpr::While { condition, body } => {
                f(*condition);
                f(*body);
            }
            RawExpr::RecordLit { fields, spread, .. } => {
                for field in fields {
                    f(field.expr);
                }
                if let Some(expr) = spread {
                    f(*expr);
                }
            }
        }
    }
}

/// Similar to `ast::PatKind`
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Pat {
    Missing,             // Indicates an error
    Wild,                // `_`
    Path(Path),          // E.g. `foo::bar`
    Bind { name: Name }, // E.g. `a`
}

impl Pat {
    pub fn walk_child_pats(&self, mut _f: impl FnMut(PatId)) {}
}
