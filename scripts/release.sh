#!/bin/bash

VERSION=$1

if [ -z "$VERSION" ]; then
    echo "Error: Version argument is required"
    echo "Usage: $0 <version>"
    exit 1
fi

sed -i.bak "s/## \[Unreleased\]/## \[Unreleased\]\n\n## \[${VERSION}\] - $(TZ=Europe/Krakow date '+%Y-%m-%d')/" CHANGELOG.md
rm CHANGELOG.md.bak 2> /dev/null

sed -i.bak "/\[workspace.package\]/,/version =/ s/version = \".*/version = \"${VERSION}\"/" Cargo.toml
rm Cargo.toml.bak 2> /dev/null

declare -a scarb_files=(
    "sncast_std/Scarb.toml"
    "snforge_std/Scarb.toml"
    "crates/snforge-scarb-plugin/Scarb.toml"
)

for file in "${scarb_files[@]}"; do
    sed -i.bak "/\[package\]/,/version =/ s/version = \".*/version = \"${VERSION}\"/" "$file"
    rm "${file}.bak" 2> /dev/null
done

scarb --manifest-path sncast_std/Scarb.toml build
scarb --manifest-path snforge_std/Scarb.toml build

cargo update -p forge
cargo update -p sncast

declare -a doc_files=(
    "./docs/src/getting-started/first-steps.md"
    "./docs/src/starknet/script.md"
    "./docs/src/appendix/scarb-toml.md"
    "./docs/src/appendix/cheatcodes.md"
    "./docs/src/appendix/snforge-library.md"
    "./docs/src/testing/contracts.md"
    "./docs/src/testing/using-cheatcodes.md"
)

for file in "${doc_files[@]}"; do
    # Use different sed patterns based on the file
    case "$file" in
        "./docs/src/appendix/scarb-toml.md")
            # Specifically target snforge_std and sncast_std versions
            sed -i.bak -E '
                /snforge_std = \{/,/version = / s/(version = ")[0-9]+\.[0-9]+\.[0-9]+")/\1'"${VERSION}"'"/;
                /sncast_std = \{/,/version = / s/(version = ")[0-9]+\.[0-9]+\.[0-9]+")/\1'"${VERSION}"'"/
            ' "$file"
            ;;
        "./docs/src/getting-started/first-steps.md")
            # Target lines containing snforge_std or sncast_std version specifications
            sed -i.bak -E '
                /snforge_std = \{/,/version = / s/(version = ")[0-9]+\.[0-9]+\.[0-9]+")/\1'"${VERSION}"'"/;
                /sncast_std = \{/,/version = / s/(version = ")[0-9]+\.[0-9]+\.[0-9]+")/\1'"${VERSION}"'"/
            ' "$file"
            ;;
        "./docs/src/starknet/script.md")
            # Target sncast_std version specifications
            sed -i.bak -E '/sncast_std = \{/,/version = / s/(version = ")[0-9]+\.[0-9]+\.[0-9]+")/\1'"${VERSION}"'"/' "$file"
            ;;
        "./docs/src/testing/"*)
            # Target snforge_std version specifications in testing docs
            sed -i.bak -E '/snforge_std = \{/,/version = / s/(version = ")[0-9]+\.[0-9]+\.[0-9]+")/\1'"${VERSION}"'"/' "$file"
            ;;
        *)
            # For any other files, target both snforge_std and sncast_std
            sed -i.bak -E '
                /snforge_std = \{/,/version = / s/(version = ")[0-9]+\.[0-9]+\.[0-9]+")/\1'"${VERSION}"'"/;
                /sncast_std = \{/,/version = / s/(version = ")[0-9]+\.[0-9]+\.[0-9]+")/\1'"${VERSION}"'"/
            ' "$file"
            ;;
    esac
    rm "${file}.bak" 2> /dev/null || exit 1
done

echo "Version update complete!"