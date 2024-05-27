# `byte_array` Module

Module containing utilities for manipulating `ByteArray`s.

## Functions

> `fn try_deserialize_bytearray_error(x: Span<felt252>) -> Result<ByteArray, ByteArray>` 

This function is meant to transform a serialized output from a contract call into a `ByteArray`.
Returns the parsed `ByteArray`, or an `Err` with reason, if the parsing failed.