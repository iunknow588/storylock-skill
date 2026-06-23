#!/usr/bin/env bash
set +e

repo_root="${1:-}"
if [ -z "$repo_root" ]; then
  repo_root="$(pwd)"
fi

echo "WSL_DISTRO=$(cat /etc/os-release 2>/dev/null | sed -n 's/^PRETTY_NAME=//p' | tr -d '"' | head -n 1)"
echo "WSL_KERNEL=$(uname -a 2>/dev/null)"
echo "NODE_DEFAULT=$(node -v 2>/dev/null || echo missing)"

if [ -s "$HOME/.nvm/nvm.sh" ]; then
  # shellcheck disable=SC1091
  . "$HOME/.nvm/nvm.sh"
  best_node_version=$(nvm ls --no-colors 2>/dev/null | sed -n 's/.*v\([0-9][0-9]*\.[0-9][0-9]*\.[0-9][0-9]*\).*/\1/p' | awk -F. '$1 >= 22 { print $0 }' | sort -V | tail -n 1)
  if [ -n "$best_node_version" ]; then
    nvm use "v$best_node_version" >/dev/null 2>&1 || true
  fi
fi

echo "NODE_SELECTED=$(node -v 2>/dev/null || echo missing)"
echo "SECRET_TOOL=$(command -v secret-tool 2>/dev/null || echo missing)"
echo "DBUS_RUN_SESSION=$(command -v dbus-run-session 2>/dev/null || echo missing)"
echo "GNOME_KEYRING_DAEMON=$(command -v gnome-keyring-daemon 2>/dev/null || echo missing)"

if [ -d "$repo_root" ]; then
  cd "$repo_root" || exit 10
  node src/skills/local-story-access/scripts/check-secret-store.mjs 2>&1
  echo "CHECK_SECRET_STORE_EXIT=$?"
else
  echo "CHECK_SECRET_STORE_EXIT=10"
  echo "Repository path is not mounted in WSL: $repo_root"
fi
