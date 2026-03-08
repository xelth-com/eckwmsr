# Task Complete: Dynamic Repair Schemas & Excel Sync

## Date: 2026-03-08

### Status
✅ **COMPLETE — Excel bidirectional sync + dynamic metadata renderer on Repair Detail page**

---

## What Was Done

### 1. Excel Sync Backend (`scraper/server.js`)
- Added `exceljs` dependency for reading/writing `.xlsm` files (preserves VBA macros)
- **`GET /api/excel/info`** — file metadata: path, size, repair count, last repair number
- **`POST /api/excel/read`** — reads repairs from "Körperanalyse" sheet, returns JSON (newest first, with limit/offset)
- **`POST /api/excel/write-row`** — adds or updates a row by repair number (with automatic `.bak` backup)
- Handles Excel formula cells (shared formulas), hyperlink cells, rich text, Date objects
- Column mapping (`EXCEL_COL` object) easily replaceable for other Excel files
- 30+ mapped fields: ticket#, repair#, warranty, error description, troubleshooting, defective parts (6 slots), firmware before/after, model, serial, customer, dates, completion status

### 2. Excel Sync UI (`web/src/routes/dashboard/scrapers/+page.svelte`)
- New "Excel Reparaturliste" section (orange themed) on Scrapers page
- **Info button** — shows repair count, last repair number, file modification date
- **Import tab** (Excel → DB): Read Excel → table with checkboxes → Import selected to orders table
- **Export tab** (DB → Excel): Load repairs from DB → select → Write to Excel file
- All operations manual, with preview, nothing automatic
- Import creates `orders` records with `type=repair`, maps all Excel fields to order fields + metadata JSONB

### 3. Dynamic Repair Schemas (`web/src/routes/dashboard/repairs/[id]/+page.svelte`)
- **Replaced Parts**: tag-based editor bound to `partsUsed` JSON array (add/remove with Enter key support)
- **Dynamic Attributes**: renders all `metadata` JSONB fields as editable form inputs
  - Nested objects (e.g. `fwBefore: {kernel, digital, analog}`) → grouped sub-fields with header
  - Boolean values → checkboxes
  - Strings/numbers → text inputs
  - System keys (`ticketId`, `trackingNumber`, `importedFromExcel`, `excelRow`) hidden from display
- **Add Custom Field**: key/value input with type inference (true/false → boolean, numbers → number)
- `formatKey()` converts camelCase to Title Case for display

### 4. Config
- `.env`: Added `EXCEL_REPAIR_FILE` path (relative to project root)
- `scraper/package.json`: Added `exceljs` dependency

---

## Files Changed

| File | Change |
|------|--------|
| `scraper/server.js` | +ExcelJS import, +EXCEL_COL mapping, +3 endpoints (info/read/write-row), +cellVal with formula/hyperlink/richtext handling |
| `scraper/package.json` | +`exceljs` dependency |
| `web/src/routes/dashboard/scrapers/+page.svelte` | +Excel Sync section (state, functions, UI, CSS) |
| `web/src/routes/dashboard/repairs/[id]/+page.svelte` | Full rewrite: +partsUsed tags, +dynamic metadata grid, +custom field adder |
| `.env` | +`EXCEL_REPAIR_FILE` |
| `.eck/JOURNAL.md` | +2026-03-08 entry |

## Build
- `npm run build` — **OK** (SvelteKit static adapter)
- Excel endpoints tested: info returns 1224 repairs, read returns correct data with proper cell parsing

---

**Agent**: Expert Developer (The Fixer)
**Status**: ✅ Complete
