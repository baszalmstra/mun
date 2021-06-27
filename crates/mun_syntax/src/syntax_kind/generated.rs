//! Generated file, do not edit by hand, see `crate/ra_tools/src/codegen`

// This file is automatically generated based on the file `./generated.rs.tera` when `cargo gen-syntax` is run
// Do not edit manually

#![allow(
    bad_style,
    missing_docs,
    unreachable_pub,
    clippy::manual_non_exhaustive,
    clippy::upper_case_acronyms
)]
use super::SyntaxInfo;

/// The kind of syntax node, e.g. `IDENT`, `USE_KW`, or `STRUCT_DEF`.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u16)]
#[non_exhaustive]
pub enum SyntaxKind {
    // Technical SyntaxKinds: they appear temporally during parsing,
    // but never end up in the final tree
    #[doc(hidden)]
    TOMBSTONE,
    #[doc(hidden)]
    EOF,
    AMP,
    PIPE,
    PLUS,
    MINUS,
    STAR,
    SLASH,
    PERCENT,
    CARET,
    HASH,
    DOT,
    LT,
    GT,
    EQ,
    L_PAREN,
    R_PAREN,
    L_CURLY,
    R_CURLY,
    L_BRACKET,
    R_BRACKET,
    SEMI,
    COLON,
    COMMA,
    EXCLAMATION,
    UNDERSCORE,
    EQEQ,
    NEQ,
    LTEQ,
    GTEQ,
    DOTDOT,
    DOTDOTDOT,
    PLUSEQ,
    MINUSEQ,
    STAREQ,
    SLASHEQ,
    PERCENTEQ,
    SHLEQ,
    SHREQ,
    AMPEQ,
    PIPEEQ,
    CARETEQ,
    DOTDOTEQ,
    COLONCOLON,
    THIN_ARROW,
    AMPAMP,
    PIPEPIPE,
    SHL,
    SHR,
    BREAK_KW,
    DO_KW,
    ELSE_KW,
    FALSE_KW,
    FOR_KW,
    FN_KW,
    IF_KW,
    IN_KW,
    AS_KW,
    USE_KW,
    NIL_KW,
    RETURN_KW,
    TRUE_KW,
    WHILE_KW,
    LOOP_KW,
    LET_KW,
    MUT_KW,
    CLASS_KW,
    STRUCT_KW,
    NEVER_KW,
    PUB_KW,
    TYPE_KW,
    PACKAGE_KW,
    SUPER_KW,
    SELF_KW,
    EXTERN_KW,
    INT_NUMBER,
    FLOAT_NUMBER,
    STRING,
    ERROR,
    IDENT,
    INDEX,
    WHITESPACE,
    COMMENT,
    GC_KW,
    VALUE_KW,
    SOURCE_FILE,
    FUNCTION_DEF,
    EXTERN,
    RET_TYPE,
    VISIBILITY,
    PARAM_LIST,
    PARAM,
    STRUCT_DEF,
    TYPE_ALIAS_DEF,
    MEMORY_TYPE_SPECIFIER,
    RECORD_FIELD_DEF_LIST,
    RECORD_FIELD_DEF,
    TUPLE_FIELD_DEF_LIST,
    TUPLE_FIELD_DEF,
    PATH_TYPE,
    ARRAY_TYPE,
    NEVER_TYPE,
    LET_STMT,
    EXPR_STMT,
    PATH_EXPR,
    PREFIX_EXPR,
    LITERAL,
    BIN_EXPR,
    PAREN_EXPR,
    CALL_EXPR,
    FIELD_EXPR,
    IF_EXPR,
    INDEX_EXPR,
    BLOCK_EXPR,
    RETURN_EXPR,
    WHILE_EXPR,
    LOOP_EXPR,
    BREAK_EXPR,
    ARRAY_EXPR,
    CONDITION,
    BIND_PAT,
    PLACEHOLDER_PAT,
    ARG_LIST,
    NAME,
    NAME_REF,
    PATH,
    PATH_SEGMENT,
    RECORD_LIT,
    RECORD_FIELD_LIST,
    RECORD_FIELD,
    USE,
    USE_TREE,
    USE_TREE_LIST,
    RENAME,
    // Technical kind so that we can cast from u16 safely
    #[doc(hidden)]
    __LAST,
}
use self::SyntaxKind::*;

