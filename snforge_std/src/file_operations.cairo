use array::ArrayTrait;
use array::SpanTrait;
use clone::Clone;
use option::OptionTrait;
use serde::Serde;

use starknet::testing::cheatcode;

#[derive(Drop, Copy)]
struct File {
    path: felt252  // relative path
}

trait FileTrait {
    fn new(path: felt252) -> File;
}

impl FileTraitImpl of FileTrait {
    fn new(path: felt252) -> File {
        File { path }
    }
}

fn parse_txt(file: @File) -> Array<felt252> {
    let content = cheatcode::<'parse_txt'>(array![*file.path].span());

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

// fn parse_json(file: @File) -> Array<felt252> {
//     let content = cheatcode::<'parse_json'>(array![*file.path].span());

//     let mut result = array![];

//     let mut i = 0;
//     loop {
//         if content.len() == i {
//             break ();
//         }
//         result.append(*content[i]);
//         i += 1;
//     };
//     result
// }

trait TxtParser<T, impl TSerde: Serde<T>> {
    fn deserialize_txt(file: @File) -> Option<T>;
}

// trait JsonParser<T, impl TSerde: Serde<T>> {
//     fn deserialize_json(file: @File) -> Option<T>;
// }

impl TxtParserImpl<T, impl TSerde: Serde<T>> of TxtParser<T> {
    fn deserialize_txt(file: @File) -> Option<T> {
        let mut content = cheatcode::<'parse_txt'>(array![*file.path].span());
        Serde::<T>::deserialize(ref content)
    }
}

// impl JsonParserImpl<T, impl TSerde: Serde<T>> of JsonParser<T> {
//     fn deserialize_json(file: @File) -> Option<T> {
//         let mut content = cheatcode::<'parse_json'>(array![*file.path].span());
//         Serde::<T>::deserialize(ref content)
//     }
// }
