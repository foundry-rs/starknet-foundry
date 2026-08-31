use super::common::runner::{setup_package, test_runner};
use assert_fs::fixture::{FileWriteStr, PathChild};
use axum::{Router, extract::Query, response::Redirect, routing::any};
use indoc::formatdoc;
use shared::consts::EXPECTED_RPC_VERSION;
use shared::test_utils::node_url::node_url;
use shared::test_utils::output_assert::assert_stdout_contains;
use std::sync::LazyLock;
use std::{thread::sleep, time::Duration};
use tokio::{
    net::TcpListener,
    runtime::{Builder, Runtime},
};

#[derive(serde::Deserialize)]
struct Params {
    url: String,
}

// to make one url look like different ones
fn setup_redirect_server() {
    static RT: LazyLock<Runtime> =
        LazyLock::new(|| Builder::new_multi_thread().enable_all().build().unwrap());

    RT.spawn(async {
        let app = Router::new().route(
            "/",
            any(|params: Query<Params>| async move { Redirect::permanent(&params.url) }),
        );

        let listener = TcpListener::bind("127.0.0.1:3030").await.unwrap();

        axum::serve(listener, app).await.unwrap();
    });

    // if test uses server make it wait for a second before it's ready
    sleep(Duration::from_secs(1));
}

#[test]
fn should_print_warning() {
    let temp = setup_package("empty");
    let mut node_url = node_url();
    node_url.set_path("rpc/v0_9");

    temp.child("tests/test.cairo")
        .write_str(
            formatdoc!(
                r#"
                #[fork(url: "{node_url}", block_tag: latest)]
                #[test]
                fn t1() {{
                    assert!(false);
                }}
            "#
            )
            .as_str(),
        )
        .unwrap();

    let output = test_runner(&temp).assert();

    assert_stdout_contains(
        output,
        formatdoc!(
            r"
                [..]Compiling[..]
                [..]Finished[..]
                [WARNING] RPC node with the url {node_url} uses incompatible version 0.9.0. Expected version: {EXPECTED_RPC_VERSION}


                Collected 1 test(s) from empty package
            "
        ),
    );
}

#[test]
fn should_dedup_urls() {
    let temp = setup_package("empty");
    let mut node_url = node_url();
    node_url.set_path("rpc/v0_9");

    temp.child("tests/test.cairo")
        .write_str(
            formatdoc!(
                r#"
                #[fork(url: "{node_url}", block_tag: latest)]
                #[test]
                fn t1() {{
                    assert!(false);
                }}
                #[fork(url: "{node_url}", block_tag: latest)]
                #[test]
                fn t2() {{
                    assert!(false);
                }}
            "#
            )
            .as_str(),
        )
        .unwrap();

    let output = test_runner(&temp).assert();

    assert_stdout_contains(
        output,
        formatdoc!(
            r"
                [..]Compiling[..]
                [..]Finished[..]
                [WARNING] RPC node with the url {node_url} uses incompatible version 0.9.0. Expected version: {EXPECTED_RPC_VERSION}


                Collected 2 test(s) from empty package
            "
        ),
    );
}

#[test]
fn should_print_foreach() {
    setup_redirect_server();

    let temp = setup_package("empty");
    let mut node_url = node_url();
    node_url.set_path("rpc/v0_9");

    temp.child("tests/test.cairo")
        .write_str(
            formatdoc!(
                r#"
                #[fork(url: "http://127.0.0.1:3030?url={node_url}", block_tag: latest)]
                #[test]
                fn t1() {{
                    assert!(false);
                }}
                #[fork(url: "{node_url}", block_tag: latest)]
                #[test]
                fn t2() {{
                    assert!(false);
                }}
            "#
            )
            .as_str(),
        )
        .unwrap();

    let output = test_runner(&temp).assert();

    assert_stdout_contains(
        output,
        formatdoc!(
            r"
                [..]Compiling[..]
                [..]Finished[..]
                [WARNING] RPC node with the url http://127.0.0.1:3030/?url={node_url} uses incompatible version 0.9.0. Expected version: {EXPECTED_RPC_VERSION}
                [WARNING] RPC node with the url {node_url} uses incompatible version 0.9.0. Expected version: {EXPECTED_RPC_VERSION}


                Collected 2 test(s) from empty package
            "
        ),
    );
}