#[macro_export]
macro_rules! T {
    (&) => {
        $crate::SyntaxKind::AMP
    };
    (|) => {
        $crate::SyntaxKind::PIPE
    };
    (+) => {
        $crate::SyntaxKind::PLUS
    };
    (-) => {
        $crate::SyntaxKind::MINUS
    };
    (*) => {
        $crate::SyntaxKind::STAR
    };
    (/) => {
        $crate::SyntaxKind::SLASH
    };
    (%) => {
        $crate::SyntaxKind::PERCENT
    };
    (^) => {
        $crate::SyntaxKind::CARET
    };
    (#) => {
        $crate::SyntaxKind::HASH
    };
    (.) => {
        $crate::SyntaxKind::DOT
    };
    (<) => {
        $crate::SyntaxKind::LT
    };
    (>) => {
        $crate::SyntaxKind::GT
    };
    (=) => {
        $crate::SyntaxKind::EQ
    };
    ('(') => {
        $crate::SyntaxKind::L_PAREN
    };
    (')') => {
        $crate::SyntaxKind::R_PAREN
    };
    ('{') => {
        $crate::SyntaxKind::L_CURLY
    };
    ('}') => {
        $crate::SyntaxKind::R_CURLY
    };
    ('[') => {
        $crate::SyntaxKind::L_BRACKET
    };
    (']') => {
        $crate::SyntaxKind::R_BRACKET
    };
    (;) => {
        $crate::SyntaxKind::SEMI
    };
    (:) => {
        $crate::SyntaxKind::COLON
    };
    (,) => {
        $crate::SyntaxKind::COMMA
    };
    (!) => {
        $crate::SyntaxKind::EXCLAMATION
    };
    (_) => {
        $crate::SyntaxKind::UNDERSCORE
    };
    (==) => {
        $crate::SyntaxKind::EQEQ
    };
    (!=) => {
        $crate::SyntaxKind::NEQ
    };
    (<=) => {
        $crate::SyntaxKind::LTEQ
    };
    (>=) => {
        $crate::SyntaxKind::GTEQ
    };
    (..) => {
        $crate::SyntaxKind::DOTDOT
    };
    (...) => {
        $crate::SyntaxKind::DOTDOTDOT
    };
    (+=) => {
        $crate::SyntaxKind::PLUSEQ
    };
    (-=) => {
        $crate::SyntaxKind::MINUSEQ
    };
    (*=) => {
        $crate::SyntaxKind::STAREQ
    };
    (/=) => {
        $crate::SyntaxKind::SLASHEQ
    };
    (%=) => {
        $crate::SyntaxKind::PERCENTEQ
    };
    (<<=) => {
        $crate::SyntaxKind::SHLEQ
    };
    (>>=) => {
        $crate::SyntaxKind::SHREQ
    };
    (&=) => {
        $crate::SyntaxKind::AMPEQ
    };
    (|=) => {
        $crate::SyntaxKind::PIPEEQ
    };
    (^=) => {
        $crate::SyntaxKind::CARETEQ
    };
    (..=) => {
        $crate::SyntaxKind::DOTDOTEQ
    };
    (::) => {
        $crate::SyntaxKind::COLONCOLON
    };
    (->) => {
        $crate::SyntaxKind::THIN_ARROW
    };
    (&&) => {
        $crate::SyntaxKind::AMPAMP
    };
    (||) => {
        $crate::SyntaxKind::PIPEPIPE
    };
    (<<) => {
        $crate::SyntaxKind::SHL
    };
    (>>) => {
        $crate::SyntaxKind::SHR
    };
    (break) => {
        $crate::SyntaxKind::BREAK_KW
    };
    (do) => {
        $crate::SyntaxKind::DO_KW
    };
    (else) => {
        $crate::SyntaxKind::ELSE_KW
    };
    (false) => {
        $crate::SyntaxKind::FALSE_KW
    };
    (for) => {
        $crate::SyntaxKind::FOR_KW
    };
    (fn) => {
        $crate::SyntaxKind::FN_KW
    };
    (if) => {
        $crate::SyntaxKind::IF_KW
    };
    (in) => {
        $crate::SyntaxKind::IN_KW
    };
    (as) => {
        $crate::SyntaxKind::AS_KW
    };
    (use) => {
        $crate::SyntaxKind::USE_KW
    };
    (nil) => {
        $crate::SyntaxKind::NIL_KW
    };
    (return) => {
        $crate::SyntaxKind::RETURN_KW
    };
    (true) => {
        $crate::SyntaxKind::TRUE_KW
    };
    (while) => {
        $crate::SyntaxKind::WHILE_KW
    };
    (loop) => {
        $crate::SyntaxKind::LOOP_KW
    };
    (let) => {
        $crate::SyntaxKind::LET_KW
    };
    (mut) => {
        $crate::SyntaxKind::MUT_KW
    };
    (class) => {
        $crate::SyntaxKind::CLASS_KW
    };
    (struct) => {
        $crate::SyntaxKind::STRUCT_KW
    };
    (never) => {
        $crate::SyntaxKind::NEVER_KW
    };
    (pub) => {
        $crate::SyntaxKind::PUB_KW
    };
    (type) => {
        $crate::SyntaxKind::TYPE_KW
    };
    (package) => {
        $crate::SyntaxKind::PACKAGE_KW
    };
    (super) => {
        $crate::SyntaxKind::SUPER_KW
    };
    (self) => {
        $crate::SyntaxKind::SELF_KW
    };
    (extern) => {
        $crate::SyntaxKind::EXTERN_KW
    };
}

