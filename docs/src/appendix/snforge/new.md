# `snforge new`

Create a new StarkNet Foundry project at the provided path that should either be empty or not exist.

## `<PATH>`

Path to a location where the new project will be created.

## `-n`, `--name`

Name of a package to create, defaults to the directory name.

## `-t`, `--template`

Name of a template to use when creating a new project. Possible values:
- `balance-contract` (default): Basic contract with example tests.
- `cairo-program`: Simple Cairo program with unit tests.

## `--no-vcs`

Do not initialize a new Git repository.

## `--overwrite`

Try to create the project even if the specified directory is not empty, which can result in overwriting existing files

## `-h`, `--help`

Print help.
