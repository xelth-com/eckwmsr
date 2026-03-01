<script>
    import { onMount } from "svelte";
    import { api } from "$lib/api";
    import { toastStore } from "$lib/stores/toastStore.js";

    export let data;

    let syncHistory = data.syncHistory || [];
    let loading = false;
    let error = data.error || null;
    let activeTab = "scraper"; // 'scraper', 'sync'

    // ── Scraper Admin state ──────────────────────────────────────────────────
    let scraperStatus = null;
    let scraperOnline = null;
    let scraperStarting = false;
    let scraperStartError = null;

    let opalDebug = false;
    let opalLimit = 10;
    let opalRunning = false;
    let opalResult = null;

    let dhlDebug = false;
    let dhlLimit = 10;
    let dhlRunning = false;
    let dhlResult = null;

    let opalJsonOpen = false;
    let dhlJsonOpen = false;

    let exactDebug = false;
    let exactRunning = false;
    let exactResult = null;
    let exactJsonOpen = false;

    let zohoDebug = false;
    let zohoRunning = false;
    let zohoLimit = 10;
    let zohoResult = null;
    let zohoJsonOpen = false;

    let zohoThreadTicketId = '';
    let zohoThreadRunning = false;
    let zohoThreadResult = null;
    let zohoThreadJsonOpen = false;

    let zohoImportRunning = false;
    let zohoImportResult = null;

    let zohoImportAllRunning = false;
    let zohoImportAllProgress = '';
    let zohoImportAllResult = null;

    let expandedSyncLogs = new Set();

    onMount(async () => {
        if (scraperOnline === null) {
            await loadScraperStatus();
        }
    });

    async function loadData() {
        loading = true;
        error = null;
        try {
            syncHistory = await api.get("/api/delivery/sync/history") || [];
            if (activeTab === 'scraper') {
                await loadScraperStatus();
            }
        } catch (e) {
            console.error(e);
            error = e.message;
        } finally {
            loading = false;
        }
    }

    async function loadScraperStatus() {
        try {
            scraperStatus = await api.get('/S/debug');
            scraperOnline = true;
        } catch {
            scraperOnline = false;
            scraperStatus = null;
        }
    }

    async function startScraper() {
        scraperStarting = true;
        scraperStartError = null;
        try {
            const res = await api.post('/api/scraper/start', {});
            if (res.success) {
                toastStore.add(res.message, 'success');
                // Poll until scraper is reachable
                for (let i = 0; i < 10; i++) {
                    await new Promise(r => setTimeout(r, 2000));
                    try {
                        scraperStatus = await api.get('/S/debug');
                        scraperOnline = true;
                        scraperStarting = false;
                        return;
                    } catch {}
                }
                scraperStartError = 'Process started but scraper did not become reachable within 20s. Check server logs.';
                scraperOnline = false;
            } else {
                scraperStartError = res.error || 'Unknown error';
            }
        } catch (e) {
            scraperStartError = e.message || 'Failed to call start endpoint';
        } finally {
            scraperStarting = false;
        }
    }

    async function copyStartError() {
        const txt = `# eckWMS Scraper Start Error
You are a technical assistant for eckWMS (Rust warehouse management system).
The user tried to start the Playwright scraper from the admin UI.

## System
- eckWMS: Rust (axum) + SvelteKit + PostgreSQL on port 3210
- Scraper: Node.js + Playwright, expected on port 3211
- Start: node scraper/server.js (from project root)
- Scraper waits for main server /E/health before listening

## Error
${scraperStartError}

## Possible causes
- Node.js not in PATH
- scraper/server.js not found (wrong working directory)
- Port 3211 already in use
- Main server /E/health not responding (scraper exits after 60s)
- Missing deps (cd scraper && npm install)

Analyze and suggest a fix. Be concise.`.trim();
        try {
            await navigator.clipboard.writeText(txt);
            toastStore.add('Error copied for AI analysis', 'success');
        } catch (err) {
            toastStore.add('Failed to copy: ' + err.message, 'error');
        }
    }

    async function testOpalFetch() {
        opalRunning = true;
        opalResult = null;
        opalJsonOpen = false;
        const t0 = Date.now();
        try {
            const res = await api.post('/S/api/opal/fetch', {
                username: '',
                password: '',
                limit: opalLimit,
                debug: opalDebug,
                _from_env: true
            });
            opalResult = { ...res, duration: ((Date.now() - t0) / 1000).toFixed(1) };
        } catch (e) {
            opalResult = { success: false, error: e.message, duration: ((Date.now() - t0) / 1000).toFixed(1) };
        } finally {
            opalRunning = false;
        }
    }

    async function testDhlFetch() {
        dhlRunning = true;
        dhlResult = null;
        dhlJsonOpen = false;
        const t0 = Date.now();
        try {
            const res = await api.post('/S/api/dhl/fetch', {
                username: '',
                password: '',
                limit: dhlLimit,
                debug: dhlDebug,
                _from_env: true
            });
            dhlResult = { ...res, duration: ((Date.now() - t0) / 1000).toFixed(1) };
        } catch (e) {
            dhlResult = { success: false, error: e.message, duration: ((Date.now() - t0) / 1000).toFixed(1) };
        } finally {
            dhlRunning = false;
        }
    }

    async function testExactFetch() {
        exactRunning = true;
        exactResult = null;
        exactJsonOpen = false;
        const t0 = Date.now();
        try {
            const res = await api.post('/S/api/exact/inventory/fetch', { _from_env: true, debug: exactDebug });
            exactResult = { ...res, duration: ((Date.now() - t0) / 1000).toFixed(1) };
        } catch (e) {
            exactResult = { success: false, error: e.message, duration: ((Date.now() - t0) / 1000).toFixed(1) };
        } finally {
            exactRunning = false;
        }
    }

    async function testZohoFetch() {
        zohoRunning = true;
        zohoResult = null;
        zohoJsonOpen = false;
        const t0 = Date.now();
        try {
            const res = await api.post('/S/api/zoho/tickets', { limit: zohoLimit, _from_env: true, debug: zohoDebug });
            zohoResult = { ...res, duration: ((Date.now() - t0) / 1000).toFixed(1) };
        } catch (e) {
            zohoResult = { success: false, error: e.message, duration: ((Date.now() - t0) / 1000).toFixed(1) };
        } finally {
            zohoRunning = false;
        }
    }

    async function importThreadsToSystem() {
        if (!zohoThreadResult?.threads?.length) return;
        zohoImportRunning = true;
        zohoImportResult = null;
        try {
            // Pass ticket metadata from thread fetch or from ticket list
            const ticket = zohoThreadResult.ticket
                || zohoResult?.tickets?.find(t => t.id === zohoThreadTicketId)
                || null;
            const res = await api.post('/api/support/import-thread', {
                ticketId: zohoThreadTicketId,
                threads: zohoThreadResult.threads,
                ticket,
            });
            zohoImportResult = res;
            if (res.imported > 0) {
                toastStore.add(`Imported ${res.imported} thread(s) to system`, 'success');
            } else {
                toastStore.add('Import finished with errors', 'error');
            }
        } catch (e) {
            zohoImportResult = { success: false, imported: 0, errors: [e.message] };
            toastStore.add('Import failed: ' + e.message, 'error');
        } finally {
            zohoImportRunning = false;
        }
    }

    async function importAllTickets() {
        const tickets = zohoResult?.tickets;
        if (!tickets?.length) return;
        zohoImportAllRunning = true;
        zohoImportAllResult = null;
        zohoImportAllProgress = `Fetching threads for ${tickets.length} tickets (single session)…`;

        let imported = 0;
        let skipped = 0;
        const errors = [];

        try {
            // Bulk fetch: one browser, one login, all tickets
            const bulkRes = await api.post('/S/api/zoho/ticket-threads-bulk', {
                ticketIds: tickets.map(t => t.id),
                _from_env: true,
            });

            if (!bulkRes.success || !bulkRes.results) {
                throw new Error(bulkRes.error || 'Bulk fetch failed');
            }

            // Now save each result to system
            for (let i = 0; i < bulkRes.results.length; i++) {
                const r = bulkRes.results[i];
                if (!r.success || !r.threads?.length) {
                    if (r.error) errors.push(`#${r.ticketId}: ${r.error}`);
                    else skipped++;
                    continue;
                }

                zohoImportAllProgress = `Saving ${i + 1}/${bulkRes.results.length}: #${r.ticketId} (${r.threads.length} threads)…`;

                try {
                    const ticket = r.ticket || tickets.find(t => t.id === r.ticketId) || null;
                    const importRes = await api.post('/api/support/import-thread', {
                        ticketId: r.ticketId,
                        threads: r.threads,
                        ticket,
                    });

                    if (importRes.imported > 0) {
                        imported += importRes.imported;
                    } else {
                        errors.push(`#${r.ticketId}: 0 threads saved`);
                    }
                } catch (e) {
                    errors.push(`#${r.ticketId}: ${e.message}`);
                }
            }
        } catch (e) {
            errors.push(e.message);
        }

        zohoImportAllProgress = '';
        zohoImportAllResult = { success: errors.length === 0, imported, skipped, total: tickets.length, errors };
        zohoImportAllRunning = false;

        if (imported > 0) {
            toastStore.add(`Imported ${imported} thread(s) from ${tickets.length} tickets`, 'success');
        } else {
            toastStore.add('Import finished with errors', 'error');
        }
    }

    async function testZohoFetchThreads() {
        if (!zohoThreadTicketId) return;
        zohoThreadRunning = true;
        zohoThreadResult = null;
        zohoThreadJsonOpen = false;
        zohoImportResult = null;
        const t0 = Date.now();
        try {
            const res = await api.post('/S/api/zoho/ticket-threads', { ticketId: zohoThreadTicketId, _from_env: true });
            zohoThreadResult = { ...res, duration: ((Date.now() - t0) / 1000).toFixed(1) };
        } catch (e) {
            zohoThreadResult = { success: false, error: e.message, duration: ((Date.now() - t0) / 1000).toFixed(1) };
        } finally {
            zohoThreadRunning = false;
        }
    }

    function formatDate(dateStr) {
        if (!dateStr) return "-";
        return new Date(dateStr).toLocaleDateString("de-DE", {
            day: "2-digit",
            month: "2-digit",
            year: "numeric",
            hour: "2-digit",
            minute: "2-digit",
        });
    }

    function toggleSyncDetails(id) {
        if (expandedSyncLogs.has(id)) {
            expandedSyncLogs.delete(id);
        } else {
            expandedSyncLogs.add(id);
        }
        expandedSyncLogs = expandedSyncLogs;
    }

    function summarizeError(error) {
        if (!error) return 'Unknown error';
        const e = String(error).toLowerCase();
        if (e.includes('timeout') || e.includes('timed out')) return 'Timeout';
        if (e.includes('econnrefused') || e.includes('connection refused')) return 'Connection refused';
        if (e.includes('navigation') || e.includes('goto')) return 'Navigation failed';
        if (e.includes('selector') || e.includes('locator')) return 'Element not found';
        if (e.includes('login') || e.includes('auth') || e.includes('session')) return 'Auth failed';
        if (e.includes('captcha') || e.includes('2fa')) return '2FA/Captcha';
        if (e.includes('network') || e.includes('dns') || e.includes('fetch')) return 'Network error';
        if (e.includes('certificate') || e.includes('ssl')) return 'SSL error';
        if (e.includes('403') || e.includes('forbidden')) return 'Forbidden';
        if (e.includes('404') || e.includes('not found')) return 'Not found';
        if (e.includes('500') || e.includes('server error')) return 'Server error';
        if (e.includes('rate') || e.includes('limit') || e.includes('throttl')) return 'Rate limited';
        const short = String(error).split('\n')[0].substring(0, 60);
        return short.length < String(error).length ? short + '...' : short;
    }

    async function copyScraperError(provider, result) {
        const debugText = `
# eckWMS Scraper Error — ${provider}
You are a technical assistant for eckWMS (warehouse management system). This is an error from the Playwright-based scraper service.

## System
- eckWMS: Rust (axum) + SvelteKit + PostgreSQL
- Scraper: Node.js + Playwright on port 3211, proxied at /E/S/*
- Providers: OPAL (courier), DHL (shipping), Zoho Desk (tickets), Exact Online (ERP)

## Error
**Provider:** ${provider}
**Short:** ${summarizeError(result.error)}
**Full message:** ${result.error || 'No error message'}

## Result JSON
${JSON.stringify(result, null, 2)}

---
Analyze this error and suggest a fix. Be concise.
`.trim();

        try {
            await navigator.clipboard.writeText(debugText);
            toastStore.add('Error copied for AI analysis', 'success');
        } catch (err) {
            toastStore.add('Failed to copy: ' + err.message, 'error');
        }
    }

    async function copyDebugInfo(sync) {
        const debugText = `
# eckWMS Sync Error — ${sync.provider}
You are a technical assistant for eckWMS (warehouse management system). This is an error from a scheduled sync operation.

## System
- eckWMS: Rust (axum) + SvelteKit + PostgreSQL
- Scraper: Node.js + Playwright on port 3211
- Providers: OPAL (courier), DHL (shipping)

## Error
**Provider:** ${sync.provider}
**Time:** ${formatDate(sync.startedAt)}
**Status:** ${sync.status}
**Duration:** ${sync.duration ? (sync.duration / 1000).toFixed(1) + "s" : "N/A"}
**Short:** ${summarizeError(sync.errorDetail)}

## Error Message
${sync.errorDetail || "No error detail"}

## Debug Information
${sync.debugInfo ? JSON.stringify(sync.debugInfo, null, 2) : "No debug info available"}

## Statistics
- Created: ${sync.created || 0}
- Updated: ${sync.updated || 0}
- Skipped: ${sync.skipped || 0}
- Errors: ${sync.errors || 0}

---
Analyze this error and suggest a fix. Be concise.
`.trim();

        try {
            await navigator.clipboard.writeText(debugText);
            toastStore.add("Debug info copied to clipboard!", "success");
        } catch (err) {
            toastStore.add("Failed to copy: " + err.message, "error");
        }
    }
