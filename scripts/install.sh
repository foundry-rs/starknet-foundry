#!/usr/bin/env bash
set -e

echo Installing snfoundryup...


XDG_DATA_HOME="${XDG_DATA_HOME:-"${HOME}/.local/share"}"
INSTALL_ROOT="${XDG_DATA_HOME}/starknet-foundry-install"
LOCAL_BIN="${HOME}/.local/bin"

SNFOUNDRYUP_URL="https://raw.githubusercontent.com/partychad/starknet-foundry/fork-master/scripts/snfoundryup"
SNFOUNDRYUP_PATH="${LOCAL_BIN}/snfoundryup"

# Check for curl
if ! command -v curl &> /dev/null; then
    echo "curl could not be found, please install it first."
    exit 1
fi

# Create the .foundry bin directory and foundryup binary if it doesn't exist.
mkdir -p ${LOCAL_BIN}
curl -# -L ${SNFOUNDRYUP_URL} -o ${SNFOUNDRYUP_PATH}
chmod +x ${SNFOUNDRYUP_PATH}

# Create the man directory for future man files if it doesn't exist.

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
    exit 1
esac

# Only add foundryup if it isn't already in PATH.
if [[ ":$PATH:" != *":${LOCAL_BIN}:"* ]]; then
    # Add the foundryup directory to the path and ensure the old PATH variables remain.
    echo >> $PROFILE && echo "export PATH=\"\$PATH:$LOCAL_BIN\"" >> $PROFILE
fi


echo && echo "Detected your preferred shell is ${PREF_SHELL} and added snfoundryup to PATH. Run 'source ${PROFILE}' or start a new terminal session to use foundryup."
echo "Then, simply run 'snfoundryup' to install Starknet-Foundry."
