use starknet::testing::cheatcode;
use super::super::byte_array::byte_array_as_felt_array;
use super::super::_cheatcode::handle_cheatcode;

#[derive(Drop, Clone)]
struct File {
    path: ByteArray // relative path
}

trait FileTrait {
    fn new(path: ByteArray) -> File;
}

impl FileTraitImpl of FileTrait {
    fn new(path: ByteArray) -> File {
        File { path }
    }
}

fn read_txt(file: @File) -> Array<felt252> {
    let content = handle_cheatcode(
        cheatcode::<'read_txt'>(byte_array_as_felt_array(file.path).span())
    );

    let mut result = array![];

    let mut i = 0;
    loop {
        if content.len() == i {
            break ();
        }
        result.append(*content[i]);
        i += 1;
    };
    result
}

fn read_json(file: @File) -> Array<felt252> {
    let content = handle_cheatcode(
        cheatcode::<'read_json'>(byte_array_as_felt_array(file.path).span())
    );

    let mut result = array![];

    let mut i = 0;
    loop {
        if content.len() == i {
            break ();
        }
        result.append(*content[i]);
        i += 1;
    };
    result
}

trait FileParser<T, impl TSerde: Serde<T>> {
    fn parse_txt(file: @File) -> Option<T>;
    fn parse_json(file: @File) -> Option<T>;
}

impl FileParserImpl<T, impl TSerde: Serde<T>> of FileParser<T> {
    fn parse_txt(file: @File) -> Option<T> {
        let mut content = handle_cheatcode(
            cheatcode::<'read_txt'>(byte_array_as_felt_array(file.path).span())
        );
        Serde::<T>::deserialize(ref content)
    }
    fn parse_json(file: @File) -> Option<T> {
        let mut content = handle_cheatcode(
            cheatcode::<'read_json'>(byte_array_as_felt_array(file.path).span())
        );
        Serde::<T>::deserialize(ref content)
    }
}
