# Oracles

> ❗ **Warning**
>
> Oracles are an experimental feature.
> To enable them, you must pass the `--experimental-oracles` flag to `snforge`.

An [oracle][oracle docs] is an external process (like a script, binary, or web service)
that exposes custom logic or data to a Cairo program at runtime. You use it to perform tasks the Cairo VM can't, such as
accessing real-world data or executing complex, non-provable computations.

Starknet Foundry supports oracles in `snforge` tests. This feature allows your tests to interact with the outside world
in ways that aren't possible in the standard Starknet execution environment or with `snforge_std` cheatcodes.

## Using oracles

The [`oracle`][oracle library] library provides a type-safe interface for interacting with external
oracles in Cairo applications. Invoking oracles via this package is the recommended way, as it provides a well-tested,
secure, and maintainable interface for oracle interactions.

The [documentation][oracle docs] for this package provides a bird's-eye overview,
guidelines, and instructions on how to invoke oracles from Cairo code.

In the [oracle repository][oracle repo] there is also an end-to-end example
showcasing the feature. It implements a simple Cairo executable script that invokes an oracle written in Rust that
runs as a child process.

## Runtime

`snforge` ships with the same oracle runtime used by [`scarb execute`][oracles in scarb]. The bundled version is fixed
at the time of a Starknet Foundry release and may differ from the one used by your currently running Scarb version. For
details on behaviour, protocol, and configuration, see Scarb’s documentation.

[oracle library]: https://scarbs.xyz/packages/oracle

[oracle docs]: https://docs.swmansion.com/cairo-oracle

[oracle repo]: https://github.com/software-mansion/cairo-oracle

[oracles in scarb]: https://docs.swmansion.com/scarb/docs/extensions/oracles/overview.html
