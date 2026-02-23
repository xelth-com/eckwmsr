# Task: Implement Zoho Ticket Thread Scraping

## Context
User wants to scrape full email thread content from Zoho Desk tickets.
Customers send repair protocols, QC reports, images of defects inside emails.
This data is currently not collected.

## What's already done (this session)
- `eckwmsr/scraper/server.js` — `POST /api/zoho/tickets` now works (was a stub before)
  - Logs in via Playwright, fetches ticket list via internal Zoho API with session cookies
  - Pattern: `page.evaluate(async (url) => fetch(url, { credentials: 'include' }).then(r => r.json()), url)`
  - Returns `{ success, count, tickets[] }` with full ticket metadata
- Scraper proxy fixed: routes moved from `/S/*` to `/E/S/*` in `src/main.rs`
- Scrapers page created: `web/src/routes/dashboard/scrapers/+page.svelte`

## What needs to be implemented

### New endpoint: `POST /api/zoho/ticket-threads`
File: `C:\Users\Dmytro\eckwmsr\scraper\server.js`

Input: `{ ticketId: "53451000033028039", _from_env: true }`

Steps:
1. Login via `zohoLogin()` (already defined in server.js)
2. Fetch thread list:
   ```
   GET /supportapi/zd/inbodyeu/api/v1/tickets/{ticketId}/threads?orgId=20078282365
   ```
3. For each thread fetch attachments:
   ```
   GET /supportapi/zd/inbodyeu/api/v1/tickets/{ticketId}/threads/{threadId}/attachments?orgId=20078282365
   ```
4. Return combined structure (see below)

### zohoApi helper — copy from zoho-clicker
```javascript
async function zohoApi(page, path) {
    return page.evaluate(async ([path]) => {
        const sep = path.includes('?') ? '&' : '?';
        const resp = await fetch(path + sep + 'orgId=20078282365', { credentials: 'include' });
        if (!resp.ok) return { error: resp.status, body: await resp.text() };
        return resp.json();
    }, [path]);
}
const BASE = '/supportapi/zd/inbodyeu/api/v1';
// orgId=20078282365, deptId=53451000019414029
```

### Suggested response format
```json
{
  "success": true,
  "ticket_id": "...",
  "thread_count": 7,
  "threads": [
    {
      "id": "...",
      "direction": "in",
      "content": "<html>...</html>",
      "from": "customer@example.com",
      "to": "support@inbody.com",
      "createdTime": "2026-02-09T14:19:52.000Z",
      "attachments": [
        { "id": "...", "fileName": "repair_protocol.pdf", "size": 102400, "href": "..." }
      ]
    }
  ]
}
```

### zoho-clicker DB schema for reference
`C:\Users\Dmytro\zoho-clicker\scraper\server.js` has full implementation:
```sql
zoho_ticket_threads: id, ticket_id, direction, content (HTML), summary,
  content_type, from_email, to_email, created_time, is_description_thread
```

## Key constants
- orgId: `20078282365`
- deptId: `53451000019414029`
- Base URL: `https://desk.inbodysupport.eu/agent/`
- Internal API base (relative, use in page.evaluate): `/supportapi/zd/inbodyeu/api/v1`

## Frontend extension
`C:\Users\Dmytro\eckwmsr\web\src\routes\dashboard\scrapers\+page.svelte`
Zoho card — add a text input for ticket number + "Fetch Threads" button that calls the new endpoint.
