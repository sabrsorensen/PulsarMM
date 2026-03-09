#!/usr/bin/env bash
set -euo pipefail

SCOPE="${FLATPAK_INSTALL_SCOPE:-user}"
REQUIRED_SDK="${REQUIRED_SDK:-org.gnome.Sdk//49}"
REQUIRED_PLATFORM="${REQUIRED_PLATFORM:-org.gnome.Platform//49}"
REMOTE_NAME="${FLATPAK_REMOTE_NAME:-flathub}"
REMOTE_URL="${FLATPAK_REMOTE_URL:-https://flathub.org/repo/flathub.flatpakrepo}"

case "$SCOPE" in
  user)
    scope_args=(--user)
    ;;
  system)
    scope_args=()
    ;;
  *)
    echo "Unsupported FLATPAK_INSTALL_SCOPE: $SCOPE" >&2
    exit 1
    ;;
esac

export DCONF_PROFILE="${DCONF_PROFILE:-/dev/null}"

flatpak "${scope_args[@]}" remote-add --if-not-exists "$REMOTE_NAME" "$REMOTE_URL"

if ! flatpak "${scope_args[@]}" info "$REQUIRED_SDK" >/dev/null 2>&1; then
  echo "Installing Flatpak runtime: $REQUIRED_SDK ($SCOPE scope)"
  flatpak "${scope_args[@]}" install -y "$REMOTE_NAME" "$REQUIRED_SDK"
fi

if ! flatpak "${scope_args[@]}" info "$REQUIRED_PLATFORM" >/dev/null 2>&1; then
  echo "Installing Flatpak runtime: $REQUIRED_PLATFORM ($SCOPE scope)"
  flatpak "${scope_args[@]}" install -y "$REMOTE_NAME" "$REQUIRED_PLATFORM"
fi
