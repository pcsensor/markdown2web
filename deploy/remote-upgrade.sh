#!/bin/bash
# Upgrade markdown2web on the server (scheme A).
# Preserves /opt/markdown2web/{env,content,data}; replaces binary + static.
# Run as root after: cargo build --release in $REPO
set -euo pipefail

REPO=/home/justin/pcsensor/markdown2web
OPT=/opt/markdown2web
TS=$(date +%Y%m%d-%H%M%S)
BACKUP_ROOT=/root/markdown2web-backups

echo "==> backup $TS"
mkdir -p "$BACKUP_ROOT"
cp -a "$OPT/markdown2web" "$BACKUP_ROOT/markdown2web.$TS" 2>/dev/null || true
cp -a "$OPT/env" "$BACKUP_ROOT/env.$TS" 2>/dev/null || true
# content + data are the durable state; generated can be rebuilt
tar czf "$BACKUP_ROOT/state.$TS.tgz" -C "$OPT" content data env 2>/dev/null || \
  tar czf "$BACKUP_ROOT/state.$TS.tgz" -C "$OPT" content data

if [[ ! -x "$REPO/target/release/markdown2web" ]]; then
  echo "missing binary: $REPO/target/release/markdown2web"
  echo "run: cd $REPO && cargo build --release"
  exit 1
fi

echo "==> stop service"
systemctl stop markdown2web

echo "==> install binary + static"
install -o m2w -g m2w -m 550 \
  "$REPO/target/release/markdown2web" "$OPT/markdown2web"
rsync -a --delete "$REPO/static/" "$OPT/static/"
chown -R m2w:m2w "$OPT/static"
# Do NOT rsync content/ — production notes live under /opt and are managed via /admin

echo "==> refresh systemd unit"
cp "$REPO/deploy/markdown2web.service" /etc/systemd/system/markdown2web.service
systemctl daemon-reload

echo "==> start service"
systemctl start markdown2web
sleep 2
systemctl is-active markdown2web
echo "==> journal"
journalctl -u markdown2web -n 40 --no-pager

echo "==> local smoke"
curl -sS -o /dev/null -w "home:%{http_code}\n" http://127.0.0.1:3000/
curl -sS -o /dev/null -w "health:%{http_code}\n" http://127.0.0.1:3000/health
curl -sS -o /dev/null -w "admin:%{http_code}\n" http://127.0.0.1:3000/admin
echo "==> done (backup under $BACKUP_ROOT)"
