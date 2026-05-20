#!/bin/bash

# Set version in all project files.
#
# Usage: ./scripts/set_version.sh <version>
# Example: ./scripts/set_version.sh 1.2.0
#
# This script must be run from the project root directory.
# It updates version in:
#   - endpoint/Cargo.toml
#   - Cargo.lock
#   - scripts/install.sh

set -e

VERSION=$1

if [ -z "$VERSION" ]; then
    echo "Usage: $0 <version>"
    echo "Example: $0 1.2.0"
    exit 1
fi

if ! [[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "Error: Invalid version format. Expected X.Y.Z (e.g., 1.2.0)"
    exit 1
fi

echo "Setting version to $VERSION"

# endpoint/Cargo.toml
cargo_toml="endpoint/Cargo.toml"
OLD_VERSION=$(grep '^version = ' "$cargo_toml" | head -1 | sed -e 's/version = "\(.*\)"/\1/')
sed -i -e "s/^version = \"${OLD_VERSION}\"/version = \"${VERSION}\"/" "$cargo_toml"
echo "Updated ${cargo_toml}"

# Cargo.lock — update trusttunnel_endpoint package version
python3 -c "
with open('Cargo.lock', 'r', encoding='UTF-8') as file:
    content = file.read()

output = ''
is_in_endpoint_section = False
for line in content.splitlines():
    if line == 'name = \"trusttunnel_endpoint\"':
        is_in_endpoint_section = True
    elif is_in_endpoint_section and line == '[[package]]':
        is_in_endpoint_section = False

    if is_in_endpoint_section and line == 'version = \"${OLD_VERSION}\"':
        output += 'version = \"${VERSION}\"\n'
        continue

    output += line + '\n'

with open('Cargo.lock', 'w') as file:
    file.write(output)
"
echo "Updated Cargo.lock"

# scripts/install.sh
install_sh="scripts/install.sh"
if [ -f "$install_sh" ]; then
    sed -i -e "s/^version='[0-9\.]*'$/version='${VERSION}'/" "$install_sh"
    echo "Updated ${install_sh}"
fi

echo "Version set to $VERSION in all project files."
