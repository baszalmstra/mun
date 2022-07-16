use crate::support::Project;

#[test]
fn test_hover() {
    let server = Project::with_fixture(
        r#"
    //- /mun.toml
    [package]
    name = "foo"
    version = "0.0.0"

    //- /src/mod.mun
    fn main() -> i32 {
        let a = 3;
        let b = a;
    }
    "#,
    )
    .server()
    .wait_until_workspace_is_loaded();

    let symbols = server.send_request::<lsp_types::request::HoverRequest>(lsp_types::HoverParams {
        text_document_position_params: lsp_types::TextDocumentPositionParams {
            text_document: server.doc_id("src/mod.mun"),
            position: lsp_types::Position {
                line: 0,
                character: 0,
            },
        },
        work_done_progress_params: Default::default(),
    });

    insta::assert_debug_snapshot!(symbols);
}
