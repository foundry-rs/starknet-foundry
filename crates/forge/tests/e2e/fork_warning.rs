use super::common::runner::{setup_package, test_runner};
use assert_fs::fixture::{FileWriteStr, PathChild};
use axum::{extract::Query, response::Redirect, routing::any, Router};
use indoc::{formatdoc, indoc};
use lazy_static::lazy_static;
use shared::{consts::EXPECTED_RPC_VERSION, test_utils::output_assert::assert_stdout_contains};
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
    lazy_static! {
        static ref RT: Runtime = Builder::new_multi_thread().enable_all().build().unwrap();
    };

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

    temp.child("tests/test.cairo")
        .write_str(indoc!(
            r#"
                #[fork(url: "http://188.34.188.184:9545/rpc/v0_5", block_id: BlockId::Tag(BlockTag::Latest))]
                #[test]
                fn t1() {
                    assert!(false);
                }
            "#
        ))
        .unwrap();

    let output = test_runner(&temp).assert();

    assert_stdout_contains(
        output,
        formatdoc!(
            r"
                [..]Compiling[..]
                [..]Finished[..]
                [WARNING] RPC node with the url http://188.34.188.184:9545/rpc/v0_5 uses incompatible version 0.5.1. Expected version: {EXPECTED_RPC_VERSION}


                Collected 1 test(s) from empty package
                Running 0 test(s) from src/
                Running 1 test(s) from tests/
                [FAIL] tests::test::t1

                Failure[..]
                Tests: 0 passed, 1 failed, 0 skipped, 0 ignored, 0 filtered out

                Latest block number = [..] for url = http://188.34.188.184:9545/rpc/v0_5

                Failures:
                    tests::test::t1
            "
        ),
    );
}

#[test]
fn should_dedup_urls() {
    let temp = setup_package("empty");

    temp.child("tests/test.cairo")
        .write_str(indoc!(
            r#"
                #[fork(url: "http://188.34.188.184:9545/rpc/v0_5", block_id: BlockId::Tag(BlockTag::Latest))]
                #[test]
                fn t1() {
                    assert!(false);
                }
                #[fork(url: "http://188.34.188.184:9545/rpc/v0_5", block_id: BlockId::Tag(BlockTag::Latest))]
                #[test]
                fn t2() {
                    assert!(false);
                }
            "#
        ))
        .unwrap();

    let output = test_runner(&temp).assert();

    assert_stdout_contains(
        output,
        formatdoc!(
            r"
                [..]Compiling[..]
                [..]Finished[..]
                [WARNING] RPC node with the url http://188.34.188.184:9545/rpc/v0_5 uses incompatible version 0.5.1. Expected version: {EXPECTED_RPC_VERSION}


                Collected 2 test(s) from empty package
                Running 0 test(s) from src/
                Running 2 test(s) from tests/
                [FAIL] tests::test::t1

                Failure[..]
                [FAIL] tests::test::t2

                Failure[..]
                Tests: 0 passed, 2 failed, 0 skipped, 0 ignored, 0 filtered out

                Latest block number = [..] for url = http://188.34.188.184:9545/rpc/v0_5

                Failures:
                    tests::test::t1
                    tests::test::t2
            "
        ),
    );
}

#[test]
fn should_print_foreach() {
    setup_redirect_server();

    let temp = setup_package("empty");

    temp.child("tests/test.cairo")
        .write_str(indoc!(
            r#"
                #[fork(url: "http://127.0.0.1:3030?url=http://188.34.188.184:9545/rpc/v0_5", block_id: BlockId::Tag(BlockTag::Latest))]
                #[test]
                fn t1() {
                    assert!(false);
                }
                #[fork(url: "http://188.34.188.184:9545/rpc/v0_5", block_id: BlockId::Tag(BlockTag::Latest))]
                #[test]
                fn t2() {
                    assert!(false);
                }
            "#
        ))
        .unwrap();

    let output = test_runner(&temp).assert();

    assert_stdout_contains(
        output,
        formatdoc!(
            r"
                [..]Compiling[..]
                [..]Finished[..]
                [WARNING] RPC node with the url http://127.0.0.1:3030?url=http://188.34.188.184:9545/rpc/v0_5 uses incompatible version 0.5.1. Expected version: {EXPECTED_RPC_VERSION}
                [WARNING] RPC node with the url http://188.34.188.184:9545/rpc/v0_5 uses incompatible version 0.5.1. Expected version: {EXPECTED_RPC_VERSION}


                Collected 2 test(s) from empty package
                Running 0 test(s) from src/
                Running 2 test(s) from tests/
                [FAIL] tests::test::t1

                Failure[..]
                [FAIL] tests::test::t2

                Failure[..]
                Tests: 0 passed, 2 failed, 0 skipped, 0 ignored, 0 filtered out

                Latest block number = [..] for url = http://127.0.0.1:3030?url=http://188.34.188.184:9545/rpc/v0_5
                Latest block number = [..] for url = http://188.34.188.184:9545/rpc/v0_5

                Failures:
                    tests::test::t1
                    tests::test::t2
            "
        ),
    );
}
