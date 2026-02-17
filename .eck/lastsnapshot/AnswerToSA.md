# Task: Test OPAL Playwright scraper (/api/opal/fetch)

## Status: DONE

## Summary
Tested `/api/opal/fetch` endpoint with real OPAL credentials from `eckwmsgo/.env`.
The endpoint works correctly — no fixes were needed.

## Test Result
```
curl -X POST http://127.0.0.1:3211/api/opal/fetch \
  -H 'Content-Type: application/json' \
  -d '{"username":"ib02","password":"Inbodygermany2021","limit":3}'
```

**Response:** `success: true`, 3 orders parsed correctly:
- `OCU-998-512895` — terra sports Gladbeck → InBody Deutschland, 17.5kg, ausgeliefert
- `OCU-998-512712` — Rhön-Klinikum Campus → InBody Deutschland, 47.5kg, ausgeliefert
- `OCU-998-512751` — InBody Deutschland → Clever Fit Holzgerlingen, 43.5kg, Zugestellt

All fields parsed: tracking_number, hwb_number, product_type, pickup/delivery address, weight, status, status_date.

## Notes
- OPAL credentials found in `C:\Users\Dmytro\eckwmsgo\.env` (OPAL_USERNAME=ib02)
- Scraper server started manually (not auto-started on boot)
- `eckwmsr/.env` does not yet have OPAL credentials — add them for production
- The Go reference script `eckwmsgo/scripts/delivery/fetch-opal-orders.js` has a more complete parser (pickup_date, delivery_date, package_count, dimensions, receiver) than the current `scraper/server.js`

## No fixes needed
The existing OPAL fetch logic in `scraper/server.js` handled the frameset architecture correctly.
