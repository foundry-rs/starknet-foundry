use array::ArrayTrait;
use array::SpanTrait;
use clone::Clone;
use option::OptionTrait;

use starknet::testing::cheatcode;

#[derive(Drop, Copy)]
struct File {
    path: felt252  // relative path to a file from a dir where snforge is executed
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

fn parse_json(file: @File) -> Array<felt252> {
    let content = cheatcode::<'parse_json'>(array![*file.path].span());

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
