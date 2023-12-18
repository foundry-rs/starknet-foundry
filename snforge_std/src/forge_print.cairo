use starknet::testing::cheatcode;

trait PrintTrait<T> {
    fn print(self: @T);
}

impl GenericPrintImpl<T, impl TInto: Into<T, felt252>, impl TCopy: Copy<T>> of PrintTrait<T> {
    fn print(self: @T) {
        cheatcode::<'print'>(array![(*self).into()].span());
    }
}

impl U256PrintImpl of PrintTrait<u256> {
    fn print(self: @u256) {
        Into::<_, felt252>::into(*self.low).print();
        Into::<_, felt252>::into(*self.high).print();
    }
}

impl ArrayGenericPrintImpl<T, impl TPrint: PrintTrait<T>> of PrintTrait<Array<T>> {
    fn print(self: @Array<T>) {
        let mut i = 0;
        loop {
            if i == self.len() {
                break;
            }
            self[i].print();
            i += 1;
        };
    }
}
