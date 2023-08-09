struct File {
    path: felt252
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

}

fn parse_json(file: @File) -> Array<felt252> {

}