impl From<u16> for SyntaxKind {
    fn from(d: u16) -> SyntaxKind {
        assert!(d <= (__LAST as u16));
        unsafe { std::mem::transmute::<u16, SyntaxKind>(d) }
    }
}

impl From<SyntaxKind> for u16 {
    fn from(k: SyntaxKind) -> u16 {
        k as u16
    }
}

impl SyntaxKind {
    #[rustfmt::skip]
    pub fn is_keyword(self) -> bool {
        matches!(self,
        BREAK_KW
        | DO_KW
        | ELSE_KW
        | FALSE_KW
        | FOR_KW
        | FN_KW
        | IF_KW
        | IN_KW
        | AS_KW
        | USE_KW
        | NIL_KW
        | RETURN_KW
        | TRUE_KW
        | WHILE_KW
        | LOOP_KW
        | LET_KW
        | MUT_KW
        | CLASS_KW
        | STRUCT_KW
        | NEVER_KW
        | PUB_KW
        | TYPE_KW
        | PACKAGE_KW
        | SUPER_KW
        | SELF_KW
        | EXTERN_KW
        )
    }

    #[rustfmt::skip]
    pub fn is_symbol(self) -> bool {
        matches!(self,
        AMP
        | PIPE
        | PLUS
        | MINUS
        | STAR
        | SLASH
        | PERCENT
        | CARET
        | HASH
        | DOT
        | LT
        | GT
        | EQ
        | L_PAREN
        | R_PAREN
        | L_CURLY
        | R_CURLY
        | L_BRACKET
        | R_BRACKET
        | SEMI
        | COLON
        | COMMA
        | EXCLAMATION
        | UNDERSCORE
        | EQEQ
        | NEQ
        | LTEQ
        | GTEQ
        | DOTDOT
        | DOTDOTDOT
        | PLUSEQ
        | MINUSEQ
        | STAREQ
        | SLASHEQ
        | PERCENTEQ
        | SHLEQ
        | SHREQ
        | AMPEQ
        | PIPEEQ
        | CARETEQ
        | DOTDOTEQ
        | COLONCOLON
        | THIN_ARROW
        | AMPAMP
        | PIPEPIPE
        | SHL
        | SHR
        )
    }

    #[rustfmt::skip]
    pub fn is_literal(self) -> bool {
        matches!(self,
            INT_NUMBER
            | FLOAT_NUMBER
            | STRING
        )
    }

