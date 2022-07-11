use crate::ast::{self, child_opt, children, AstChildren, AstNode, AstToken};
use crate::syntax_node::SyntaxElementChildren;
use crate::SyntaxKind;

/// Defines that the node may contain [`ast::ModuleItem`]s.
pub trait HasModuleItem: AstNode {
    /// Returns all the [`ast::ModuleItem`]s contained in this instance.
    fn items(&self) -> AstChildren<ast::ModuleItem> {
        children(self)
    }
}

/// Defines that the node may contain [`ast::FunctionDef`]s.
pub trait HasFunctionDef: AstNode {
    /// Returns all the [`ast::FunctionDef`]s contained in this instance.
    fn functions(&self) -> AstChildren<ast::FunctionDef> {
        children(self)
    }
}

/// Defines that the node can have an associated name.
pub trait HasName: AstNode {
    /// Returns the name associated with the node.
    fn name(&self) -> Option<ast::Name> {
        child_opt(self)
    }
}

/// Defines that the node can have an associated type ascription (e.g. `a:i32`).
pub trait HasTypeAscription: AstNode {
    /// Returns the type associated with the node.
    fn ascribed_type(&self) -> Option<ast::TypeRef> {
        child_opt(self)
    }
}

/// Defines that the node might have a visibility specifier associated with it.
pub trait HasVisibility: AstNode {
    /// Returns the visibility associated with the node.
    fn visibility(&self) -> Option<ast::Visibility> {
        child_opt(self)
    }
}

/// Defines the the node might have a loop body associated with it
pub trait HasLoopBody: AstNode {
    /// Returns the loop body associated with the node.
    fn loop_body(&self) -> Option<ast::BlockExpr> {
        child_opt(self)
    }
}

/// Defines that the node might have an associated argument list.
pub trait HasArgList: AstNode {
    /// Returns the argument list associated with the node.
    fn arg_list(&self) -> Option<ast::ArgList> {
        child_opt(self)
    }
}

/// Defines that the node might have an associated doc comment.
pub trait HasDocComments: AstNode {
    /// Returns an iterator over the doc comments associated with the item.
    fn doc_comments(&self) -> DocCommentIter {
        DocCommentIter {
            iter: self.syntax().children_with_tokens(),
        }
    }
}

/// An iterator over the doc comments of an [`ast::AstNode`].
pub struct DocCommentIter {
    iter: SyntaxElementChildren,
}

impl DocCommentIter {
    /// Construct a joined complete string of all the doc comments.
    #[cfg(test)]
    pub fn doc_comment_text(self) -> Option<String> {
        let docs = itertools::Itertools::join(
            &mut self.filter_map(|comment| comment.doc_comment().map(ToOwned::to_owned)),
            "\n",
        );
        if docs.is_empty() {
            None
        } else {
            Some(docs)
        }
    }
}

impl Iterator for DocCommentIter {
    type Item = ast::Comment;
    fn next(&mut self) -> Option<ast::Comment> {
        self.iter.by_ref().find_map(|el| {
            el.into_token()
                .and_then(ast::Comment::cast)
                .filter(ast::Comment::is_doc)
        })
    }
}

/// Defines that the node might have an associated default type.
pub trait HasDefaultTypeParam: AstNode {
    /// Returns the default type associated with the node.
    fn default_type(&self) -> Option<ast::PathType> {
        child_opt(self)
    }
}

/// Defines that the node might have an `extern` keyword associated with.
pub trait HasExtern: AstNode {
    /// Returns true if the node has the `extern` keyword associated with it.
    fn is_extern(&self) -> bool {
        self.syntax()
            .children()
            .any(|p| p.kind() == SyntaxKind::EXTERN)
    }
}
