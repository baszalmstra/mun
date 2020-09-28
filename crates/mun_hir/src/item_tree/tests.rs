use crate::{
    fixture::WithFixture,
    item_tree::Fields,
    item_tree::{ItemTree, ModItem},
    mock::MockDatabase,
    DefDatabase,
};
use std::{fmt, fmt::Write, sync::Arc};

fn item_tree(text: &str) -> Arc<ItemTree> {
    let (db, file_id) = MockDatabase::with_single_file(text);
    db.item_tree(file_id.into())
}

fn print_item_tree(text: &str) -> Result<String, fmt::Error> {
    let tree = item_tree(text);
    let mut out = String::new();
    write!(&mut out, "top-level items:\n")?;
    for item in tree.top_level_items() {
        format_mod_item(&mut out, &tree, *item)?;
        write!(&mut out, "\n")?;
    }

    Ok(out)
}

fn format_mod_item(out: &mut String, tree: &ItemTree, item: ModItem) -> fmt::Result {
    let mut children = String::new();
    match item {
        ModItem::Function(item) => {
            write!(out, "{:?}", tree[item])?;
        }
        ModItem::Struct(item) => {
            write!(out, "{:?}", tree[item])?;
            match &tree[item].fields {
                Fields::Record(a) | Fields::Tuple(a) => {
                    for field in a.clone() {
                        write!(children, "{:?}\n", tree[field])?;
                    }
                }
                _ => {}
            };
        }
        ModItem::TypeAlias(item) => {
            write!(out, "{:?}", tree[item])?;
        }
    }

    for line in children.lines() {
        write!(out, "\n> {}", line)?;
    }

    Ok(())
}

#[test]
fn top_level_items() {
    insta::assert_snapshot!(print_item_tree(
        r#"
    fn foo(a:i32, b:u8, c:String) -> i32 {}
    fn bar(a:i32, b:u8, c:String) ->  {}
    fn baz(a:i32, b:, c:String) ->  {}
    extern fn eval(a:String) -> bool;

    struct Foo {
        a: i32,
        b: u8,
        c: String,
    }
    struct Foo2 {
        a: i32,
        b: ,
        c: String,
    }
    struct Bar (i32, u32, String)
    struct Baz;

    type FooBar = Foo;
    type FooBar = package::Foo;
    "#
    )
    .unwrap());
}
