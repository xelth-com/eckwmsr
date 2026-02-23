# Task: Add Zoho Ticket Thread Fetching (Email Body + Attachments)

## Why
Customers send repair protocols, QC reports, damage photos inside Zoho ticket emails.
We need to fetch the full HTML content + attachments per ticket.

## Key files
- `eckwmsr/scraper/server.js` — add `zohoApi()` helper + `/api/zoho/ticket-threads` endpoint
- `web/src/routes/dashboard/scrapers/+page.svelte` — add thread fetch UI to Zoho card (lines ~399-438)

---

## Step 1: Add `zohoApi()` helper to `eckwmsr/scraper/server.js`

Insert after `zohoLogin()` function (after line ~168). Copy from `zoho-clicker/scraper/server.js` line 171:

```js
async function zohoApi(page, path) {
    return page.evaluate(async ([path]) => {
        const sep = path.includes('?') ? '&' : '?';
        const resp = await fetch(path + sep + 'orgId=20078282365', { credentials: 'include' });
        if (!resp.ok) return { error: resp.status, body: await resp.text() };
        return resp.json();
    }, [path]);
}
```

---

## Step 2: Add `POST /api/zoho/ticket-threads` endpoint

Add after the existing `/api/zoho/tickets` endpoint (~line 735). Pattern is identical to tickets endpoint.

```js
// ─── ZOHO DESK: Fetch Ticket Email Threads ─────────────────────────────────
app.post('/api/zoho/ticket-threads', (req, res) => {
    runScraper(req, res, async (page, data) => {
        const username  = data.username || (data._from_env ? process.env.ZOHO_EMAIL    : '') || '';
        const password  = data.password || (data._from_env ? process.env.ZOHO_PASSWORD : '') || '';
        const targetUrl = data.url || process.env.ZOHO_URL || 'https://desk.inbodysupport.eu/agent/';
        const ticketId  = data.ticketId;

        if (!ticketId) throw new Error('ticketId is required');
        if (!username || !password) throw new Error('ZOHO_EMAIL or ZOHO_PASSWORD missing');

        await zohoLogin(page, targetUrl, username, password);
        if (!page.url().includes('desk.inbodysupport.eu')) {
            await page.goto(targetUrl, { waitUntil: 'domcontentloaded', timeout: 30000 });
            await page.waitForLoadState('networkidle', { timeout: 20000 }).catch(() => {});
        }

        const threadsRes = await zohoApi(page, `${ZOHO_BASE}/tickets/${ticketId}/threads`);
        if (threadsRes.error) throw new Error(`Zoho threads API error ${threadsRes.error}: ${threadsRes.body}`);

        const threads = threadsRes.data || [];

        // Fetch attachments for each thread
        for (const thread of threads) {
            const attRes = await zohoApi(page, `${ZOHO_BASE}/tickets/${ticketId}/threads/${thread.id}/attachments`);
            thread.attachments = attRes.error ? [] : (attRes.data || []);
        }

        console.log(`[Zoho] Fetched ${threads.length} threads for ticket ${ticketId}`);
        return { success: true, ticketId, count: threads.length, threads };
    });
});
```

Also add to the `/debug` endpoint's `endpoints` array:
```js
{ method: 'POST', path: '/api/zoho/ticket-threads', desc: 'Zoho Desk: fetch ticket email threads with HTML content and attachments' },
```

Response shape per thread:
```json
{
  "id": "...",
  "direction": "in",
  "content": "<html>...</html>",
  "from": "customer@example.com",
  "createdTime": "2026-02-09T14:19:52.000Z",
  "attachments": [
    { "id": "...", "fileName": "repair_protocol.pdf", "size": 102400, "href": "..." }
  ]
}
```

---

## Step 3: Update frontend `web/src/routes/dashboard/scrapers/+page.svelte`

**Add state variables** in `<script>` section, after `let zohoJsonOpen = false;` (line ~39):
```js
let zohoThreadTicketId = '';
let zohoThreadRunning = false;
let zohoThreadResult = null;
let zohoThreadJsonOpen = false;
```

**Add fetch function** after `testZohoFetch()`:
```js
async function testZohoFetchThreads() {
    if (!zohoThreadTicketId) return;
    zohoThreadRunning = true;
    zohoThreadResult = null;
    const t0 = Date.now();
    try {
        const res = await api.post('/S/api/zoho/ticket-threads', { ticketId: zohoThreadTicketId, _from_env: true });
        zohoThreadResult = { ...res, duration: ((Date.now() - t0) / 1000).toFixed(1) };
    } catch(e) {
        zohoThreadResult = { success: false, error: e.message, duration: ((Date.now() - t0) / 1000).toFixed(1) };
    } finally {
        zohoThreadRunning = false;
    }
}
```

**Add UI block** inside `.zoho-card` div, after the existing `zohoResult` block (~line 440):
```svelte
<div class="sub-section" style="margin-top:0.75rem">
    <label style="font-size:0.8rem;opacity:0.7">Fetch Email Threads</label>
    <div style="display:flex;gap:0.5rem;margin-top:0.25rem">
        <input type="text" bind:value={zohoThreadTicketId}
            placeholder="Ticket ID" disabled={zohoThreadRunning}
            style="flex:1;background:#1a0a2e;border:1px solid #6b21a8;color:#e2d9f3;padding:0.4rem 0.6rem;border-radius:6px" />
        <button class="run-btn zoho-run" on:click={testZohoFetchThreads}
            disabled={zohoThreadRunning || !zohoThreadTicketId || scraperOnline !== true}>
            {#if zohoThreadRunning}<span class="spinner">⏳</span> Fetching...
            {:else}📧 Fetch Threads{/if}
        </button>
    </div>
    {#if zohoThreadResult}
        <div class="result-box" class:result-ok={zohoThreadResult.success} class:result-err={!zohoThreadResult.success}>
            {#if zohoThreadResult.success}
                <div class="result-summary">✅ {zohoThreadResult.count} threads in {zohoThreadResult.duration}s</div>
            {:else}
                <div class="result-summary error">❌ {zohoThreadResult.error}</div>
            {/if}
            {#if zohoThreadResult.threads?.length}
                <button class="toggle-json" on:click={() => zohoThreadJsonOpen = !zohoThreadJsonOpen}>
                    {zohoThreadJsonOpen ? '▼' : '▶'} View threads ({zohoThreadResult.threads.length})
                </button>
                {#if zohoThreadJsonOpen}<pre class="result-json">{JSON.stringify(zohoThreadResult.threads, null, 2)}</pre>{/if}
            {/if}
        </div>
    {/if}
</div>
```

---

## Notes
- `ZOHO_BASE` is already defined as `/supportapi/zd/inbodyeu/api/v1` in server.js
- `zohoApi()` auto-appends `orgId=20078282365` to all requests
- Thread `content` is HTML — may contain embedded images as base64 or cid refs
- Attachment `href` is a direct download URL valid while Playwright session is alive
- `runScraper` wraps browser lifecycle — just use `page` and `data` in callback
- Frontend proxy: requests go via `/S/api/...` → Rust proxies to `http://127.0.0.1:3211`


[SYSTEM: EMBEDDED]
