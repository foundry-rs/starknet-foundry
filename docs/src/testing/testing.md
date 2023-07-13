# Testing
Forge lets you test standalone Cairo functions. This technique is referred to as unit testing. You should write as many unit tests as possible as these are faster than integration tests.

## Writing your first test

First, add the following code to the src/lib.cairo file:

```
fn sum(a: felt252, b: felt252) -> felt252 {
    return a + b;
}

#[test]
fn test_sum() {
    assert(sum(2, 3) == 5, 'sum incorrect');
}
```

It is good to keep your unit tests close to the tested code, in the source file. If you would like to you could put tests in a separate files. Create a file tests/test_sum.cairo:

Now run forge using command:
```
forge
```

## Test collecting

Forge considers as test all functions in your package with #[test] attribute.

Test cases cannot return any values and cannot take any arguments. // TODO

## Failing tests

Your tests fail when code panics. To write a test that fails, you will need to use panic function, here's how you do it:

```
use array::ArrayTrait;

// Single value in the panic payload
#[test]
fn test_panic_single_value() {
    let mut data = ArrayTrait::new();
    data.append('this one should fail');
    panic(data)
}
```

Of course, if any of the functions you call from tests panics, your test will fail as well.