</script>

<div class="scrapers-page">
    <header>
        <h1>🤖 Scrapers & Integrations</h1>
        <div class="header-actions">
            <button class="refresh-btn" on:click={loadData} disabled={loading}>
                {loading ? "↻ Loading..." : "↻ Refresh"}
            </button>
        </div>
    </header>

    <div class="tabs">
        <button
            class="tab"
            class:active={activeTab === "scraper"}
            on:click={() => { activeTab = "scraper"; if (scraperOnline === null) loadScraperStatus(); }}
        >
            🎛️ Scraper Admin
        </button>
        <button
            class="tab"
            class:active={activeTab === "sync"}
            on:click={() => (activeTab = "sync")}
        >
            🔄 Sync History
        </button>
    </div>

    {#if error}
        <div class="error">Failed to load data: {error}</div>
    {:else if activeTab === "scraper"}
        <div class="scraper-section">
            <div class="scraper-status-bar">
                <div class="status-left">
                    <span class="status-dot"
                        class:online={scraperOnline === true}
                        class:offline={scraperOnline === false}
                        class:unknown={scraperOnline === null}
                        class:starting={scraperStarting}
                    ></span>
                    <span class="status-label">
                        {#if scraperStarting}
                            Starting scraper...
                        {:else if scraperOnline === true}
                            Playwright Scraper — running on port {scraperStatus?.port ?? 3211}
                        {:else if scraperOnline === false}
                            Scraper offline
                        {:else}
                            Scraper status unknown
                        {/if}
                    </span>
                </div>
                <div class="status-actions">
                    {#if scraperOnline !== true && !scraperStarting}
                        <button class="run-btn start-scraper-btn" on:click={startScraper}>
                            Start Scraper
                        </button>
                    {/if}
                    <button class="refresh-btn small" on:click={loadScraperStatus} disabled={scraperStarting}>
                        ↻ Check Status
                    </button>
                </div>
            </div>
            {#if scraperStartError}
                <div class="scraper-start-error">
                    <div class="error-row">
                        <span class="error-badge">Failed: {summarizeError(scraperStartError)}</span>
                        <button class="action-btn copy-btn" on:click={copyStartError}>Copy to AI</button>
                    </div>
                    <div class="error-detail">{scraperStartError}</div>
                </div>
            {/if}

            {#if scraperOnline === true && scraperStatus}
                <div class="endpoints-hint">
                    {#each scraperStatus.endpoints as ep}
                        <span class="ep-badge">
                            <span class="ep-method">{ep.method}</span>
                            <span class="ep-path">{ep.path}</span>
                        </span>
                    {/each}
                </div>
            {/if}

            <div class="provider-cards">
                <!-- OPAL card -->
                <div class="provider-card opal-card">
                    <div class="card-header">
                        <span class="card-title">🟢 OPAL Kurier</span>
                        <span class="card-hint">opal-kurier.de</span>
                    </div>
                    <div class="card-controls">
                        <label class="control-row">
                            <span>Limit</span>
                            <select bind:value={opalLimit} disabled={opalRunning}>
                                <option value={5}>5</option>
                                <option value={10}>10</option>
                                <option value={25}>25</option>
                                <option value={50}>50</option>
                            </select>
                        </label>
                        <label class="toggle-row">
                            <input type="checkbox" bind:checked={opalDebug} disabled={opalRunning} />
                            <span class="toggle-label" class:debug-on={opalDebug}>
                                {opalDebug ? '🔍 Debug (headed)' : 'Headless'}
                            </span>
                        </label>
                    </div>
                    {#if opalDebug}
                        <div class="debug-hint">Browser window will open with 600ms slow-motion.</div>
                    {/if}
                    <button class="run-btn opal-run" on:click={testOpalFetch} disabled={opalRunning || scraperOnline !== true}>
                        {#if opalRunning}<span class="spinner">⏳</span> Running{opalDebug ? ' (watch browser)' : '...'}
                        {:else}🚀 Run Fetch{/if}
                    </button>
                    {#if opalResult}
                        <div class="result-box" class:result-ok={opalResult.success} class:result-err={!opalResult.success}>
                            {#if opalResult.success}
                                <div class="result-summary">✅ {opalResult.count} orders fetched in {opalResult.duration}s</div>
                            {:else}
                                <div class="error-row">
                                    <span class="error-badge">❌ {summarizeError(opalResult.error)}</span>
                                    <button class="action-btn copy-btn" on:click={() => copyScraperError('OPAL', opalResult)}>🤖 Copy for AI</button>
                                </div>
                                <div class="error-detail">{opalResult.error}</div>
                            {/if}
                            {#if opalResult.orders?.length}
                                <button class="toggle-json" on:click={() => opalJsonOpen = !opalJsonOpen}>
                                    {opalJsonOpen ? '▼' : '▶'} View JSON ({opalResult.orders.length} orders)
                                </button>
                                {#if opalJsonOpen}<pre class="result-json">{JSON.stringify(opalResult.orders, null, 2)}</pre>{/if}
                            {/if}
                        </div>
                    {/if}
                </div>

                <!-- DHL card -->
                <div class="provider-card dhl-card">
                    <div class="card-header">
                        <span class="card-title">🟡 DHL</span>
                        <span class="card-hint">geschaeftskunden.dhl.de</span>
                    </div>
                    <div class="card-controls">
                        <label class="control-row">
                            <span>Limit</span>
                            <select bind:value={dhlLimit} disabled={dhlRunning}>
                                <option value={5}>5</option>
                                <option value={10}>10</option>
                                <option value={25}>25</option>
                                <option value={50}>50</option>
                            </select>
                        </label>
                        <label class="toggle-row">
                            <input type="checkbox" bind:checked={dhlDebug} disabled={dhlRunning} />
                            <span class="toggle-label" class:debug-on={dhlDebug}>
                                {dhlDebug ? '🔍 Debug (headed)' : 'Headless'}
                            </span>
                        </label>
                    </div>
                    {#if dhlDebug}
                        <div class="debug-hint">Browser window will open with 600ms slow-motion.</div>
                    {/if}
                    <button class="run-btn dhl-run" on:click={testDhlFetch} disabled={dhlRunning || scraperOnline !== true}>
                        {#if dhlRunning}<span class="spinner">⏳</span> Running{dhlDebug ? ' (watch browser)' : '...'}
                        {:else}🚀 Run Fetch{/if}
                    </button>
                    {#if dhlResult}
                        <div class="result-box" class:result-ok={dhlResult.success} class:result-err={!dhlResult.success}>
                            {#if dhlResult.success}
                                <div class="result-summary">✅ {dhlResult.count} shipments fetched in {dhlResult.duration}s</div>
                            {:else}
                                <div class="error-row">
                                    <span class="error-badge">❌ {summarizeError(dhlResult.error)}</span>
                                    <button class="action-btn copy-btn" on:click={() => copyScraperError('DHL', dhlResult)}>🤖 Copy for AI</button>
                                </div>
                                <div class="error-detail">{dhlResult.error}</div>
                            {/if}
                            {#if dhlResult.shipments?.length}
                                <button class="toggle-json" on:click={() => dhlJsonOpen = !dhlJsonOpen}>
                                    {dhlJsonOpen ? '▼' : '▶'} View JSON ({dhlResult.shipments.length} shipments)
                                </button>
                                {#if dhlJsonOpen}<pre class="result-json">{JSON.stringify(dhlResult.shipments, null, 2)}</pre>{/if}
                            {/if}
                        </div>
                    {/if}
                </div>

                <!-- Exact Online card (stub) -->
                <div class="provider-card exact-card">
                    <div class="card-header">
                        <span class="card-title">🔵 Exact Online</span>
                        <span class="card-hint">start.exactonline.de</span>
                    </div>
                    <div class="stub-warning">⚠️ Stub — 2FA not implemented yet</div>
                    <div class="card-controls">
                        <label class="toggle-row">
                            <input type="checkbox" bind:checked={exactDebug} disabled={exactRunning} />
                            <span class="toggle-label" class:debug-on={exactDebug}>
                                {exactDebug ? '🔍 Debug (headed)' : 'Headless'}
                            </span>
                        </label>
                    </div>
                    {#if exactDebug}
                        <div class="debug-hint">Browser window will open with 600ms slow-motion.</div>
                    {/if}
                    <button class="run-btn exact-run" on:click={testExactFetch} disabled={exactRunning || scraperOnline !== true}>
                        {#if exactRunning}<span class="spinner">⏳</span> Running{exactDebug ? ' (watch browser)' : '...'}
                        {:else}🚀 Run Fetch{/if}
                    </button>
                    {#if exactResult}
                        <div class="result-box" class:result-ok={exactResult.success} class:result-err={!exactResult.success}>
                            {#if exactResult.success}
                                <div class="result-summary">✅ Done in {exactResult.duration}s</div>
                            {:else}
                                <div class="error-row">
                                    <span class="error-badge">❌ {summarizeError(exactResult.error)}</span>
                                    <button class="action-btn copy-btn" on:click={() => copyScraperError('Exact Online', exactResult)}>🤖 Copy for AI</button>
                                </div>
                                <div class="error-detail">{exactResult.error}</div>
                            {/if}
                            {#if exactResult.data}
                                <button class="toggle-json" on:click={() => exactJsonOpen = !exactJsonOpen}>
                                    {exactJsonOpen ? '▼' : '▶'} View JSON
                                </button>
                                {#if exactJsonOpen}<pre class="result-json">{JSON.stringify(exactResult.data, null, 2)}</pre>{/if}
                            {/if}
                        </div>
                    {/if}
                </div>

                <!-- Zoho Desk card -->
                <div class="provider-card zoho-card">
                    <div class="card-header">
                        <span class="card-title">🟣 Zoho Desk</span>
                        <span class="card-hint">desk.inbodysupport.eu</span>
                    </div>
                    <div class="card-controls">
                        <label class="control-row">
                            <span>Limit</span>
                            <select bind:value={zohoLimit} disabled={zohoRunning}>
                                <option value={10}>10</option>
                                <option value={50}>50</option>
                                <option value={100}>100</option>
                                <option value={500}>500</option>
                                <option value={1000}>1000</option>
                            </select>
                        </label>
                        <label class="toggle-row">
                            <input type="checkbox" bind:checked={zohoDebug} disabled={zohoRunning} />
                            <span class="toggle-label" class:debug-on={zohoDebug}>
                                {zohoDebug ? '🔍 Debug (headed)' : 'Headless'}
                            </span>
                        </label>
                    </div>
                    {#if zohoDebug}
                        <div class="debug-hint">Browser window will open with 600ms slow-motion.</div>
                    {/if}
                    <button class="run-btn zoho-run" on:click={testZohoFetch} disabled={zohoRunning || scraperOnline !== true}>
                        {#if zohoRunning}<span class="spinner">⏳</span> Running{zohoDebug ? ' (watch browser)' : '...'}
                        {:else}🚀 Fetch Tickets{/if}
                    </button>
                    {#if zohoResult}
                        <div class="result-box" class:result-ok={zohoResult.success} class:result-err={!zohoResult.success}>
                            {#if zohoResult.success}
                                <div class="result-summary">✅ {zohoResult.count ?? zohoResult.tickets?.length ?? 0} tickets in {zohoResult.duration}s</div>
                            {:else}
                                <div class="error-row">
                                    <span class="error-badge">❌ {summarizeError(zohoResult.error)}</span>
                                    <button class="action-btn copy-btn" on:click={() => copyScraperError('Zoho Desk', zohoResult)}>🤖 Copy for AI</button>
                                </div>
                                <div class="error-detail">{zohoResult.error}</div>
                            {/if}
                            {#if zohoResult.tickets?.length}
                                <button class="toggle-json" on:click={() => zohoJsonOpen = !zohoJsonOpen}>
                                    {zohoJsonOpen ? '▼' : '▶'} View JSON ({zohoResult.tickets.length} tickets)
                                </button>
                                {#if zohoJsonOpen}<pre class="result-json">{JSON.stringify(zohoResult.tickets, null, 2)}</pre>{/if}
                                <button class="run-btn import-run" on:click={importAllTickets}
                                    disabled={zohoImportAllRunning || scraperOnline !== true}>
                                    {#if zohoImportAllRunning}<span class="spinner">⏳</span> {zohoImportAllProgress || 'Importing...'}
                                    {:else}📥 Import All to Support{/if}
                                </button>
                            {/if}
                        </div>
                        {#if zohoImportAllResult}
                            <div class="result-box" class:result-ok={zohoImportAllResult.success} class:result-err={!zohoImportAllResult.success}>
                                <div class="result-summary">
                                    {zohoImportAllResult.success ? '✅' : '⚠️'}
                                    {zohoImportAllResult.imported} thread(s) imported from {zohoImportAllResult.total} tickets
                                    {#if zohoImportAllResult.skipped > 0}({zohoImportAllResult.skipped} skipped — no threads){/if}
                                </div>
                                {#if zohoImportAllResult.errors?.length}
                                    <div class="import-errors">
                                        {#each zohoImportAllResult.errors as err}
                                            <div class="import-error-line">⚠️ {err}</div>
                                        {/each}
                                    </div>
                                {/if}
                            </div>
                        {/if}
                    {/if}

                    <div class="threads-section">
                        <div class="threads-row">
                            <input
                                type="text"
                                bind:value={zohoThreadTicketId}
                                placeholder="Ticket ID for email threads"
                                disabled={zohoThreadRunning}
                                class="ticket-id-input"
                            />
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
                                    <div class="error-row">
                                        <span class="error-badge">❌ {summarizeError(zohoThreadResult.error)}</span>
                                        <button class="action-btn copy-btn" on:click={() => copyScraperError('Zoho Threads', zohoThreadResult)}>🤖 Copy for AI</button>
                                    </div>
                                    <div class="error-detail">{zohoThreadResult.error}</div>
                                {/if}
                                {#if zohoThreadResult.threads?.length}
                                    <button class="toggle-json" on:click={() => zohoThreadJsonOpen = !zohoThreadJsonOpen}>
                                        {zohoThreadJsonOpen ? '▼' : '▶'} View threads ({zohoThreadResult.threads.length})
                                    </button>
                                    {#if zohoThreadJsonOpen}<pre class="result-json">{JSON.stringify(zohoThreadResult.threads, null, 2)}</pre>{/if}
                                    <button class="run-btn import-run" on:click={importThreadsToSystem}
                                        disabled={zohoImportRunning}>
                                        {#if zohoImportRunning}<span class="spinner">⏳</span> Saving...
                                        {:else}💾 Save to System{/if}
                                    </button>
                                {/if}
                            </div>
                        {/if}
                        {#if zohoImportResult}
                            <div class="result-box" class:result-ok={zohoImportResult.success} class:result-err={!zohoImportResult.success}>
                                {#if zohoImportResult.success}
                                    <div class="result-summary">✅ {zohoImportResult.imported} thread(s) saved to documents table</div>
                                {:else}
                                    <div class="result-summary error">❌ Import failed ({zohoImportResult.imported} saved)</div>
                                {/if}
                                {#if zohoImportResult.errors?.length}
                                    <div class="import-errors">
                                        {#each zohoImportResult.errors as err}
                                            <div class="import-error-line">⚠️ {err}</div>
                                        {/each}
                                    </div>
                                {/if}
                                {#if zohoImportResult.documents?.length}
                                    <div class="import-ids">
                                        Document IDs: {zohoImportResult.documents.join(', ')}
                                    </div>
                                {/if}
                            </div>
                        {/if}
                    </div>
                </div>
            </div>

            <div class="creds-note">
                Credentials are read from server <code>.env</code>
                (OPAL_USERNAME / DHL_USERNAME). To test with different creds,
                use curl directly on <code>POST /S/api/opal/fetch</code> with
                <code>"username"</code> and <code>"password"</code> fields.
            </div>
        </div>

    {:else if activeTab === "sync"}
        <div class="sync-section">
            <p class="section-desc">
                Synchronization history with external services (OPAL, DHL, Odoo).
                OPAL syncs every hour (on the hour), DHL syncs at :30 past the hour. Active 8 AM - 6 PM.
            </p>

            {#if syncHistory.length === 0}
                <div class="empty-state">
                    <p>📭 No sync history yet</p>
                    <small>Synchronizations will appear automatically</small>
                </div>
            {:else}
                <div class="table-container">
                    <table>
                        <thead>
                            <tr>
                                <th></th>
                                <th>Time</th>
                                <th>Provider</th>
                                <th>Status</th>
                                <th>Created</th>
                                <th>Updated</th>
                                <th>Skipped</th>
                                <th>Duration</th>
                                <th>Actions</th>
                            </tr>
                        </thead>
                        <tbody>
                            {#each syncHistory as sync}
                                <tr
                                    class="sync-row"
                                    class:expanded={expandedSyncLogs.has(sync.id)}
                                    class:has-error={sync.status === "error"}
                                    on:click={() => sync.status === "error" ? toggleSyncDetails(sync.id) : null}
                                >
                                    <td class="expand-cell">
                                        {#if sync.status === "error"}
                                            <span class="expand-icon">{expandedSyncLogs.has(sync.id) ? "▼" : "▶"}</span>
                                        {:else}
                                            <span class="muted">-</span>
                                        {/if}
                                    </td>
                                    <td class="sync-time">{formatDate(sync.startedAt)}</td>
                                    <td>
                                        <span class="provider-badge" class:opal={sync.provider === "opal"} class:dhl={sync.provider === "dhl"}>
                                            {sync.provider.toUpperCase()}
                                        </span>
                                    </td>
                                    <td>
                                        <span class="sync-badge" class:success={sync.status === "success"} class:error={sync.status === "error"} class:running={sync.status === "running"}>
                                            {sync.status === "success" ? "✅ Success" : sync.status === "error" ? "❌ Error" : "⏳ Running"}
                                        </span>
                                    </td>
                                    <td class="stat-cell">{sync.created || 0}</td>
                                    <td class="stat-cell">{sync.updated || 0}</td>
                                    <td class="stat-cell muted">{sync.skipped || 0}</td>
                                    <td class="duration-cell">{sync.duration ? (sync.duration / 1000).toFixed(1) + "s" : "-"}</td>
                                    <td on:click|stopPropagation>
                                        {#if sync.status === "error" && (sync.errorDetail || sync.debugInfo)}
                                            <button class="action-btn copy-btn" on:click={() => copyDebugInfo(sync)} title="Copy debug info for AI">
                                                🤖 Copy for AI
                                            </button>
                                        {:else}
                                            <span class="muted">-</span>
                                        {/if}
                                    </td>
                                </tr>
                                {#if expandedSyncLogs.has(sync.id) && sync.status === "error"}
                                    <tr class="debug-row">
                                        <td colspan="9">
                                            <div class="debug-details">
                                                <div class="debug-section">
                                                    <h4>⚠️ Error</h4>
                                                    <pre class="error-message">{sync.errorDetail || "No error detail"}</pre>
                                                </div>
                                                {#if sync.debugInfo}
                                                    <div class="debug-section">
                                                        <h4>🔍 Debug Information</h4>
                                                        <div class="debug-grid">
                                                            {#if sync.debugInfo.error_category}
                                                                <div class="debug-item">
                                                                    <label>Category:</label>
                                                                    <span class="category-badge" class:playwright={sync.debugInfo.error_category === "playwright_scraper"}>{sync.debugInfo.error_category}</span>
                                                                </div>
                                                            {/if}
                                                            {#if sync.debugInfo.likely_cause}
                                                                <div class="debug-item">
                                                                    <label>Likely Cause:</label>
                                                                    <span class="highlight">{sync.debugInfo.likely_cause}</span>
                                                                </div>
                                                            {/if}
                                                            {#if sync.debugInfo.ai_analysis_hint}
                                                                <div class="debug-item">
                                                                    <label>💡 AI Hint:</label>
                                                                    <span class="ai-hint">{sync.debugInfo.ai_analysis_hint}</span>
                                                                </div>
                                                            {/if}
                                                        </div>
                                                        {#if sync.debugInfo.playwright_stderr}
                                                            <div class="stderr-section">
                                                                <h5>📋 Playwright Output (stderr):</h5>
                                                                <pre class="stderr-output">{sync.debugInfo.playwright_stderr}</pre>
                                                            </div>
                                                        {/if}
                                                        <details class="raw-json">
                                                            <summary>🔧 Raw Debug JSON</summary>
                                                            <pre>{JSON.stringify(sync.debugInfo, null, 2)}</pre>
                                                        </details>
                                                    </div>
                                                {/if}
                                            </div>
                                        </td>
                                    </tr>
                                {/if}
                            {/each}
                        </tbody>
                    </table>
                </div>
            {/if}
        </div>
    {/if}
</div>

<style>
    .scrapers-page { padding: 0; }
    header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 1.5rem; }
    h1 { font-size: 1.8rem; color: #fff; margin: 0; }
    .header-actions { display: flex; gap: 1rem; }

    .refresh-btn { padding: 0.6rem 1.2rem; border-radius: 4px; border: 1px solid #4a69bd; background: transparent; color: #4a69bd; font-weight: 600; cursor: pointer; transition: all 0.2s; }
    .refresh-btn:hover:not(:disabled) { background: #4a69bd; color: white; }
    .refresh-btn:disabled { opacity: 0.5; cursor: not-allowed; }

    .tabs { display: flex; gap: 1rem; margin-bottom: 2rem; border-bottom: 2px solid #333; }
    .tab { padding: 0.8rem 1.5rem; border: none; background: transparent; color: #aaa; font-size: 1rem; font-weight: 600; cursor: pointer; border-bottom: 3px solid transparent; transition: all 0.2s; }
    .tab:hover { color: #fff; }
    .tab.active { color: #4a69bd; border-bottom-color: #4a69bd; }

    .section-desc { color: #aaa; margin-bottom: 1.5rem; font-size: 0.95rem; }
    .error { text-align: center; padding: 3rem; color: #ff6b6b; background: #1e1e1e; border-radius: 8px; border: 1px solid #ff6b6b; }
    .empty-state { text-align: center; padding: 3rem; color: #666; background: #1e1e1e; border-radius: 8px; border: 1px dashed #333; }
    .empty-state p { font-size: 1.2rem; margin: 0 0 0.5rem 0; }
    .empty-state small { color: #555; }

    .table-container { background: #1e1e1e; border-radius: 8px; border: 1px solid #333; overflow-x: auto; }
    table { width: 100%; border-collapse: collapse; }
    thead { background: #252525; }
    th { padding: 1rem; text-align: left; font-weight: 600; color: #aaa; text-transform: uppercase; font-size: 0.75rem; border-bottom: 2px solid #333; }
    td { padding: 1rem; border-bottom: 1px solid #2a2a2a; color: #e0e0e0; }
    tbody tr:hover { background: #252525; }

    .action-btn { padding: 0.5rem 1rem; border-radius: 4px; border: none; font-weight: 600; font-size: 0.85rem; cursor: pointer; transition: all 0.2s; white-space: nowrap; }
    .muted { color: #666; font-style: italic; }

    /* Sync history */
    .sync-time { font-family: monospace; color: #aaa; font-size: 0.9rem; }
    .sync-badge { display: inline-block; padding: 0.3rem 0.8rem; border-radius: 12px; font-size: 0.75rem; font-weight: 600; color: white; }
    .sync-badge.success { background: #28a745; }
    .sync-badge.error { background: #dc3545; }
    .sync-badge.running { background: #17a2b8; }
    .provider-badge { display: inline-block; padding: 0.3rem 0.8rem; border-radius: 4px; background: #2a2a2a; font-family: monospace; font-size: 0.85rem; text-transform: uppercase; color: #4a69bd; }
    .provider-badge.opal { background: #166534; color: #4ade80; }
    .provider-badge.dhl { background: #713f12; color: #fbbf24; }
    .stat-cell { font-family: monospace; text-align: center; color: #4a69bd; font-weight: 600; }
    .duration-cell { font-family: monospace; color: #888; }

    .sync-row { transition: background 0.2s; }
    .sync-row.has-error { cursor: pointer; }
    .sync-row.has-error:hover { background: #2a2a2a; }
    .sync-row.expanded { background: #252525; border-bottom: none; }
    .expand-cell { width: 30px; text-align: center; }
    .expand-icon { color: #666; font-size: 0.8rem; }

    .copy-btn { background: #1a472a; color: #4ade80; border: 1px solid #22c55e; padding: 0.4rem 0.8rem; font-size: 0.8rem; }
    .copy-btn:hover { background: #166534; }

    .debug-row { background: #1a1a1a; }
    .debug-row td { padding: 0; border-bottom: 2px solid #333; }
    .debug-details { padding: 1.5rem; }
    .debug-section { background: #252525; border-radius: 8px; padding: 1rem; margin-bottom: 1rem; }
    .debug-section h4 { margin: 0 0 1rem 0; color: #fff; font-size: 0.95rem; border-bottom: 1px solid #333; padding-bottom: 0.5rem; }
    .error-message { background: #2a1a1a; color: #ff6b6b; padding: 1rem; border-radius: 4px; border-left: 3px solid #dc3545; overflow-x: auto; font-size: 0.85rem; line-height: 1.4; white-space: pre-wrap; word-wrap: break-word; }
    .debug-grid { display: grid; gap: 0.75rem; }
    .debug-item { display: flex; gap: 0.5rem; font-size: 0.9rem; }
    .debug-item label { color: #888; min-width: 150px; flex-shrink: 0; }
    .debug-item .highlight { color: #ffc107; font-weight: 600; }
    .debug-item .ai-hint { color: #4ade80; font-style: italic; }
    .category-badge { display: inline-block; padding: 0.2rem 0.6rem; border-radius: 4px; background: #2a2a2a; font-size: 0.8rem; text-transform: uppercase; font-weight: 600; }
    .category-badge.playwright { background: #422006; color: #fbbf24; }
    .stderr-section { margin-top: 1rem; }
    .stderr-section h5 { margin: 1rem 0 0.5rem 0; color: #aaa; font-size: 0.85rem; }
    .stderr-output { background: #1a1a1a; color: #aaa; padding: 1rem; border-radius: 4px; border: 1px solid #333; overflow-x: auto; font-size: 0.8rem; line-height: 1.4; max-height: 300px; overflow-y: auto; }
    .raw-json { margin-top: 1rem; }
    .raw-json summary { cursor: pointer; color: #888; font-size: 0.85rem; padding: 0.5rem; background: #2a2a2a; border-radius: 4px; user-select: none; }
    .raw-json summary:hover { color: #aaa; background: #333; }
    .raw-json pre { background: #1a1a1a; color: #4a69bd; padding: 1rem; border-radius: 4px; border: 1px solid #333; overflow-x: auto; font-size: 0.75rem; line-height: 1.4; margin-top: 0.5rem; }

    /* Scraper Admin */
    .scraper-section { display: flex; flex-direction: column; gap: 1.25rem; }
    .scraper-status-bar { display: flex; align-items: center; justify-content: space-between; background: #1e1e1e; border: 1px solid #333; border-radius: 8px; padding: 0.8rem 1.2rem; }
    .status-left { display: flex; align-items: center; gap: 0.75rem; font-size: 0.9rem; color: #ccc; }
    .status-dot { width: 10px; height: 10px; border-radius: 50%; flex-shrink: 0; }
    .status-dot.online  { background: #22c55e; box-shadow: 0 0 6px #22c55e; }
    .status-dot.offline { background: #ef4444; box-shadow: 0 0 6px #ef4444; }
    .status-dot.unknown { background: #6b7280; }
    .status-label code { background: #2a2a2a; border-radius: 3px; padding: 0.1rem 0.4rem; font-size: 0.8rem; color: #4a69bd; }
    .status-actions { display: flex; gap: 0.5rem; align-items: center; }
    .start-scraper-btn { padding: 0.4rem 1rem; font-size: 0.8rem; background: #166534; color: #4ade80; border: 1px solid #22c55e; border-radius: 4px; font-weight: 600; cursor: pointer; }
    .start-scraper-btn:hover { background: #14532d; }
    .status-dot.starting { background: #f59e0b; box-shadow: 0 0 6px #f59e0b; animation: pulse 1.2s ease-in-out infinite; }
    @keyframes pulse { 0%,100% { opacity: 1; } 50% { opacity: 0.4; } }
    .scraper-start-error { background: rgba(239,68,68,0.05); border: 1px solid #ef4444; border-radius: 8px; padding: 0.75rem 1rem; display: flex; flex-direction: column; gap: 0.5rem; }
    .refresh-btn.small { padding: 0.4rem 0.8rem; font-size: 0.8rem; }

    .endpoints-hint { display: flex; flex-wrap: wrap; gap: 0.5rem; }
    .ep-badge { display: inline-flex; align-items: center; gap: 0.3rem; background: #1e1e1e; border: 1px solid #333; border-radius: 4px; padding: 0.25rem 0.6rem; font-size: 0.75rem; }
    .ep-method { color: #4a69bd; font-weight: 700; font-family: monospace; }
    .ep-path { color: #888; font-family: monospace; }

    .provider-cards { display: grid; grid-template-columns: repeat(auto-fit, minmax(340px, 1fr)); gap: 1.25rem; }
    .provider-card { background: #1e1e1e; border: 1px solid #333; border-radius: 10px; padding: 1.25rem; display: flex; flex-direction: column; gap: 1rem; }
    .opal-card  { border-color: #166534; }
    .dhl-card   { border-color: #713f12; }
    .exact-card { border-color: #1e3a5f; }
    .zoho-card  { border-color: #4a1d6e; }

    .card-header { display: flex; align-items: baseline; justify-content: space-between; }
    .card-title { font-size: 1.1rem; font-weight: 700; color: #fff; }
    .card-hint { font-size: 0.75rem; color: #666; font-family: monospace; }
    .card-controls { display: flex; align-items: center; gap: 1.5rem; flex-wrap: wrap; }
    .control-row { display: flex; align-items: center; gap: 0.5rem; font-size: 0.85rem; color: #aaa; }
    .control-row select { background: #2a2a2a; color: #e0e0e0; border: 1px solid #444; border-radius: 4px; padding: 0.3rem 0.5rem; font-size: 0.85rem; cursor: pointer; }
    .toggle-row { display: flex; align-items: center; gap: 0.5rem; cursor: pointer; font-size: 0.85rem; }
    .toggle-label { color: #888; }
    .toggle-label.debug-on { color: #fbbf24; font-weight: 600; }
    .debug-hint { font-size: 0.8rem; color: #fbbf24; background: rgba(251,191,36,0.08); border: 1px solid rgba(251,191,36,0.2); border-radius: 4px; padding: 0.5rem 0.75rem; }

    .run-btn { padding: 0.75rem 1.5rem; border-radius: 6px; border: none; font-weight: 700; font-size: 0.95rem; cursor: pointer; transition: all 0.2s; }
    .opal-run { background: #166534; color: #4ade80; border: 1px solid #22c55e; }
    .opal-run:hover:not(:disabled) { background: #14532d; }
    .dhl-run { background: #713f12; color: #fbbf24; border: 1px solid #f59e0b; }
    .dhl-run:hover:not(:disabled) { background: #92400e; }
    .exact-run { background: #1e3a5f; color: #93c5fd; border: 1px solid #3b82f6; }
    .exact-run:hover:not(:disabled) { background: #1e40af; }
    .zoho-run { background: #4a1d6e; color: #d8b4fe; border: 1px solid #a855f7; }
    .zoho-run:hover:not(:disabled) { background: #6b21a8; }
    .stub-warning { font-size: 0.78rem; color: #f59e0b; }
    .run-btn:disabled { opacity: 0.45; cursor: not-allowed; }
    .spinner { display: inline-block; animation: spin 1s linear infinite; }
    @keyframes spin { to { transform: rotate(360deg); } }

    .result-box { border-radius: 6px; border: 1px solid #333; padding: 0.75rem 1rem; display: flex; flex-direction: column; gap: 0.5rem; }
    .result-box.result-ok  { border-color: #22c55e; background: rgba(34,197,94,0.05); }
    .result-box.result-err { border-color: #ef4444; background: rgba(239,68,68,0.05); }
    .result-summary { font-size: 0.9rem; color: #e0e0e0; }
    .result-summary.error { color: #ff6b6b; }
    .toggle-json { align-self: flex-start; background: none; border: none; color: #4a69bd; font-size: 0.82rem; cursor: pointer; padding: 0; font-family: monospace; }
    .toggle-json:hover { text-decoration: underline; }
    .result-json { background: #141414; color: #4a69bd; border: 1px solid #2a2a2a; border-radius: 4px; padding: 0.75rem; font-size: 0.72rem; line-height: 1.5; overflow: auto; max-height: 400px; white-space: pre; }

    .threads-section { display: flex; flex-direction: column; gap: 0.75rem; border-top: 1px solid #2a2a2a; padding-top: 1rem; }
    .threads-row { display: flex; gap: 0.5rem; }
    .ticket-id-input { flex: 1; background: #141414; border: 1px solid #4a1d6e; color: #e2d9f3; padding: 0.5rem 0.75rem; border-radius: 6px; font-size: 0.85rem; font-family: monospace; }
    .ticket-id-input::placeholder { color: #555; }
    .ticket-id-input:focus { outline: none; border-color: #a855f7; }
    .ticket-id-input:disabled { opacity: 0.5; cursor: not-allowed; }

    .creds-note { font-size: 0.8rem; color: #555; text-align: center; }
    .creds-note code { background: #2a2a2a; border-radius: 3px; padding: 0.1rem 0.35rem; color: #888; }

    .import-run { background: #1a3a5c; color: #93c5fd; border: 1px solid #3b82f6; align-self: flex-start; padding: 0.5rem 1rem; font-size: 0.85rem; }
    .import-run:hover:not(:disabled) { background: #1e40af; }
    .import-errors { display: flex; flex-direction: column; gap: 0.25rem; }
    .import-error-line { font-size: 0.8rem; color: #fbbf24; background: rgba(251,191,36,0.07); border-radius: 4px; padding: 0.3rem 0.6rem; }
    .import-ids { font-size: 0.75rem; color: #555; font-family: monospace; word-break: break-all; }

    .error-row { display: flex; align-items: center; justify-content: space-between; gap: 0.75rem; }
    .error-badge { font-size: 0.9rem; font-weight: 700; color: #ff6b6b; white-space: nowrap; }
    .error-detail { font-size: 0.78rem; color: #aa6b6b; font-family: monospace; background: #1a1010; border-left: 3px solid #ef4444; padding: 0.5rem 0.75rem; border-radius: 4px; white-space: pre-wrap; word-break: break-word; max-height: 120px; overflow-y: auto; }
</style>
