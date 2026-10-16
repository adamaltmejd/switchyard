#!/bin/sh
# Only for disposable hosted Linux CI runners. Installs the native runtime.
set -eu
test "${GITHUB_ACTIONS:-}" = true
: "${RUNNER_TEMP:?}" "${GITHUB_ENV:?}"

sudo apt-get update
sudo apt-get install -y build-essential pkg-config libapparmor-dev libbtrfs-dev \
    libgpgme-dev libseccomp-dev libsqlite3-dev libsubid-dev libsystemd-dev \
    dbus-user-session

podman_source=$(mktemp -d "$RUNNER_TEMP/yard-podman.XXXXXX")
trap 'rm -rf "$podman_source"' EXIT
git clone --depth 1 --branch v6.1.4 https://github.com/containers/podman.git "$podman_source"
test "$(git -C "$podman_source" rev-parse HEAD)" = 464c15fec06f0074c916f9eff87c4806869c2833
# Linux release binaries are remote clients. Keep the native build's default
# toolchain and build tags, including seccomp, AppArmor and systemd support.
make -C "$podman_source" bin/podman PREFIX=/usr
sudo install -m 0755 "$podman_source/bin/podman" /usr/local/bin/podman
sudo install -D -m 0644 "$podman_source/vendor/go.podman.io/storage/storage.conf" \
    /usr/share/containers/storage.conf
test "$(podman --version)" = 'podman version 6.1.4'

if [ -e /proc/sys/kernel/apparmor_restrict_unprivileged_userns ]; then
    sudo sysctl -w kernel.apparmor_restrict_unprivileged_userns=0
fi
# Without a user session podman ignores --memory, and pinfold refuses every box.
sudo loginctl enable-linger "$(id -un)"
sudo systemctl start "user@$(id -u).service"
export XDG_RUNTIME_DIR="/run/user/$(id -u)"
export DBUS_SESSION_BUS_ADDRESS="unix:path=$XDG_RUNTIME_DIR/bus"
systemctl --user enable --now dbus.socket
podman system migrate
test "$(podman info --format '{{.Host.CgroupManager}}')" = systemd
echo "XDG_RUNTIME_DIR=$XDG_RUNTIME_DIR" >> "$GITHUB_ENV"
echo "DBUS_SESSION_BUS_ADDRESS=$DBUS_SESSION_BUS_ADDRESS" >> "$GITHUB_ENV"
