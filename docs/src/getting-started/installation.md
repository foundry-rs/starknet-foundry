# Installation

Starknet Foundry is easy to install on Linux, macOS and WSL.
In this section, we will walk through the process of installing Starknet Foundry.

## Contents

<!-- TOC -->

* [Installation](#installation)
    * [Contents](#contents)
    * [With Starkup](#with-starkup)
    * [Manual Installation](#manual-installation)
      * [Requirements](#requirements)
      * [Linux and macOS](#linux-and-macos)
          * [Install asdf](#install-asdf)
          * [Install Scarb version >= 2.8.5](#install-scarb-version--285)
          * [(Optional for Scarb >= 2.10.0) Rust Installation](#optional-for-scarb--21001-rust-installation)
          * [Install Starknet Foundry](#install-starknet-foundry)
      * [Windows](#windows)
    * [Common Errors](#common-errors)
        * [No Version Set (Linux and macOS Only)](#no-version-set-linux-and-macos-only)
        * [Invalid Rust Version](#invalid-rust-version)
            * [Linux and macOS](#linux-and-macos-1)
        * [`scarb test` Isn’t Running `snforge`](#scarb-test-isnt-running-snforge)
    * [Shell completions (Optional)](#set-up-shell-completions-optional)
    * [Universal-Sierra-Compiler update](#universal-sierra-compiler-update)
        * [Linux and macOS](#linux-and-macos-2)
    * [How to build Starknet Foundry from source code](#how-to-build-starknet-foundry-from-source-code)
* [Uninstallation](#uninstallation)

<!-- TOC -->

## With Starkup

[starkup](https://github.com/software-mansion/starkup) helps you install all the tools used to develop packages in Cairo and write contracts for Starknet, including Starknet Foundry.

Run the following in your terminal, then follow the onscreen instructions:

```shell
curl --proto '=https' --tlsv1.2 -sSf https://sh.starkup.sh | sh
```

To verify that Starknet Foundry was installed, open a new terminal and run

```shell
snforge --version
```

## Manual Installation

### Requirements

> 📝 **Note**
>
> Ensure all requirements are installed and follow the required minimal versions.
> Starknet Foundry will not run if not following these requirements.

To use Starknet Foundry, you need:

- [Scarb](https://docs.swmansion.com/scarb/download.html) version >= 2.8.5
- [Universal-Sierra-Compiler](https://github.com/software-mansion/universal-sierra-compiler)
- _(Optional for Scarb >= 2.10.0)_[^note] [Rust](https://www.rust-lang.org/tools/install) version >= 1.80.1

all installed and added to your `PATH` environment variable.

[^note]: Additionally, your platform must be one of the supported:
* `aarch64-apple-darwin`
* `aarch64-unknown-linux-gnu`
* `x86_64-apple-darwin`
* `x86_64-unknown-linux-gnu`

> 📝 **Note**
>
> `Universal-Sierra-Compiler` will be automatically installed if you use `asdf` or `snfoundryup`.
> You can also create `UNIVERSAL_SIERRA_COMPILER` env var to make it visible for `snforge`.

### Linux and macOS

> ℹ️ **Info**
>
> If you already have installed Rust, Scarb and asdf simply run
> `asdf plugin add starknet-foundry`

#### Install asdf

Follow the instructions from [asdf docs](https://asdf-vm.com/guide/getting-started.html#getting-started).

To verify that asdf was installed, run

```shell
asdf --version
```

#### Install Scarb version >= 2.8.5

First, add Scarb plugin to asdf

```shell
asdf plugin add scarb
```

Install Scarb

```shell
asdf install scarb latest
```

Set a version globally (in your ~/.tool-versions file):

```shell
asdf set --home scarb latest
```

To verify that Scarb was installed, run

```shell
scarb --version
```

and verify that version is >= 2.8.5

#### (Optional for Scarb >= 2.10.0)[^note] Rust Installation

> ℹ️️ **Info**
>
> Rust installation is only required if **ANY** of the following is true:
>
> * You are using Scarb version <= 2.10.0
> * Your platform is not one of the following supported platforms:
>   * `aarch64-apple-darwin`
>   * `aarch64-unknown-linux-gnu`
>   * `x86_64-apple-darwin`
>   * `x86_64-unknown-linux-gnu`

```shell
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

To verify that correct Rust version was installed, run

```shell
rustc --version
```

and verify that version is >= 1.80.1

See [Rust docs](https://doc.rust-lang.org/beta/book/ch01-01-installation.html#installation) for more details.

#### Install Starknet Foundry

First, add Starknet Foundry plugin to asdf

```shell
asdf plugin add starknet-foundry
```

Install Starknet Foundry

```shell
asdf install starknet-foundry latest
```

Set a version globally (in your ~/.tool-versions file):

```shell
asdf set --home starknet-foundry latest
```

To verify that Starknet Foundry was installed, run

```shell
snforge --version
```

or

```shell
sncast --version
```

### Windows

> 🐧 **Info** - WSL (Windows Subsystem for Linux)
>
> Starknet Foundry can be installed on Windows using [WSL](https://learn.microsoft.com/en-us/windows/wsl/install).
>
> Please follow the [Linux and macOS](#linux-and-macos) guide within your WSL environment.

## Common Errors

### No Version Set

Users may encounter this error when trying to use `snforge` or `sncast` without setting a version:

```shell
No version is set for command snforge
Consider adding one of the following versions in your config file at $HOME/.tool_versions
starknet-foundry 0.37.0
```

This error indicates that `Starknet Foundry` version is unset. To resolve it, set the version globally using asdf:

```shell
asdf set --home starknet-foundry latest
```

For additional information on asdf version management, see
the [asdf](https://asdf-vm.com/guide/getting-started.html#_6-set-a-version)

### Invalid Rust Version

When running any `snforge` command, error similar to this is displayed

```shell
Compiling snforge_scarb_plugin v0.34.0
error: package snforge_scarb_plugin v0.34.0 cannot be built because it requires rustc 1.80.1 or newer, while the currently active rustc version is 1.76.0
```

This indicates incorrect Rust version is installed or set.

Verify if rust version >= 1.80.1 is installed

```shell
rustc --version
1.80.1
```

To fix, follow the platform specific instructions:

If the version is incorrect or the error persists, try changing the global version of Rust

```shell
rustup default stable
```

and local version of Rust

```shell
rustup override set stable
```

### `scarb test` Isn’t Running `snforge`

By default, `scarb test` doesn't use `snforge` to run tests, and it needs to be configured.
Make sure to include this section in `Scarb.toml`

```toml
[scripts]
test = "snforge test"
```

## Set up shell completions (optional)

Shell completions allow your terminal to suggest and automatically complete commands and options when you press `Tab`.

<details>
  <summary><strong>Bash</strong></summary>

Add the following to `~/.bashrc` or `~/.bash_profile` (macOS):

```bash
# BEGIN FOUNDRY COMPLETIONS
_snforge() {
  if ! snforge completions bash >/dev/null 2>&1; then
    return 0
  fi
  source <(snforge completions bash)
  _snforge "$@"
}

_sncast() {
  if ! sncast completions bash >/dev/null 2>&1; then
    return 0
  fi
  source <(sncast completions bash)
  _sncast "$@"
}

complete -o default -F _snforge snforge
complete -o default -F _sncast sncast
# END FOUNDRY COMPLETIONS
```

Run `source ~/.bashrc` (or `source ~/.bash_profile`), or open a new terminal session to apply the changes.

</details>

<details>
  <summary><strong>ZSH</strong></summary>

Add the following to `~/.zshrc`:

```bash
# BEGIN FOUNDRY COMPLETIONS
_snforge() {
  if ! snforge completions zsh >/dev/null 2>&1; then
    return 0
  fi
  eval "$(snforge completions zsh)"
  _snforge "$@"
}

_sncast() {
  if ! sncast completions zsh >/dev/null 2>&1; then
    return 0
  fi
  eval "$(sncast completions zsh)"
  _sncast "$@"
}

autoload -Uz compinit && compinit
compdef _snforge snforge
compdef _sncast sncast
# END FOUNDRY COMPLETIONS
```

> 📝 **Note**
>
> If you already have `autoload -Uz compinit && compinit` in your `~/.zshrc` (for example, from another completions such as `scarb`), do not add it again. Only one call is needed.

Run `source ~/.zshrc`, or open a new terminal session to apply the changes.

For more information about Zsh completions, see the [Zsh documentation](https://zsh.sourceforge.io/Doc/Release/Completion-System.html) or the [Arch Wiki](https://wiki.archlinux.org/title/Zsh#Command_completion).

</details>

<details>
  <summary><strong>Fish</strong></summary>

Add the following to `~/.config/fish/config.fish`:

```bash
# BEGIN FOUNDRY COMPLETIONS
function _snforge
  if not snforge completions fish >/dev/null 2>&1
    return 0
  end
  source (snforge completions fish | psub)
  complete -C (commandline -cp)
end

function _sncast
  if not sncast completions fish >/dev/null 2>&1
    return 0
  end
  source (sncast completions fish | psub)
  complete -C (commandline -cp)
end

complete -c snforge -f -a '(_snforge)'
complete -c sncast -f -a '(_sncast)'
# END FOUNDRY COMPLETIONS
```

Run `source ~/.config/fish/config.fish`, or open a new terminal session to apply the changes.

</details>

<details>
  <summary><strong>Elvish</strong></summary>

Add the following to your `~/.config/elvish/rc.elv` file:

```bash
# BEGIN FOUNDRY COMPLETIONS
try {
  eval (snforge completions elvish | slurp)
} catch { return }

try {
  eval (sncast completions elvish | slurp)
} catch { return }
# END FOUNDRY COMPLETIONS
```

Run `eval (slurp <  ~/.config/elvish/rc.elv)`, or open a new terminal session to apply the changes.

</details>

## Universal-Sierra-Compiler update

If you would like to bump the USC manually (e.g. when the new Sierra version is released) you can do it by running:

```shell
curl -L https://raw.githubusercontent.com/software-mansion/universal-sierra-compiler/master/scripts/install.sh | sh
```

## How to build Starknet Foundry from source code

If you are unable to install Starknet Foundry using the instructions above, you can try building it from
the [source code](https://github.com/foundry-rs/starknet-foundry) as follows:

1. [Set up a development environment.](../development/environment-setup.md)
2. Run `cd starknet-foundry && cargo build --release`. This will create a `target` directory.
3. Move the `target` directory to the desired location (e.g. `~/.starknet-foundry`).
4. Add `DESIRED_LOCATION/target/release/` to your `PATH`.

# Uninstallation

## Remove the Starknet Foundry Plugin

Follow the official asdf documentation to remove the Starknet Foundry plugin:

```bash
asdf plugin remove starknet-foundry
```

For more details, refer to the [asdf plugin documentation](https://asdf-vm.com/manage/plugins.html#remove).

## Verify Uninstallation

To confirm Starknet Foundry has been completely removed, run:

```bash
snforge --version
```

If the uninstallation was successful, you should see `command not found: snforge`
