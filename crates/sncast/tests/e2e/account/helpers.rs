use indoc::indoc;
use std::{fs::File, io::Write};
use tempfile::{TempDir, tempdir};

#[must_use]
pub fn create_tempdir_with_accounts_file(file_name: &str, with_sample_data: bool) -> TempDir {
    let dir = tempdir().expect("Unable to create a temporary directory");
    let location = dir.path().join(file_name);

    let mut file = File::create(location).expect("Unable to create a temporary accounts file");

    let data = if with_sample_data {
        indoc! {r#"
        {
            "alpha-sepolia": {
                "user0": {
                    "private_key": "0x1e9038bdc68ce1d27d54205256988e85",
                    "public_key": "0x2f91ed13f8f0f7d39b942c80bfcd3d0967809d99e0cc083606cbe59033d2b39",
                    "address": "0x4f5f24ceaae64434fa2bc2befd08976b51cf8f6a5d8257f7ec3616f61de263a",
                    "type": "open_zeppelin"
                },
                "user-1": {
                    "address": "0x9613a934141dd6625748a7e066a380b3f9787f079f35ecc2f3ba934d507d4e",
                    "class_hash": "0x36078334509b514626504edc9fb252328d1a240e4e948bef8d0c08dff45927f",
                    "private_key": "0x1c3495fce931c0b3ed244f55c54226441a8254deafbc7fab2e46926b4d2fdae",
                    "public_key": "0x63b3a3ac141e4c007b167b27450f110c729cc0d0238541ca705b0de5144edbd",
                    "salt": "0xe2b200bbdf76c31b",
                    "type": "argent"
                }
            },
            "custom-network": {
                "user3": {
                    "private_key": "0xe3e70682c2094cac629f6fbed82c07cd",
                    "public_key": "0x7e52885445756b313ea16849145363ccb73fb4ab0440dbac333cf9d13de82b9",
                    "address": "0x7e00d496e324876bbc8531f2d9a82bf154d1a04a50218ee74cdd372f75a551a"
                },
                "user4": {
                    "private_key": "0x73fbb3c1eff11167598455d0408f3932e42c678bd8f7fbc6028c716867cc01f",
                    "public_key": "0x43a74f86b7e204f1ba081636c9d4015e1f54f5bb03a4ae8741602a15ffbb182",
                    "salt": "0x54aa715a5cff30ccf7845ad4659eb1dac5b730c2541263c358c7e3a4c4a8064",
                    "address": "0x7ccdf182d27c7aaa2e733b94db4a3f7b28ff56336b34abf43c15e3a9edfbe91",
                    "deployed": true
                }
            }
        }
        "#}
    } else {
        r"{}"
    };

    file.write_all(data.as_bytes())
        .expect("Unable to write test data to a temporary file");

    file.flush().expect("Unable to flush a temporary file");

    dir
}
