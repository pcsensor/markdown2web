#!/bin/bash
# Production scheme A: binary + systemd + Caddy (automatic HTTPS)
# Run on the Linux server as root after: cargo build --release in $REPO
set -euo pipefail

REPO=/home/justin/pcsensor/markdown2web
OPT=/opt/markdown2web
# Change to your real domain before first deploy
DOMAIN="${M2W_DOMAIN:-notes.example.com}"

echo "==> 0. prerequisites"
command -v openssl >/dev/null
if [[ ! -x "$REPO/target/release/markdown2web" ]]; then
  echo "missing binary: $REPO/target/release/markdown2web"
  echo "run: cd $REPO && cargo build --release"
  exit 1
fi

echo "==> 1. user + dirs"
if ! id m2w >/dev/null 2>&1; then
  useradd --system --home "$OPT" --shell /usr/sbin/nologin m2w
fi
mkdir -p \
  "$OPT/content/notes" \
  "$OPT/content/assets" \
  "$OPT/generated/site/assets" \
  "$OPT/data" \
  "$OPT/static"
chown -R m2w:m2w "$OPT"

echo "==> 2. binary + static (+ seed content if empty)"
install -o m2w -g m2w -m 550 \
  "$REPO/target/release/markdown2web" "$OPT/markdown2web"
rsync -a --delete "$REPO/static/" "$OPT/static/"
# Seed notes/assets only when production content is empty (never clobber live data)
shopt -s nullglob
seed_notes=("$OPT/content/notes"/*.md)
shopt -u nullglob
if ((${#seed_notes[@]} == 0)); then
  echo "    seeding content/ from repo (first deploy)"
  rsync -a "$REPO/content/" "$OPT/content/"
fi
chown -R m2w:m2w "$OPT/static" "$OPT/content" "$OPT/generated" "$OPT/data"
chmod 750 "$OPT"

echo "==> 3. env (create only if missing — never overwrite secrets)"
if [[ ! -f "$OPT/env" ]]; then
  ADMIN_PASS="$(openssl rand -base64 24 | tr -d '/+=' | head -c 24)"
  cat > "$OPT/env" <<EOF
# markdown2web production env — managed on server, do not commit
# App listens on localhost; Caddy/Nginx terminates TLS

M2W_HOST=127.0.0.1
M2W_PORT=3000
M2W_BASE_URL=https://${DOMAIN}
M2W_SITE_NAME=markdown2web

M2W_CONTENT_DIR=/opt/markdown2web/content
M2W_GENERATED_DIR=/opt/markdown2web/generated/site
M2W_DATA_DIR=/opt/markdown2web/data

M2W_ADMIN_USERNAME=admin
M2W_ADMIN_PASSWORD=${ADMIN_PASS}

# Production: content managed via /admin; disable fs watcher
M2W_WATCH_ENABLED=false
M2W_TURNSTILE_ENABLED=false
# M2W_TURNSTILE_SITE_KEY=
# M2W_TURNSTILE_SECRET_KEY=

# Raise when uploading large videos
M2W_UPLOAD_LIMIT_MB=128

RUST_LOG=markdown2web=info,tower_http=info
EOF
  chown root:m2w "$OPT/env"
  chmod 640 "$OPT/env"
  echo "    wrote $OPT/env (admin password printed once below)"
  echo "    M2W_ADMIN_PASSWORD=${ADMIN_PASS}"
else
  echo "    keep existing $OPT/env"
fi

echo "==> 4. systemd"
cp "$REPO/deploy/markdown2web.service" /etc/systemd/system/markdown2web.service
systemctl daemon-reload
systemctl enable markdown2web

echo "==> 5. Caddy site block for ${DOMAIN}"
if [[ ! -f /etc/caddy/Caddyfile ]]; then
  echo "Caddyfile not found at /etc/caddy/Caddyfile — install Caddy first or merge deploy/Caddyfile.snippet manually"
else
  TS="$(date +%Y%m%d-%H%M%S)"
  cp -a /etc/caddy/Caddyfile "/etc/caddy/Caddyfile.bak.${TS}"
  # Idempotent: replace existing block for DOMAIN, or append
  python3 - <<PY
from pathlib import Path
import re

domain = ${DOMAIN@Q}
path = Path("/etc/caddy/Caddyfile")
text = path.read_text(encoding="utf-8")
block = f"""
# --- markdown2web ({domain}) ---
{domain} {{
	encode gzip

	request_body {{
		max_size 140MB
	}}

	header {{
		X-Frame-Options "DENY"
		X-Content-Type-Options "nosniff"
		Referrer-Policy "no-referrer"
		-Server
	}}

	reverse_proxy 127.0.0.1:3000 {{
		header_up Host {{host}}
		header_up X-Real-IP {{remote_host}}
		header_up X-Forwarded-For {{remote_host}}
		header_up X-Forwarded-Proto {{scheme}}
		transport http {{
			read_timeout 3600s
			write_timeout 3600s
		}}
	}}
}}
"""
# Remove previous markdown2web-managed block for this domain if present
pattern = re.compile(
    r"\n?# --- markdown2web \\(" + re.escape(domain) + r"\\) ---\\n"
    + re.escape(domain) + r" \\{.*?\\n\\}\\n?",
    re.DOTALL,
)
text = pattern.sub("\n", text)
text = text.rstrip() + "\n" + block
path.write_text(text, encoding="utf-8")
print("updated Caddyfile for", domain)
PY
  caddy validate --config /etc/caddy/Caddyfile
fi

echo "==> 6. start services"
systemctl restart markdown2web
sleep 2
systemctl is-active markdown2web
if systemctl is-active --quiet caddy 2>/dev/null || systemctl list-unit-files caddy.service &>/dev/null; then
  systemctl reload caddy || systemctl restart caddy
  sleep 1
  systemctl is-active caddy || true
fi

echo "==> journal markdown2web"
journalctl -u markdown2web -n 50 --no-pager

echo "==> local smoke"
curl -sS -o /dev/null -w "home:%{http_code}\n" http://127.0.0.1:3000/
curl -sS -o /dev/null -w "health:%{http_code}\n" http://127.0.0.1:3000/health
curl -sS -o /dev/null -w "admin:%{http_code}\n" http://127.0.0.1:3000/admin

echo "==> layout"
ls -la "$OPT"
ls -la "$OPT/static" | head -20

echo "==> done"
echo "    public URL: https://${DOMAIN}"
echo "    admin:      https://${DOMAIN}/admin"
echo "    env file:   $OPT/env (chmod 640, root:m2w)"
