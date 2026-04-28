#!/usr/bin/env bash
# Bump the cookbook SHA pinned in src/commands/forge.rs.
#
# Usage:
#   scripts/bump-cookbook.sh           # bump to current main HEAD
#   scripts/bump-cookbook.sh <ref>     # bump to a branch / tag / full SHA
#
# Does NOT commit. Prints a compare URL so the diff between old and new
# cookbook content is one click away.

set -euo pipefail

REPO="steel-dev/steel-cookbook"
FORGE_FILE="src/commands/forge.rs"
REF="${1:-main}"

cd "$(git rev-parse --show-toplevel)"

if [[ ! -f "$FORGE_FILE" ]]; then
    echo "error: $FORGE_FILE not found" >&2
    exit 1
fi

# Resolve <ref> via remote refs. If REF is already a 40-char SHA,
# ls-remote returns nothing — use it directly.
NEW_SHA="$(git ls-remote "https://github.com/$REPO.git" "$REF" 2>/dev/null | awk '{print $1; exit}')"
if [[ -z "$NEW_SHA" ]]; then
    if [[ "$REF" =~ ^[a-f0-9]{40}$ ]]; then
        NEW_SHA="$REF"
    else
        echo "error: could not resolve '$REF' to a SHA in $REPO" >&2
        exit 1
    fi
fi

OLD_SHA="$(awk -F'"' '/^const COOKBOOK_REF: &str =/ {print $2; exit}' "$FORGE_FILE")"

if [[ -z "$OLD_SHA" ]]; then
    echo "error: could not find COOKBOOK_REF declaration in $FORGE_FILE" >&2
    exit 1
fi

if [[ "$OLD_SHA" == "$NEW_SHA" ]]; then
    echo "Already at $NEW_SHA. Nothing to do."
    exit 0
fi

# Anchor the substitution to the const line so no other occurrence
# of OLD_SHA elsewhere in the file can be hit.
TMP="$(mktemp)"
trap 'rm -f "$TMP"' EXIT
sed "/^const COOKBOOK_REF/ s/$OLD_SHA/$NEW_SHA/" "$FORGE_FILE" > "$TMP"
mv "$TMP" "$FORGE_FILE"

echo "Bumped $REPO:"
echo "  old: $OLD_SHA"
echo "  new: $NEW_SHA"
echo
echo "Compare: https://github.com/$REPO/compare/$OLD_SHA...$NEW_SHA"
echo
echo "Next: cargo test --lib forge && git diff $FORGE_FILE"
