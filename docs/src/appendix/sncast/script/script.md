# `script`
Compile and run a cairo deployment script or initialize a script template by using subcommand [`init`](./init.md)

## `<MODULE_NAME>`
Required.

Script module name that contains the 'main' function that will be executed.

## `--package <NAME>`
Optional.

Name of the package that should be used.

If supplied, a script from this package will be used. Required if more than one package exists in a workspace.
