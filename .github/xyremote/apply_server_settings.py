# XYRemote: point the client at our own relay/ID server before building.
#
# The server address and public key live in the libs/hbb_common submodule,
# which tracks upstream rustdesk/hbb_common, so they are applied here at build
# time instead of being committed there. Fails the build if anything does not
# match exactly once, so a stock-RustDesk binary can never be produced silently.
#
# Usage: python3 .github/xyremote/apply_server_settings.py [--app-name]
import re
import sys

CONFIG = "libs/hbb_common/src/config.rs"
SERVER = "14.51.2.222"
PUB_KEY = "aVwXc5GC7rwTX3uqSrpw0LHbpyo5zjUE6Na8Asuj7zs="

subs = [
    (r'pub const RENDEZVOUS_SERVERS: &\[&str\] = &\[[^\]]*\];',
     f'pub const RENDEZVOUS_SERVERS: &[&str] = &["{SERVER}"];'),
    (r'pub const RS_PUB_KEY: &str = "[^"]*";',
     f'pub const RS_PUB_KEY: &str = "{PUB_KEY}";'),
]
if "--app-name" in sys.argv:
    subs.append((r'pub static ref APP_NAME: RwLock<String> = RwLock::new\("[^"]*"\.to_owned\(\)\);',
                 'pub static ref APP_NAME: RwLock<String> = RwLock::new("XYRemote".to_owned());'))

with open(CONFIG, encoding="utf-8", newline="") as f:
    text = f.read()

for pattern, replacement in subs:
    text, count = re.subn(pattern, lambda _m, r=replacement: r, text)
    if count != 1:
        sys.exit(f"XYRemote setting not applied ({count} matches): {pattern}")

with open(CONFIG, "w", encoding="utf-8", newline="") as f:
    f.write(text)

print(f"XYRemote settings applied: server={SERVER}, app_name={'--app-name' in sys.argv}")
