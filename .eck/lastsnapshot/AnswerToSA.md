# Report: Nginx Fix for Photo Upload (413 Entity Too Large)

## Date: 2026-03-03

### Status
✅ **COMPLETE — Production fix applied**

---

## Problem
Photos uploaded from Android PDA via global failover (pda.repair) were rejected with **413 Request Entity Too Large**. The Nginx config for `pda.repair` had no `client_max_body_size` set, defaulting to 1MB. Repair photos are 850KB-1MB, so anything slightly over 1MB was blocked.

This was discovered when investigating why only 2 out of 10-15 photos reached the server. The other causes were on the Android side (slot_N.webp overwriting — fixed in eckwms-movFast).

## What Was Done
- Added `client_max_body_size 50M` to the `location ~ ^/E/` block in `/etc/nginx/sites-available/pda.repair.conf`
- `nginx -t` passed, `systemctl reload nginx` applied

## Verification
```
$ grep client_max_body_size /etc/nginx/sites-available/pda.repair.conf
        client_max_body_size 50M;
```

Other eck*.com configs already had 100M — only pda.repair was missing it.

---

## No Code Changes
The Rust server (`eckwmsr`) was not modified. The fix was purely Nginx configuration on the production server.

---

**Agent**: Expert Developer (The Fixer)
**Status**: ✅ Complete