    #[rustfmt::skip]
    pub(crate) fn info(self) -> &'static SyntaxInfo {
        match self {
            AMP => &SyntaxInfo { name: "AMP" },
            PIPE => &SyntaxInfo { name: "PIPE" },
            PLUS => &SyntaxInfo { name: "PLUS" },
            MINUS => &SyntaxInfo { name: "MINUS" },
            STAR => &SyntaxInfo { name: "STAR" },
            SLASH => &SyntaxInfo { name: "SLASH" },
            PERCENT => &SyntaxInfo { name: "PERCENT" },
            CARET => &SyntaxInfo { name: "CARET" },
            HASH => &SyntaxInfo { name: "HASH" },
            DOT => &SyntaxInfo { name: "DOT" },
            LT => &SyntaxInfo { name: "LT" },
            GT => &SyntaxInfo { name: "GT" },
            EQ => &SyntaxInfo { name: "EQ" },
            L_PAREN => &SyntaxInfo { name: "L_PAREN" },
            R_PAREN => &SyntaxInfo { name: "R_PAREN" },
            L_CURLY => &SyntaxInfo { name: "L_CURLY" },
            R_CURLY => &SyntaxInfo { name: "R_CURLY" },
            L_BRACKET => &SyntaxInfo { name: "L_BRACKET" },
            R_BRACKET => &SyntaxInfo { name: "R_BRACKET" },
            SEMI => &SyntaxInfo { name: "SEMI" },
            COLON => &SyntaxInfo { name: "COLON" },
            COMMA => &SyntaxInfo { name: "COMMA" },
            EXCLAMATION => &SyntaxInfo { name: "EXCLAMATION" },
            UNDERSCORE => &SyntaxInfo { name: "UNDERSCORE" },
            EQEQ => &SyntaxInfo { name: "EQEQ" },
            NEQ => &SyntaxInfo { name: "NEQ" },
            LTEQ => &SyntaxInfo { name: "LTEQ" },
            GTEQ => &SyntaxInfo { name: "GTEQ" },
            DOTDOT => &SyntaxInfo { name: "DOTDOT" },
            DOTDOTDOT => &SyntaxInfo { name: "DOTDOTDOT" },
            PLUSEQ => &SyntaxInfo { name: "PLUSEQ" },
            MINUSEQ => &SyntaxInfo { name: "MINUSEQ" },
            STAREQ => &SyntaxInfo { name: "STAREQ" },
            SLASHEQ => &SyntaxInfo { name: "SLASHEQ" },
            PERCENTEQ => &SyntaxInfo { name: "PERCENTEQ" },
            SHLEQ => &SyntaxInfo { name: "SHLEQ" },
            SHREQ => &SyntaxInfo { name: "SHREQ" },
            AMPEQ => &SyntaxInfo { name: "AMPEQ" },
            PIPEEQ => &SyntaxInfo { name: "PIPEEQ" },
            CARETEQ => &SyntaxInfo { name: "CARETEQ" },
            DOTDOTEQ => &SyntaxInfo { name: "DOTDOTEQ" },
            COLONCOLON => &SyntaxInfo { name: "COLONCOLON" },
            THIN_ARROW => &SyntaxInfo { name: "THIN_ARROW" },
            AMPAMP => &SyntaxInfo { name: "AMPAMP" },
            PIPEPIPE => &SyntaxInfo { name: "PIPEPIPE" },
            SHL => &SyntaxInfo { name: "SHL" },
            SHR => &SyntaxInfo { name: "SHR" },
            BREAK_KW => &SyntaxInfo { name: "BREAK_KW" },
            DO_KW => &SyntaxInfo { name: "DO_KW" },
            ELSE_KW => &SyntaxInfo { name: "ELSE_KW" },
            FALSE_KW => &SyntaxInfo { name: "FALSE_KW" },
            FOR_KW => &SyntaxInfo { name: "FOR_KW" },
            FN_KW => &SyntaxInfo { name: "FN_KW" },
            IF_KW => &SyntaxInfo { name: "IF_KW" },
            IN_KW => &SyntaxInfo { name: "IN_KW" },
            AS_KW => &SyntaxInfo { name: "AS_KW" },
            USE_KW => &SyntaxInfo { name: "USE_KW" },
            NIL_KW => &SyntaxInfo { name: "NIL_KW" },
            RETURN_KW => &SyntaxInfo { name: "RETURN_KW" },
            TRUE_KW => &SyntaxInfo { name: "TRUE_KW" },
            WHILE_KW => &SyntaxInfo { name: "WHILE_KW" },
            LOOP_KW => &SyntaxInfo { name: "LOOP_KW" },
            LET_KW => &SyntaxInfo { name: "LET_KW" },
            MUT_KW => &SyntaxInfo { name: "MUT_KW" },
            CLASS_KW => &SyntaxInfo { name: "CLASS_KW" },
            STRUCT_KW => &SyntaxInfo { name: "STRUCT_KW" },
            NEVER_KW => &SyntaxInfo { name: "NEVER_KW" },
            PUB_KW => &SyntaxInfo { name: "PUB_KW" },
            TYPE_KW => &SyntaxInfo { name: "TYPE_KW" },
            PACKAGE_KW => &SyntaxInfo { name: "PACKAGE_KW" },
            SUPER_KW => &SyntaxInfo { name: "SUPER_KW" },
            SELF_KW => &SyntaxInfo { name: "SELF_KW" },
            EXTERN_KW => &SyntaxInfo { name: "EXTERN_KW" },
            INT_NUMBER => &SyntaxInfo { name: "INT_NUMBER" },
            FLOAT_NUMBER => &SyntaxInfo { name: "FLOAT_NUMBER" },
            STRING => &SyntaxInfo { name: "STRING" },
            ERROR => &SyntaxInfo { name: "ERROR" },
            IDENT => &SyntaxInfo { name: "IDENT" },
            INDEX => &SyntaxInfo { name: "INDEX" },
            WHITESPACE => &SyntaxInfo { name: "WHITESPACE" },
            COMMENT => &SyntaxInfo { name: "COMMENT" },
            GC_KW => &SyntaxInfo { name: "GC_KW" },
            VALUE_KW => &SyntaxInfo { name: "VALUE_KW" },
            SOURCE_FILE => &SyntaxInfo { name: "SOURCE_FILE" },
            FUNCTION_DEF => &SyntaxInfo { name: "FUNCTION_DEF" },
            EXTERN => &SyntaxInfo { name: "EXTERN" },
            RET_TYPE => &SyntaxInfo { name: "RET_TYPE" },
            VISIBILITY => &SyntaxInfo { name: "VISIBILITY" },
            PARAM_LIST => &SyntaxInfo { name: "PARAM_LIST" },
            PARAM => &SyntaxInfo { name: "PARAM" },
            STRUCT_DEF => &SyntaxInfo { name: "STRUCT_DEF" },
            TYPE_ALIAS_DEF => &SyntaxInfo { name: "TYPE_ALIAS_DEF" },
            MEMORY_TYPE_SPECIFIER => &SyntaxInfo { name: "MEMORY_TYPE_SPECIFIER" },
            RECORD_FIELD_DEF_LIST => &SyntaxInfo { name: "RECORD_FIELD_DEF_LIST" },
            RECORD_FIELD_DEF => &SyntaxInfo { name: "RECORD_FIELD_DEF" },
            TUPLE_FIELD_DEF_LIST => &SyntaxInfo { name: "TUPLE_FIELD_DEF_LIST" },
            TUPLE_FIELD_DEF => &SyntaxInfo { name: "TUPLE_FIELD_DEF" },
            PATH_TYPE => &SyntaxInfo { name: "PATH_TYPE" },
            ARRAY_TYPE => &SyntaxInfo { name: "ARRAY_TYPE" },
            NEVER_TYPE => &SyntaxInfo { name: "NEVER_TYPE" },
            LET_STMT => &SyntaxInfo { name: "LET_STMT" },
            EXPR_STMT => &SyntaxInfo { name: "EXPR_STMT" },
            PATH_EXPR => &SyntaxInfo { name: "PATH_EXPR" },
            PREFIX_EXPR => &SyntaxInfo { name: "PREFIX_EXPR" },
            LITERAL => &SyntaxInfo { name: "LITERAL" },
            BIN_EXPR => &SyntaxInfo { name: "BIN_EXPR" },
            PAREN_EXPR => &SyntaxInfo { name: "PAREN_EXPR" },
            CALL_EXPR => &SyntaxInfo { name: "CALL_EXPR" },
            FIELD_EXPR => &SyntaxInfo { name: "FIELD_EXPR" },
            IF_EXPR => &SyntaxInfo { name: "IF_EXPR" },
            INDEX_EXPR => &SyntaxInfo { name: "INDEX_EXPR" },
            BLOCK_EXPR => &SyntaxInfo { name: "BLOCK_EXPR" },
            RETURN_EXPR => &SyntaxInfo { name: "RETURN_EXPR" },
            WHILE_EXPR => &SyntaxInfo { name: "WHILE_EXPR" },
            LOOP_EXPR => &SyntaxInfo { name: "LOOP_EXPR" },
            BREAK_EXPR => &SyntaxInfo { name: "BREAK_EXPR" },
            ARRAY_EXPR => &SyntaxInfo { name: "ARRAY_EXPR" },
            CONDITION => &SyntaxInfo { name: "CONDITION" },
            BIND_PAT => &SyntaxInfo { name: "BIND_PAT" },
            PLACEHOLDER_PAT => &SyntaxInfo { name: "PLACEHOLDER_PAT" },
            ARG_LIST => &SyntaxInfo { name: "ARG_LIST" },
            NAME => &SyntaxInfo { name: "NAME" },
            NAME_REF => &SyntaxInfo { name: "NAME_REF" },
            PATH => &SyntaxInfo { name: "PATH" },
            PATH_SEGMENT => &SyntaxInfo { name: "PATH_SEGMENT" },
            RECORD_LIT => &SyntaxInfo { name: "RECORD_LIT" },
            RECORD_FIELD_LIST => &SyntaxInfo { name: "RECORD_FIELD_LIST" },
            RECORD_FIELD => &SyntaxInfo { name: "RECORD_FIELD" },
            USE => &SyntaxInfo { name: "USE" },
            USE_TREE => &SyntaxInfo { name: "USE_TREE" },
            USE_TREE_LIST => &SyntaxInfo { name: "USE_TREE_LIST" },
            RENAME => &SyntaxInfo { name: "RENAME" },
            TOMBSTONE => &SyntaxInfo { name: "TOMBSTONE" },
            EOF => &SyntaxInfo { name: "EOF" },
            __LAST => &SyntaxInfo { name: "__LAST" },
        }
    }

    pub fn from_keyword(ident: &str) -> Option<SyntaxKind> {
        let kw = match ident {
            "break" => BREAK_KW,
            "do" => DO_KW,
            "else" => ELSE_KW,
            "false" => FALSE_KW,
            "for" => FOR_KW,
            "fn" => FN_KW,
            "if" => IF_KW,
            "in" => IN_KW,
            "as" => AS_KW,
            "use" => USE_KW,
            "nil" => NIL_KW,
            "return" => RETURN_KW,
            "true" => TRUE_KW,
            "while" => WHILE_KW,
            "loop" => LOOP_KW,
            "let" => LET_KW,
            "mut" => MUT_KW,
            "class" => CLASS_KW,
            "struct" => STRUCT_KW,
            "never" => NEVER_KW,
            "pub" => PUB_KW,
            "type" => TYPE_KW,
            "package" => PACKAGE_KW,
            "super" => SUPER_KW,
            "self" => SELF_KW,
            "extern" => EXTERN_KW,
            _ => return None,
        };
        Some(kw)
    }

    pub fn from_char(c: char) -> Option<SyntaxKind> {
        let tok = match c {
            '&' => AMP,
            '|' => PIPE,
            '+' => PLUS,
            '-' => MINUS,
            '*' => STAR,
            '/' => SLASH,
            '%' => PERCENT,
            '^' => CARET,
            '#' => HASH,
            '.' => DOT,
            '<' => LT,
            '>' => GT,
            '=' => EQ,
            '(' => L_PAREN,
            ')' => R_PAREN,
            '{' => L_CURLY,
            '}' => R_CURLY,
            '[' => L_BRACKET,
            ']' => R_BRACKET,
            ';' => SEMI,
            ':' => COLON,
            ',' => COMMA,
            '!' => EXCLAMATION,
            '_' => UNDERSCORE,
            _ => return None,
        };
        Some(tok)
    }
}
