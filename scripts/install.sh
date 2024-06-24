#!/usr/bin/env sh
set -e

echo Installing snfoundryup...

LOCAL_BIN="${HOME}/.local/bin"

SNFOUNDRYUP_URL="https://raw.githubusercontent.com/foundry-rs/starknet-foundry/master/scripts/snfoundryup"
SNFOUNDRYUP_PATH="${LOCAL_BIN}/snfoundryup"

# Check for curl
if ! command -v curl > /dev/null 2>&1; then
    echo "curl could not be found, please install it first."
    exit 1
fi


# Create the ${HOME}/.local/bin bin directory and snfoundryup binary if it doesn't exist.
mkdir -p "${LOCAL_BIN}"
curl -# -L "${SNFOUNDRYUP_URL}" -o "${SNFOUNDRYUP_PATH}"
chmod +x "${SNFOUNDRYUP_PATH}"


# Store the correct profile file (i.e. .profile for bash or .zshenv for ZSH).
case $SHELL in
*/zsh)
    PROFILE=${ZDOTDIR-"$HOME"}/.zshenv
    PREF_SHELL=zsh
    ;;
*/bash)
    PROFILE=$HOME/.bashrc
    PREF_SHELL=bash
    ;;
*/fish)
    PROFILE=$HOME/.config/fish/config.fish
    PREF_SHELL=fish
    ;;
*/ash)
    PROFILE=$HOME/.profile
    PREF_SHELL=ash
    ;;
*)
    echo "snfoundryup: could not detect shell, manually add ${LOCAL_BIN} to your PATH."
    exit 0
esac

# Only add snfoundryup if it isn't already in PATH.
case ":$PATH:" in
    *":${LOCAL_BIN}:"*)
        # The path is already in PATH, do nothing
        ;;
    *)
        # Add the snfoundryup directory to the path
        echo >> "$PROFILE" && echo "export PATH=\"\$PATH:$LOCAL_BIN\"" >> "$PROFILE"
        ;;
esac

printf "\nDetected your preferred shell is %s and added snfoundryup to PATH. Run 'source %s' or start a new terminal session to use snfoundryup.\n" "$PREF_SHELL" "$PROFILE"
printf "Then, simply run 'snfoundryup' to install Starknet-Foundry.\n"
