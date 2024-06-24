# `var`

> `fn var(name: ByteArray) -> Array<felt252>`

Reads an environment variable, without parsing it.

The serialized output is correlated with the inferred input type, same as
during [reading from a file](../fs/read_txt.md#accepted-format).

> 📝 **Note**
>
> If you want snfoundry to treat your variable like a short string, surround it with 'single quotes'.
>
> If you would like it to be serialized as a `ByteArray`, use "double quoting". It will be then de-serializable
> with `Serde`.


