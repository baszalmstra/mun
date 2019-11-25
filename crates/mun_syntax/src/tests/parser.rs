use crate::SourceFile;

fn snapshot_test(text: &str) {
    let text = text.trim().replace("\n    ", "\n");
    let file = SourceFile::parse(&text);
    insta::assert_snapshot!(insta::_macro_support::AutoName, file.debug_dump(), &text);
}

#[test]
fn empty() {
    snapshot_test(r#""#);
}

#[test]
fn function() {
    snapshot_test(
        r#"
    // Source file comment

    // Comment that belongs to the function
    fn a() {}
    fn b(value:number) {}
    export fn c():never {}
    fn b(value:number):number {}"#,
    );
}

#[test]
fn block() {
    snapshot_test(
        r#"
    fn foo() {
        let a;
        let b:i32;
        let c:string;
    }"#,
    );
}

#[test]
fn literals() {
    snapshot_test(
        r#"
    fn foo() {
        let a = true;
        let b = false;
        let c = 1;
        let d = 1.12;
        let e = "Hello, world!"
    }
    "#,
    );
}

#[test]
fn unary_expr() {
    snapshot_test(
        r#"
    fn foo() {
        let a = --3;
        let b = !!true;
    }
    "#,
    )
}

#[test]
fn binary_expr() {
    snapshot_test(
        r#"
    fn foo() {
        let a = 3+4*5
        let b = 3*4+10/2
    }
    "#,
    )
}

#[test]
fn expression_statement() {
    snapshot_test(
        r#"
    fn foo() {
        let a = "hello"
        let b = "world"
        let c
        b = "Hello, world!"
        !-5+2*(a+b);
        -3
    }
    "#,
    )
}

#[test]
fn function_calls() {
    snapshot_test(
        r#"
    fn bar(i:number) { }
    fn foo(i:number) {
      bar(i+1)
    }
    "#,
    )
}

#[test]
fn patterns() {
    snapshot_test(
        r#"
    fn main(_:number) {
       let a = 0;
       let _ = a;
    }
    "#,
    )
}

#[test]
fn compare_operands() {
    snapshot_test(
        r#"
    fn main() {
        let _ = a==b;
        let _ = a == b;
        let _ = a != b;
        let _ = a < b;
        let _ = a > b;
        let _ = a <= b;
        let _ = a >= b;
    }
    "#,
    )
}

#[test]
fn if_expr() {
    snapshot_test(
        r#"
    fn bar() {
        if true {};
        if true {} else {};
        if true {} else if false {} else {};
        if {true} {} else {}
    }
    "#,
    );
}

#[test]
fn block_expr() {
    snapshot_test(
        r#"
    fn bar() {
        {3}
    }
    "#,
    );
}

#[test]
fn return_expr() {
    snapshot_test(
        r#"
    fn foo() {
        return;
        return 50;
    }
    "#,
    )
}

#[test]
fn loop_expr() {
    snapshot_test(
        r#"
    fn foo() {
        loop {}
    }"#,
    )
}

#[test]
fn break_expr() {
    snapshot_test(
        r#"
    fn foo() {
        break;
        if break { 3; }
        if break 4 { 3; }
    }
    "#,
    )
}

#[test]
fn while_expr() {
    snapshot_test(
        r#"
    fn foo() {
        while true {};
        while { true } {};
    }
    "#,
    )
}

#[test]
fn range_expr() {
    snapshot_test(
        r#"
    fn foo() {
        1..2;
        3..;
        ..4;
        ..;
        5..=6;
        ..=7;
        3..=; // Error: an inclusive range must have an end expression
        ..=;  // Error: an inclusive range must have an end expression
    }
    "#,
    )
}

#[test]
fn postfix_range() {
    snapshot_test(
        r#"
    fn foo() {
        let x = 1..;
        if 1.. {};
    }
    "#,
    )
}
