const express = require('express');
const { chromium } = require('playwright');
const fs = require('fs').promises;
const path = require('path');
const app = express();

app.use(express.json());

// Helper to launch browser, run logic, and close
// Uses persistent user data directory to preserve cookies/sessions across requests.
// Pass ?debug=1 in the request body or query to run in headed (visible) mode with slowMo.
const USER_DATA_DIR = path.join(__dirname, '.browser-data');
let activeBrowser = null;
let browserLock = Promise.resolve();

async function runScraper(req, res, logicFn) {
    const debugMode = req.body?.debug || req.query?.debug;
    const headless = !debugMode;
    const slowMo = debugMode ? 600 : 0;
    if (debugMode) console.log('[Scraper] DEBUG MODE — browser window will be visible');

    // Serialize requests to avoid concurrent browser conflicts
    const prevLock = browserLock;
    let releaseLock;
    browserLock = new Promise(r => releaseLock = r);
    await prevLock;

    let context = null;
    let page = null;
    try {
        context = await chromium.launchPersistentContext(USER_DATA_DIR, {
            headless,
            slowMo,
            args: ['--no-sandbox', '--disable-setuid-sandbox', '--disable-blink-features=AutomationControlled'],
            viewport: { width: 1920, height: 1080 },
            userAgent: 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36',
            locale: 'de-DE',
            timezoneId: 'Europe/Berlin',
            acceptDownloads: true
        });
        page = await context.newPage();
        const result = await logicFn(page, req.body);
        res.json(result);
    } catch (error) {
        console.error("Scraper Error:", error);
        res.status(500).json({ error: error.message });
    } finally {
        if (page) await page.close().catch(() => {});
        if (context) await context.close().catch(() => {});
        releaseLock();
    }
}

// Inject OneTrust consent into localStorage before page loads — skips cookie banner
async function injectDhlConsentCookies(page) {
    await page.addInitScript(() => {
        const consent = 'groups=C0001:1,C0002:1,C0003:1,C0004:1';
        localStorage.setItem('OptanonAlertBoxClosed', new Date().toISOString());
        localStorage.setItem('OptanonConsent', consent);
    });
}

// Force-dismiss any OneTrust overlay that may block clicks
async function dismissOnetrust(page) {
    await page.evaluate(() => {
        // Remove the blocking overlay div
        document.querySelectorAll('.onetrust-pc-dark-filter, #onetrust-consent-sdk').forEach(el => el.remove());
    });
    await page.waitForTimeout(500);
}

// DHL login sequence
async function dhlLogin(page, targetUrl, username, password) {
    // Inject consent BEFORE first page load so OneTrust skips the banner
    await injectDhlConsentCookies(page);

    await page.goto(targetUrl, { waitUntil: 'domcontentloaded', timeout: 60000 });
    console.log(`[DHL] Loaded: ${page.url()}`);
    // Wait for SPA to fully render
    await page.waitForTimeout(5000);
    // Debug: log all buttons found on page
    const pageButtons = await page.evaluate(() =>
        Array.from(document.querySelectorAll('button')).map(b => b.textContent?.trim().substring(0, 40)));
    console.log(`[DHL] Buttons on page: ${JSON.stringify(pageButtons)}`);
    const pageText = await page.evaluate(() => document.body?.innerText?.substring(0, 200) || 'EMPTY');
    console.log(`[DHL] Page text preview: ${pageText}`);

    // Click "Anmelden" login trigger using JS to bypass any remaining overlays
    const clicked = await page.evaluate(() => {
        const selectors = [
            '[data-testid="noName"]',
            '.login-module-container button',
            '.dhlBtn-primary',
            'button'
        ];
        for (const sel of selectors) {
            for (const btn of document.querySelectorAll(sel)) {
                if (btn.textContent?.includes('Anmelden')) { btn.click(); return sel + ':' + btn.textContent.trim(); }
            }
        }
        return null;
    });
    console.log(`[DHL] Login trigger clicked via JS: ${clicked}`);

    // Wait for SSO redirect and form to render (Keycloak SSO)
    try {
        await page.waitForSelector('input[name="username"], input[type="email"], input[id="username"]', { timeout: 15000 });
    } catch (e) {
        console.warn(`[DHL] Login form wait timed out. URL: ${page.url()}`);
    }
    console.log(`[DHL] URL after trigger: ${page.url()}`);

    // Fill credentials (Keycloak uses name="username", not type="email")
    const emailField = page.locator('input[name="username"], input[type="email"], input[id="username"]').first();
    const passField = page.locator('input[type="password"], input[name="password"]').first();
    const emailCount = await emailField.count();
    console.log(`[DHL] Login form field count: ${emailCount}`);

    if (emailCount > 0) {
        await emailField.fill(username);
        await page.waitForTimeout(300);
        if (await passField.count() > 0) await passField.fill(password);

        for (const sel of ['input[name="login"]', 'button[type="submit"]', '#kc-login', 'button.btn-primary']) {
            try {
                const btn = page.locator(sel).first();
                if (await btn.count() > 0 && await btn.isVisible({ timeout: 2000 })) {
                    await btn.click();
                    console.log(`[DHL] Submit clicked: ${sel}`);
                    break;
                }
            } catch (e) { /* try next */ }
        }
        await page.waitForTimeout(8000);
        console.log(`[DHL] URL after login: ${page.url()}`);
    } else {
        console.warn('[DHL] Login form NOT FOUND');
    }
}

// Exact Online login sequence
// Flow: start.exactonline.de (email) → Weiter → login.exact.com/Azure B2C (password) → Anmelden → dashboard
async function exactLogin(page, targetUrl, username, password) {
    console.log(`[Exact] Navigating to ${targetUrl}`);
    await page.goto(targetUrl, { waitUntil: 'domcontentloaded', timeout: 60000 });

    // Step 1: fill email on start.exactonline.de and click "Weiter"
    await page.waitForSelector('[id="LoginForm$UserName"]', { timeout: 15000 });
    await page.fill('[id="LoginForm$UserName"]', username);
    await page.waitForTimeout(300);
    await page.click('button[type="submit"]');
    console.log('[Exact] Clicked Weiter, waiting for Azure B2C redirect...');

    // Step 2: Azure B2C page (login.exact.com) — fill password and click "Anmelden"
    await page.waitForSelector('#password', { timeout: 20000 });
    await page.fill('#password', password);
    await page.waitForTimeout(300);
    await page.click('#next');
    console.log('[Exact] Clicked Anmelden, waiting for dashboard...');

    // Wait for redirect back to start.exactonline.de
    await page.waitForURL('**/start.exactonline.de/**', { timeout: 30000 }).catch(() => {});
    await page.waitForTimeout(3000);
    console.log(`[Exact] URL after login: ${page.url()}`);
}

// Zoho Desk login sequence — uses persistent cookies, only logs in when needed
async function zohoLogin(page, targetUrl, username, password) {
    // Step 1: Quick session check via API (no navigation needed if cookies are valid)
    const sessionOk = await page.evaluate(async (base) => {
        try {
            const r = await fetch(base + '/tickets?limit=1&orgId=20078282365', { credentials: 'include' });
            return r.ok;
        } catch { return false; }
    }, ZOHO_BASE).catch(() => false);

    if (sessionOk) {
        console.log('[Zoho] Session cookies valid — skipping login.');
        return;
    }

    // Step 2: Need to navigate to Desk to establish session
    console.log(`[Zoho] Session expired. Navigating to ${targetUrl}`);
    await page.goto(targetUrl, { waitUntil: 'domcontentloaded', timeout: 60000 });
    await page.waitForTimeout(3000);

    const currentUrl = page.url();

    // Detect signin-block/announcement — click "I understand" if present, otherwise fail
    if (currentUrl.includes('signin-block') || currentUrl.includes('announcement')) {
        console.log('[Zoho] Signin warning/block page detected. Looking for "I understand" button...');
        try {
            const understandBtn = page.locator('button:has-text("I understand"), button:has-text("Ich verstehe"), a:has-text("I understand"), .understand_btn, #understand_btn');
            await understandBtn.first().waitFor({ state: 'visible', timeout: 5000 });
            await understandBtn.first().click();
            console.log('[Zoho] Clicked "I understand", waiting for redirect...');
            await page.waitForTimeout(5000);
        } catch {
            throw new Error('Zoho signin blocked (too many logins today). Could not find "I understand" button. Wait 24h or login manually in Debug mode.');
        }
    }

    if (currentUrl.includes('accounts.zoho')) {
        console.log('[Zoho] Login page detected. Proceeding with authentication...');

        // Step 1: Email
        await page.waitForSelector('#login_id', { timeout: 15000 });
        await page.fill('#login_id', username);
        await page.click('#nextbtn');
        console.log('[Zoho] Email submitted, waiting for password field...');
        await page.waitForTimeout(2000);

        // Step 2: Password
        await page.waitForSelector('#password', { timeout: 15000 });
        await page.fill('#password', password);
        await page.click('#nextbtn');
        console.log('[Zoho] Password submitted, waiting for redirect back to Desk...');
        await page.waitForTimeout(8000);

        // Check for signin-block after login attempt
        if (page.url().includes('signin-block')) {
            throw new Error('Zoho signin blocked after login attempt. Wait 24h or login manually in Debug mode.');
        }

        // Handle "Trust this browser" / "Remind me later" popup if it appears
        try {
            const trustBtn = page.locator('button:has-text("Remind me later"), button:has-text("Später erinnern"), .remind_me_later');
            if (await trustBtn.count() > 0 && await trustBtn.first().isVisible()) {
                await trustBtn.first().click();
                await page.waitForTimeout(3000);
            }
        } catch (e) {}
    } else if (currentUrl.includes('desk.inbodysupport.eu')) {
        console.log('[Zoho] Already logged in (redirected to Desk).');
    } else {
        console.log(`[Zoho] Unexpected URL: ${currentUrl}`);
    }

    await page.waitForLoadState('networkidle', { timeout: 30000 }).catch(() => {});
    console.log(`[Zoho] Final URL after login: ${page.url()}`);
}

async function zohoApi(page, path) {
    // Ensure we're on the Zoho Desk domain so cookies are sent
    if (!page.url().includes('desk.inbodysupport.eu')) {
        await page.goto('https://desk.inbodysupport.eu/agent/', { waitUntil: 'domcontentloaded', timeout: 30000 });
        await page.waitForTimeout(1000);
    }
    return page.evaluate(async ([path]) => {
        const sep = path.includes('?') ? '&' : '?';
        const resp = await fetch(path + sep + 'orgId=20078282365', { credentials: 'include' });
        if (!resp.ok) return { error: resp.status, body: await resp.text() };
        return resp.json();
    }, [path]);
}

// ─── DHL: Create Shipment ──────────────────────────────────────────────────
app.post('/api/dhl/create', (req, res) => {
    runScraper(req, res, async (page, data) => {
        const { username, password, url, weight, receiver_address, order_number } = data;
        const targetUrl = url || 'https://geschaeftskunden.dhl.de';

        await dhlLogin(page, targetUrl, username, password);

        await page.goto(`${targetUrl}/content/vls/vc/ShipmentDetails`, { waitUntil: 'networkidle' });

        await page.fill("input[id='receiver.name1']", receiver_address.name1 || '');
        await page.fill("input[id='receiver.street']", receiver_address.street || '');
        await page.fill("input[id='receiver.streetNumber']", receiver_address.house_number || '');
        await page.fill("input[id='receiver.plz']", receiver_address.zip || '');
        await page.fill("input[id='receiver.city']", receiver_address.city || '');

        const weightVal = parseFloat(weight) || 1.0;
        const weightStr = weightVal.toFixed(1).replace('.', ',');
        await page.fill("input[id='shipment-weight']", weightStr);

        await page.locator("button:has-text('Versenden'), button:has-text('Drucken')").first().click();
        await page.waitForLoadState('networkidle');

        const bodyText = await page.innerText('body');
        const match = bodyText.match(/(?:Sendungsnummer|Tracking|Paketnummer)[:\s]+(\d{10,20})/i);
        const tracking_number = match ? match[1] : `DHL-UNKNOWN-${order_number}`;
        if (!match) console.warn('[DHL] Tracking number not found in output.');

        return { tracking_number, raw_response: { status: 'created', provider: 'dhl', text_snippet: bodyText.substring(0, 200) } };
    });
});

// ─── DHL: Fetch Recent Shipments (CSV export) ──────────────────────────────
app.post('/api/dhl/fetch', (req, res) => {
    runScraper(req, res, async (page, data) => {
        // Support _from_env: use server-side env vars when credentials not provided
        const username = data.username || (data._from_env ? process.env.DHL_USERNAME : '') || '';
        const password = data.password || (data._from_env ? process.env.DHL_PASSWORD : '') || '';
        const targetUrl = data.url || process.env.DHL_URL || 'https://geschaeftskunden.dhl.de';

        await dhlLogin(page, targetUrl, username, password);

        console.log('[DHL] Navigating to shipment list...');
        await page.goto(`${targetUrl}/content/scc/shipmentlist`, { waitUntil: 'load', timeout: 60000 });
        await page.waitForTimeout(5000);
        // Dismiss OneTrust overlay that reappears after SSO redirect
        await dismissOnetrust(page);

        // Switch to content iframe
        let contentFrame = page.mainFrame();
        try {
            await page.waitForSelector('iframe[src*="shipmentlist"]', { timeout: 10000 });
            const el = await page.$('iframe[src*="shipmentlist"]');
            if (el) contentFrame = await el.contentFrame();
        } catch (e) {
            const frames = page.frames();
            for (const f of frames) {
                if (f.url().includes('shipmentlist') && f.url() !== page.url()) { contentFrame = f; break; }
            }
        }

        await page.waitForTimeout(3000);

        // Load shipment list (using JS click to bypass any overlay)
        const loadBtn = contentFrame.locator('button:has-text("Sendungsliste laden")');
        if (await loadBtn.count() > 0) {
            await loadBtn.evaluate(btn => btn.click());
            console.log('[DHL] Clicked "Sendungsliste laden"...');
            console.log('[DHL] Clicked "Sendungsliste laden", waiting for data...');
            await page.waitForTimeout(10000);
        }

        // Export CSV
        const csvBtn = contentFrame.locator('button:has-text("Sendungsliste als CSV exportieren"), button.btn-primary:has-text("CSV")').first();
        if (await csvBtn.count() === 0) throw new Error('CSV export button not found');

        const csvPath = path.join(__dirname, '../data/dhl-shipments.csv');
        await fs.mkdir(path.dirname(csvPath), { recursive: true });

        const [download] = await Promise.all([
            page.waitForEvent('download', { timeout: 30000 }),
            csvBtn.click()
        ]);
        await download.saveAs(csvPath);
        console.log('[DHL] CSV saved');

        const csvContent = await fs.readFile(csvPath, 'utf-8');
        const shipments = parseDhlCsv(csvContent);
        console.log(`[DHL] Parsed ${shipments.length} shipments`);

        return { success: true, count: shipments.length, shipments };
    });
});

function parseDhlCsv(csvContent) {
    const lines = csvContent.replace(/\r/g, '').split('\n').filter(l => l.trim());
    if (lines.length < 2) return [];

    const headers = lines[0].split(';').map(h => h.trim());
    const headerMap = {
        'Sendungsnummer': 'tracking_number',
        'Sendungsreferenz': 'reference',
        'internationale Sendungsnummer': 'international_number',
        'Abrechnungsnummer': 'billing_number',
        'Empfängername': 'recipient_name',
        'Empfängerstraße (inkl. Hausnummer)': 'recipient_street',
        'Empfänger-PLZ': 'recipient_zip',
        'Empfänger-Ort': 'recipient_city',
        'Empfänger-Land': 'recipient_country',
        'Status': 'status',
        'Datum Status': 'status_date',
        'Hinweis': 'note',
        'Zugestellt an - Name': 'delivered_to_name',
        'Zugestellt an - Straße (inkl. Hausnummer)': 'delivered_to_street',
        'Zugestellt an - PLZ': 'delivered_to_zip',
        'Zugestellt an - Ort': 'delivered_to_city',
        'Zugestellt an - Land': 'delivered_to_country',
        'Produkt': 'product',
        'Services': 'services',
    };

    const shipments = [];
    for (let i = 1; i < lines.length; i++) {
        const values = lines[i].split(';').map(v => v.trim());
        if (values.length < headers.length) continue;
        const obj = {};
        headers.forEach((h, idx) => {
            const key = headerMap[h] || h.toLowerCase().replace(/[^a-z0-9]/g, '_');
            obj[key] = values[idx] || '';
        });
        if (obj.tracking_number) shipments.push(obj);
    }
    return shipments;
}

// ─── OPAL: Create Shipment ─────────────────────────────────────────────────
app.post('/api/opal/create', (req, res) => {
    runScraper(req, res, async (page, data) => {
        const { username, password, url, weight, sender_address, receiver_address, ref_number, order_number } = data;
        const targetUrl = url || 'https://opal-kurier.de';

        await page.goto(targetUrl, { waitUntil: 'networkidle' });
        await page.fill("input[name='username']", username);
        await page.fill("input[type='password']", password);
        await page.locator("input[type='submit'], button[type='submit']").first().click();
        await page.waitForLoadState('networkidle');

        const topFrame = page.frame({ name: 'optop' });
        if (!topFrame) throw new Error("Frame 'optop' not found");
        await topFrame.locator("a:has-text('Neuer Auftrag'), a[href*='new']").first().click();
        await page.waitForTimeout(2000);

        const mainFrame = page.frame({ name: 'opmain' });
        if (!mainFrame) throw new Error("Frame 'opmain' not found");

        const fillArray = async (sel, idx, val) => {
            const locs = mainFrame.locator(sel);
            if (await locs.count() > idx) await locs.nth(idx).fill(val || '');
        };

        const senderStreet = `${sender_address.street || ''} ${sender_address.house_number || ''}`.trim();
        const receiverStreet = `${receiver_address.street || ''} ${receiver_address.house_number || ''}`.trim();

        await fillArray("input[name='address_name1[]']", 0, sender_address.name1);
        await fillArray("input[name='address_str[]']", 0, senderStreet);
        await fillArray("input[name='address_plz[]']", 0, sender_address.zip);
        await fillArray("input[name='address_ort[]']", 0, sender_address.city);
        await fillArray("input[name='address_name1[]']", 1, receiver_address.name1);
        await fillArray("input[name='address_str[]']", 1, receiverStreet);
        await fillArray("input[name='address_plz[]']", 1, receiver_address.zip);
        await fillArray("input[name='address_ort[]']", 1, receiver_address.city);

        const weightVal = parseFloat(weight) || 1.0;
        await mainFrame.fill("input#segewicht", weightVal.toFixed(1).replace('.', ','));
        await mainFrame.fill("input#seclref", ref_number || order_number);

        await mainFrame.locator("input[type='submit'], button[type='submit']").first().click();
        await page.waitForLoadState('networkidle');
        await page.waitForTimeout(2000);

        const bodyText = await mainFrame.innerText('body').catch(() => page.innerText('body'));
        const match = bodyText.match(/Sendungsnummer[:\s]*([A-Z0-9-]+)/i);
        const tracking_number = match ? match[1] : `OPAL-UNKNOWN-${order_number}`;
        if (!match) console.warn('[OPAL] Tracking number not found.');

        return { tracking_number, raw_response: { status: 'created', provider: 'opal', text_snippet: bodyText.substring(0, 200) } };
    });
});

// ─── OPAL: Fetch Recent Shipments (detail pages) ────────────────────────────
app.post('/api/opal/fetch', (req, res) => {
    runScraper(req, res, async (page, data) => {
        // Support _from_env: use server-side env vars when credentials not provided
        const username = data.username || (data._from_env ? process.env.OPAL_USERNAME : '') || '';
        const password = data.password || (data._from_env ? process.env.OPAL_PASSWORD : '') || '';
        const url = data.url;
        const limit = data.limit || 50;
        const targetUrl = url || process.env.OPAL_URL || 'https://opal-kurier.de';

        // Navigate and login
        await page.goto(targetUrl, { waitUntil: 'networkidle', timeout: 60000 });
        const hasLoginForm = await page.locator('input[type="password"]').count();
        if (hasLoginForm > 0) {
            await page.locator('input[name="username"], input[type="text"]').first().fill(username);
            await page.locator('input[type="password"]').first().fill(password);
            await page.locator('button[type="submit"], input[type="submit"]').first().click();
            await page.waitForLoadState('networkidle', { timeout: 30000 });
        }

        // Wait for frameset
        await page.waitForSelector('frameset, frame[name="optop"]', { timeout: 30000 });

        // Find optop frame
        const getFrame = async (name) => {
            for (let i = 0; i < 20; i++) {
                const f = page.frames().find(fr => fr.name() === name);
                if (f) return f;
                await page.waitForTimeout(500);
            }
            throw new Error(`Frame '${name}' not found`);
        };

        const headerFrame = await getFrame('optop');
        await headerFrame.waitForSelector('a', { timeout: 10000 });

        // Click Auftragsliste
        for (const sel of ['a:has-text("Auftragsliste")', 'a:has-text("Liste")', 'a[href*="list"]']) {
            try { await headerFrame.click(sel, { timeout: 5000 }); break; } catch (e) { /* try next */ }
        }
        await page.waitForTimeout(3000);

        const mainFrame = await getFrame('opmain');
        await mainFrame.waitForSelector('tr[onmouseover]', { timeout: 30000 });

        const orders = [];
        let currentRow = 0;

        while (orders.length < limit) {
            const rowCount = await mainFrame.evaluate(() =>
                document.querySelectorAll('tr[onmouseover]').length);

            if (currentRow >= rowCount) {
                // Try next page
                const hasNext = await mainFrame.evaluate(() => {
                    for (const el of document.querySelectorAll('a, td[onclick]')) {
                        if (el.textContent.trim() === '>') { el.click(); return true; }
                    }
                    return false;
                });
                if (!hasNext) break;
                await page.waitForTimeout(2000);
                currentRow = 0;
                continue;
            }

            const clicked = await mainFrame.evaluate((idx) => {
                const rows = document.querySelectorAll('tr[onmouseover]');
                const td = rows[idx]?.querySelector('td[onclick]');
                if (td) { td.click(); return true; }
                return false;
            }, currentRow);

            if (!clicked) { currentRow++; continue; }

            try {
                await mainFrame.waitForFunction(() =>
                    document.body.innerText.includes('SendungsNr') &&
                    document.body.innerText.includes('zur Liste zurück'),
                    { timeout: 10000 });
            } catch (e) { currentRow++; continue; }

            const text = await mainFrame.evaluate(() => document.body.innerText);
            if (!text.includes('zur Liste zurück')) { currentRow++; continue; }

            const order = parseOpalDetail(text);
            if (order.tracking_number || order.hwb_number) {
                orders.push(order);
                console.log(`[OPAL] Parsed: ${order.tracking_number} - ${order.delivery_name}`);
            }

            // Go back to list
            const wentBack = await mainFrame.evaluate(() => {
                for (const el of document.querySelectorAll('a, button')) {
                    if (el.textContent.includes('zur Liste')) { el.click(); return true; }
                }
                return false;
            });
            if (!wentBack) break;

            try {
                await mainFrame.waitForFunction(() =>
                    document.body.innerText.includes('Datensätzen') &&
                    !document.body.innerText.includes('SendungsNr'),
                    { timeout: 10000 });
            } catch (e) { await page.waitForTimeout(2000); }

            currentRow++;
        }

        console.log(`[OPAL] Fetched ${orders.length} orders`);
        return { success: true, count: orders.length, orders };
    });
});

function parseOpalDetail(text) {
    const order = {
        tracking_number: '', hwb_number: '', product_type: '', reference: '',
        created_at: '', created_by: '',

        pickup_name: '', pickup_name2: '', pickup_contact: '', pickup_phone: '',
        pickup_email: '', pickup_street: '', pickup_zip: '', pickup_city: '',
        pickup_country: 'DE', pickup_note: '', pickup_date: '',
        pickup_time_from: '', pickup_time_to: '', pickup_vehicle: '',

        delivery_name: '', delivery_name2: '', delivery_contact: '', delivery_phone: '',
        delivery_email: '', delivery_street: '', delivery_zip: '', delivery_city: '',
        delivery_country: 'DE', delivery_note: '', delivery_date: '',
        delivery_time_from: '', delivery_time_to: '',

        package_count: null, weight: null, value: null, description: '', dimensions: '',
        status: '', status_date: '', status_time: '', receiver: ''
    };

    const lines = text.split('\n').map(l => l.trim()).filter(l => l);

    // Parse SendungsNr, HWB, Auftragsart, Referenz
    for (const line of lines) {
        if (line.includes('SendungsNr')) {
            const m = line.match(/SendungsNr\s+(OCU[-\d]+)/);
            if (m) order.tracking_number = m[1];
        }
        if (line.includes('HWB') && !order.hwb_number) {
            const m = line.match(/HWB\s+(\d{12}|OCU-[\d-]+)/);
            if (m) order.hwb_number = m[1];
        }
        if (line.includes('Auftragsart')) {
            const m = line.match(/Auftragsart\s+(\S+)/);
            if (m) order.product_type = m[1];
        }
        if (line.includes('Referenz') && !line.includes('Ref/KST')) {
            const m = line.match(/Referenz\s+(\S+)/);
            if (m) order.reference = m[1];
        }
    }

    // Parse created info
    const createdMatch = text.match(/erfasst am\s+([\d\.\-\s:]+Uhr)/);
    if (createdMatch) order.created_at = createdMatch[1].trim();
    const createdByMatch = text.match(/erfasst durch\s+(\S+)/);
    if (createdByMatch) order.created_by = createdByMatch[1];

    // Parse Abholung section
    const abholungIdx = lines.findIndex(l => l === 'Abholung');
    if (abholungIdx >= 0) {
        for (let i = abholungIdx + 1; i < Math.min(abholungIdx + 15, lines.length); i++) {
            const line = lines[i];
            if (line.startsWith('Name1')) order.pickup_name = line.replace('Name1', '').trim();
            if (line.startsWith('Name2')) order.pickup_name2 = line.replace('Name2', '').trim();
            if (line.startsWith('Ansprechpartner')) order.pickup_contact = line.replace('Ansprechpartner', '').trim();
            if (line.startsWith('Telefon')) order.pickup_phone = line.replace('Telefon', '').trim();
            if (line.startsWith('Mail')) order.pickup_email = line.replace('Mail', '').trim();
            if (line.startsWith('Straße/Hs')) order.pickup_street = line.replace('Straße/Hs', '').trim();
            if (line.startsWith('LKZ-Land')) {
                const addr = line.replace('LKZ-Land', '').trim();
                const m = addr.match(/([A-Z]{2})-(\d{4,5})\s+(.+)/);
                if (m) { order.pickup_country = m[1]; order.pickup_zip = m[2]; order.pickup_city = m[3]; }
            }
            if (line.startsWith('Hinweis') && !order.pickup_note) order.pickup_note = line.replace('Hinweis', '').trim();
            if (line === 'Zustellung') break;
        }
    }

    // Parse Zustellung section
    const zustellungIdx = lines.findIndex(l => l === 'Zustellung');
    if (zustellungIdx >= 0) {
        for (let i = zustellungIdx + 1; i < Math.min(zustellungIdx + 15, lines.length); i++) {
            const line = lines[i];
            if (line.startsWith('Name1')) order.delivery_name = line.replace('Name1', '').trim();
            if (line.startsWith('Name2')) order.delivery_name2 = line.replace('Name2', '').trim();
            if (line.startsWith('Ansprechpartner')) order.delivery_contact = line.replace('Ansprechpartner', '').trim();
            if (line.startsWith('Telefon')) order.delivery_phone = line.replace('Telefon', '').trim();
            if (line.startsWith('Mail')) order.delivery_email = line.replace('Mail', '').trim();
            if (line.startsWith('Straße/Hs')) order.delivery_street = line.replace('Straße/Hs', '').trim();
            if (line.startsWith('LKZ-Land')) {
                const addr = line.replace('LKZ-Land', '').trim();
                const m = addr.match(/([A-Z]{2})-(\d{4,5})\s+(.+)/);
                if (m) { order.delivery_country = m[1]; order.delivery_zip = m[2]; order.delivery_city = m[3]; }
            }
            if (line.startsWith('Hinweis') && !order.delivery_note) order.delivery_note = line.replace('Hinweis', '').trim();
            if (line.includes('Abholtermin') || line.includes('Frühtermine')) break;
        }
    }

    // Parse pickup date/time
    const abholTerminIdx = lines.findIndex(l => l.includes('Abholtermin'));
    if (abholTerminIdx >= 0) {
        for (let i = abholTerminIdx + 1; i < Math.min(abholTerminIdx + 5, lines.length); i++) {
            const line = lines[i];
            const dateMatch = line.match(/(\d{2}\.\d{2}\.\d{4})/);
            if (dateMatch) order.pickup_date = dateMatch[1];
            const timeMatch = line.match(/Zeit\s+(\d{2}:\d{2})\s+-\s+(\d{2}:\d{2})/);
            if (timeMatch) { order.pickup_time_from = timeMatch[1]; order.pickup_time_to = timeMatch[2]; }
            if (line.includes('Fahrzeug')) {
                const vehicleMatch = line.match(/Fahrzeug\s+(\S+)/);
                if (vehicleMatch) order.pickup_vehicle = vehicleMatch[1];
            }
            if (line.includes('Zustelltermin')) break;
        }
    }

    // Parse delivery date/time
    const zustellTerminIdx = lines.findIndex(l => l.includes('Zustelltermin'));
    if (zustellTerminIdx >= 0) {
        for (let i = zustellTerminIdx + 1; i < Math.min(zustellTerminIdx + 3, lines.length); i++) {
            const line = lines[i];
            const dateMatch = line.match(/(\d{2}\.\d{2}\.\d{4})/);
            if (dateMatch) order.delivery_date = dateMatch[1];
            const timeMatch = line.match(/Zeit\s+(\d{2}:\d{2})\s+-\s+(\d{2}:\d{2})/);
            if (timeMatch) { order.delivery_time_from = timeMatch[1]; order.delivery_time_to = timeMatch[2]; }
            if (line.includes('Sendung & Pack')) break;
        }
    }

    // Parse package value
    const wertMatch = text.match(/Wert\s+([\d\.,]+)\s*EUR/);
    if (wertMatch) order.value = parseFloat(wertMatch[1].replace('.', '').replace(',', '.'));

    // Parse weight, package count, description
    const weightMatch = text.match(/(\d+)\s+([\d,]+)\s+([A-Za-z_][\w\s]+?)(?:\s+VolG|$)/m);
    if (weightMatch) {
        order.package_count = parseInt(weightMatch[1]);
        order.weight = parseFloat(weightMatch[2].replace(',', '.'));
        order.description = weightMatch[3].trim();
    }

    // Parse dimensions
    const dimMatch = text.match(/L:\s*([\d,]+)\s*B:\s*([\d,]+)\s*H:\s*([\d,]+)/);
    if (dimMatch) order.dimensions = `${dimMatch[1]}x${dimMatch[2]}x${dimMatch[3]}`;

    // Parse status
    const statusMatch = text.match(/(\d{12}|OCU-[\d-]+)\s+(\d{2}\.\d{2}\.\d{2})\s+(\d{2}:\d{2})\s+(Zugestellt|Abgeholt|Storniert|AKTIV|geliefert|ausgeliefert|Fehlanfahrt)\s*(\S*)/i);
    if (statusMatch) {
        order.status = statusMatch[4];
        order.status_date = statusMatch[2];
        order.status_time = statusMatch[3];
        order.receiver = statusMatch[5] || '';
    }

    return order;
}

// ─── EXACT ONLINE: Fetch Inventory (Stub) ─────────────────────────────────
app.post('/api/exact/inventory/fetch', (req, res) => {
    runScraper(req, res, async (page, data) => {
        const username = data.username || (data._from_env ? process.env.EXACT_USERNAME : '') || '';
        const password = data.password || (data._from_env ? process.env.EXACT_PASSWORD : '') || '';
        const targetUrl = data.url || process.env.EXACT_URL || 'https://start.exactonline.de';

        await exactLogin(page, targetUrl, username, password);

        // TODO: Navigate to inventory/stock view and parse table
        const pageText = await page.evaluate(() => document.body?.innerText?.substring(0, 500) || 'EMPTY');

        return {
            success: true,
            message: 'Logged in successfully. Navigation paths for inventory pending.',
            current_url: page.url(),
            text_preview: pageText
        };
    });
});

// ─── EXACT ONLINE: Create Quotation / Kostenvoranschlag (Stub) ─────────────
app.post('/api/exact/quotation/create', (req, res) => {
    runScraper(req, res, async (page, data) => {
        const username = data.username || (data._from_env ? process.env.EXACT_USERNAME : '') || '';
        const password = data.password || (data._from_env ? process.env.EXACT_PASSWORD : '') || '';
        const targetUrl = data.url || process.env.EXACT_URL || 'https://start.exactonline.de';

        await exactLogin(page, targetUrl, username, password);

        // TODO: Navigate to Sales -> Quotations -> New and fill form
        return {
            success: true,
            message: 'Logged in successfully. Navigation paths to create Kostenvoranschlag pending.',
            current_url: page.url()
        };
    });
});

// ─── ZOHO DESK: Fetch Tickets ─────────────────────────────────────────────
const ZOHO_BASE = '/supportapi/zd/inbodyeu/api/v1';
const ZOHO_ORG  = '20078282365';
const ZOHO_DEPT = '53451000019414029';

app.post('/api/zoho/tickets', (req, res) => {
    runScraper(req, res, async (page, data) => {
        const username  = data.username || (data._from_env ? process.env.ZOHO_EMAIL    : '') || '';
        const password  = data.password || (data._from_env ? process.env.ZOHO_PASSWORD : '') || '';
        const targetUrl = data.url || process.env.ZOHO_URL || 'https://desk.inbodysupport.eu/agent/';
        const limit     = Math.min(data.limit || 50, 200);

        if (!username || !password) {
            throw new Error('ZOHO_EMAIL or ZOHO_PASSWORD missing. Provide in body or .env');
        }

        await zohoLogin(page, targetUrl, username, password);

        // Ensure we're on the agent page so session cookies are in scope
        if (!page.url().includes('desk.inbodysupport.eu')) {
            await page.goto(targetUrl, { waitUntil: 'domcontentloaded', timeout: 30000 });
            await page.waitForLoadState('networkidle', { timeout: 20000 }).catch(() => {});
        }

        // Use browser's session cookies to call the internal Zoho API
        const url = `${ZOHO_BASE}/tickets?include=contacts,assignee,departments&from=0&limit=${limit}&sortBy=-modifiedTime&departmentId=${ZOHO_DEPT}&orgId=${ZOHO_ORG}`;
        const resp = await page.evaluate(async (url) => {
            const r = await fetch(url, { credentials: 'include' });
            if (!r.ok) return { error: r.status, body: await r.text() };
            return r.json();
        }, url);

        if (resp.error) {
            throw new Error(`Zoho API error ${resp.error}: ${resp.body}`);
        }

        const tickets = resp.data || [];
        console.log(`[Zoho] Fetched ${tickets.length} tickets`);

        return {
            success: true,
            count: tickets.length,
            tickets,
        };
    });
});

// ─── ZOHO DESK: Fetch Ticket Email Threads ────────────────────────────────────
app.post('/api/zoho/ticket-threads', (req, res) => {
    runScraper(req, res, async (page, data) => {
        const username  = data.username || (data._from_env ? process.env.ZOHO_EMAIL    : '') || '';
        const password  = data.password || (data._from_env ? process.env.ZOHO_PASSWORD : '') || '';
        const targetUrl = data.url || process.env.ZOHO_URL || 'https://desk.inbodysupport.eu/agent/';
        const ticketId  = data.ticketId;

        if (!ticketId) throw new Error('ticketId is required');
        if (!username || !password) throw new Error('ZOHO_EMAIL or ZOHO_PASSWORD missing. Provide in body or .env');

        await zohoLogin(page, targetUrl, username, password);

        if (!page.url().includes('desk.inbodysupport.eu')) {
            await page.goto(targetUrl, { waitUntil: 'domcontentloaded', timeout: 30000 });
            await page.waitForLoadState('networkidle', { timeout: 20000 }).catch(() => {});
        }

        // Fetch ticket metadata (subject, contact, status)
        const ticketRes = await zohoApi(page, `${ZOHO_BASE}/tickets/${ticketId}?include=contacts`);
        const ticket = ticketRes.error ? null : ticketRes;

        const threadsRes = await zohoApi(page, `${ZOHO_BASE}/tickets/${ticketId}/threads`);
        if (threadsRes.error) throw new Error(`Zoho threads API error ${threadsRes.error}: ${threadsRes.body}`);

        const threads = threadsRes.data || [];

        // Fetch full content + attachments for each thread (list API returns truncated content)
        for (const thread of threads) {
            const fullThread = await zohoApi(page, `${ZOHO_BASE}/tickets/${ticketId}/threads/${thread.id}`);
            if (!fullThread.error && fullThread.content) {
                thread.content = fullThread.content;
                thread.summary = fullThread.summary || thread.summary;
            }
            // Attachments: try from individual thread response, then ticket-level API
            let attArr = [];
            if (thread.hasAttach || parseInt(thread.attachmentCount) > 0) {
                // Individual thread response may include attachments directly
                if (!fullThread.error && Array.isArray(fullThread.attachments) && fullThread.attachments.length > 0) {
                    attArr = fullThread.attachments;
                    console.log(`[Zoho] Thread ${thread.id}: got ${attArr.length} attachments from thread response`);
                }
                // Fallback: ticket-level attachments endpoint
                if (attArr.length === 0) {
                    const ticketAttRes = await zohoApi(page, `${ZOHO_BASE}/tickets/${ticketId}/attachments`);
                    if (!ticketAttRes.error) {
                        attArr = Array.isArray(ticketAttRes.data) ? ticketAttRes.data : (Array.isArray(ticketAttRes) ? ticketAttRes : []);
                    }
                    if (attArr.length > 0) console.log(`[Zoho] Thread ${thread.id}: got ${attArr.length} attachments from ticket API`);
                }
                if (attArr.length === 0) {
                    // Log full individual thread response keys for debugging
                    console.log(`[Zoho] Thread ${thread.id}: no attachments found. fullThread keys:`, !fullThread.error ? Object.keys(fullThread) : 'error');
                }
            }
            thread.attachments = attArr;
        }

        console.log(`[Zoho] Fetched ${threads.length} threads for ticket ${ticketId}`);
        return { success: true, ticketId, count: threads.length, threads, ticket };
    });
});

// ─── ZOHO DESK: Bulk Fetch Threads for Multiple Tickets ───────────────────────
// Single browser session, one login, fetches threads for all provided ticket IDs.
app.post('/api/zoho/ticket-threads-bulk', (req, res) => {
    runScraper(req, res, async (page, data) => {
        const username  = data.username || (data._from_env ? process.env.ZOHO_EMAIL    : '') || '';
        const password  = data.password || (data._from_env ? process.env.ZOHO_PASSWORD : '') || '';
        const targetUrl = data.url || process.env.ZOHO_URL || 'https://desk.inbodysupport.eu/agent/';
        const ticketIds = data.ticketIds;

        if (!Array.isArray(ticketIds) || ticketIds.length === 0) throw new Error('ticketIds[] is required');
        if (!username || !password) throw new Error('ZOHO_EMAIL or ZOHO_PASSWORD missing');

        // Single login for all tickets
        await zohoLogin(page, targetUrl, username, password);
        if (!page.url().includes('desk.inbodysupport.eu')) {
            await page.goto(targetUrl, { waitUntil: 'domcontentloaded', timeout: 30000 });
            await page.waitForLoadState('networkidle', { timeout: 20000 }).catch(() => {});
        }

        const results = [];
        for (let i = 0; i < ticketIds.length; i++) {
            const ticketId = ticketIds[i];
            // Pause between tickets to avoid Zoho rate limiting
            if (i > 0) await page.waitForTimeout(1500);
            try {
                const ticketRes = await zohoApi(page, `${ZOHO_BASE}/tickets/${ticketId}?include=contacts`);
                const ticket = ticketRes.error ? null : ticketRes;

                const threadsRes = await zohoApi(page, `${ZOHO_BASE}/tickets/${ticketId}/threads`);
                if (threadsRes.error) {
                    results.push({ ticketId, success: false, error: `API ${threadsRes.error}` });
                    continue;
                }

                const threads = threadsRes.data || [];
                for (const thread of threads) {
                    const fullThread = await zohoApi(page, `${ZOHO_BASE}/tickets/${ticketId}/threads/${thread.id}`);
                    if (!fullThread.error && fullThread.content) {
                        thread.content = fullThread.content;
                        thread.summary = fullThread.summary || thread.summary;
                    }
                    // Attachments from individual thread response
                    let attArr = [];
                    if (thread.hasAttach || parseInt(thread.attachmentCount) > 0) {
                        if (!fullThread.error && Array.isArray(fullThread.attachments) && fullThread.attachments.length > 0) {
                            attArr = fullThread.attachments;
                        }
                    }
                    thread.attachments = attArr;
                }

                console.log(`[Zoho] Bulk: ${threads.length} threads for ticket ${ticketId} (${i+1}/${ticketIds.length})`);
                results.push({ ticketId, success: true, count: threads.length, threads, ticket });
            } catch (e) {
                console.error(`[Zoho] Bulk: error for ticket ${ticketId}:`, e.message);
                results.push({ ticketId, success: false, error: e.message });
            }
        }

        return { success: true, results };
    });
});

// ─── ZOHO DESK: Download Attachment (using browser session cookies) ───────────
// Takes a Zoho attachment href, logs in, fetches the file buffer via page.evaluate,
// and returns it as base64 + mimeType so the Rust server can save it to CAS.
app.post('/api/zoho/download-attachment', (req, res) => {
    runScraper(req, res, async (page, data) => {
        const username  = data.username || (data._from_env ? process.env.ZOHO_EMAIL    : '') || '';
        const password  = data.password || (data._from_env ? process.env.ZOHO_PASSWORD : '') || '';
        const targetUrl = data.url || process.env.ZOHO_URL || 'https://desk.inbodysupport.eu/agent/';
        const href      = data.href;
        const fileName  = data.fileName || 'attachment';

        if (!href) throw new Error('href is required');
        if (!username || !password) throw new Error('ZOHO_EMAIL or ZOHO_PASSWORD missing. Provide in body or .env');

        await zohoLogin(page, targetUrl, username, password);

        // Ensure we're on the Zoho Desk domain so cookies are sent
        if (!page.url().includes('desk.inbodysupport.eu')) {
            await page.goto(targetUrl, { waitUntil: 'domcontentloaded', timeout: 30000 });
            await page.waitForLoadState('networkidle', { timeout: 20000 }).catch(() => {});
        }

        // Use browser session cookies to fetch the file buffer
        // Append orgId (required by Zoho API) just like zohoApi() does
        const sep = href.includes('?') ? '&' : '?';
        const fullHref = href + sep + 'orgId=20078282365';
        const result = await page.evaluate(async ([url]) => {
            const resp = await fetch(url, { credentials: 'include' });
            if (!resp.ok) return { error: resp.status, message: await resp.text().catch(() => '') };
            const mimeType = resp.headers.get('content-type') || 'application/octet-stream';
            const buffer   = await resp.arrayBuffer();
            // Convert ArrayBuffer → base64 in chunks to avoid call-stack limits
            const bytes = new Uint8Array(buffer);
            let binary = '';
            const chunk = 8192;
            for (let i = 0; i < bytes.length; i += chunk) {
                binary += String.fromCharCode(...bytes.subarray(i, i + chunk));
            }
            return { base64: btoa(binary), mimeType, size: bytes.length };
        }, [fullHref]);

        if (result.error) throw new Error(`Download failed HTTP ${result.error}: ${result.message}`);

        console.log(`[Zoho] Downloaded attachment: ${fileName} (${result.mimeType}, ${result.size} bytes)`);
        return { success: true, base64: result.base64, mimeType: result.mimeType, fileName };
    });
});

// ─── Debug / Info ─────────────────────────────────────────────────────────────

// GET /debug — shows scraper info and available endpoints.
// Useful for checking if service is running and seeing all routes.
app.get('/debug', (req, res) => {
    res.json({
        service: 'eck-playwright-scraper',
        port: PORT,
        status: 'running',
        debug_mode_hint: 'Add "debug": true to POST body to run browser in headed (visible) mode with 600ms slowMo',
        endpoints: [
            { method: 'GET',  path: '/',                  desc: 'HTML status page' },
            { method: 'POST', path: '/api/opal/create',   desc: 'Create OPAL shipment' },
            { method: 'POST', path: '/api/opal/fetch',    desc: 'Fetch OPAL shipment list (supports debug:true)' },
            { method: 'POST', path: '/api/dhl/create',    desc: 'Create DHL shipment' },
            { method: 'POST', path: '/api/dhl/fetch',     desc: 'Fetch DHL shipment list via CSV (supports debug:true)' },
            { method: 'POST', path: '/api/exact/inventory/fetch',  desc: 'Exact Online: fetch inventory (stub)' },
            { method: 'POST', path: '/api/exact/quotation/create', desc: 'Exact Online: create Kostenvoranschlag (stub)' },
            { method: 'POST', path: '/api/zoho/tickets',               desc: 'Zoho Desk: login and fetch tickets' },
            { method: 'POST', path: '/api/zoho/ticket-threads',       desc: 'Zoho Desk: fetch ticket email threads with HTML content and attachments' },
            { method: 'POST', path: '/api/zoho/download-attachment',  desc: 'Zoho Desk: download a single attachment as base64 using session cookies' },
            { method: 'GET',  path: '/debug',                         desc: 'This page' },
        ]
    });
});

const PORT = process.env.PORT || 3211;
const MAIN_HEALTH = process.env.MAIN_SERVER_URL
    ? `${process.env.MAIN_SERVER_URL}/E/health`
    : 'http://127.0.0.1:3210/E/health';

// Wait for the main eckwmsr server to be available before starting.
// The main server also owns the embedded PostgreSQL — no point running without it.
async function waitForMainServer(maxMs = 60000) {
    const start = Date.now();
    console.log(`[Startup] Waiting for main server at ${MAIN_HEALTH} ...`);
    while (Date.now() - start < maxMs) {
        try {
            const res = await fetch(MAIN_HEALTH);
            if (res.ok) { console.log('[Startup] Main server is up.'); return true; }
        } catch (_) {}
        await new Promise(r => setTimeout(r, 2000));
    }
    return false;
}

// Periodic health check — exits if main server is unreachable for too long.
function startHealthMonitor() {
    const INTERVAL_MS  = 30_000; // check every 30s
    const MAX_FAILURES = 3;      // exit after 3 consecutive failures (~90s)
    let failures = 0;

    setInterval(async () => {
        try {
            const res = await fetch(MAIN_HEALTH, { signal: AbortSignal.timeout(5000) });
            if (res.ok) { failures = 0; return; }
        } catch (_) {}
        failures++;
        console.warn(`[Health] Main server unreachable (${failures}/${MAX_FAILURES})`);
        if (failures >= MAX_FAILURES) {
            console.error('[Health] Main server unavailable. Shutting down scraper.');
            process.exit(1);
        }
    }, INTERVAL_MS);
}

waitForMainServer(60_000).then(ok => {
    if (!ok) {
        console.error('[Startup] Main server did not respond within 60s. Exiting.');
        process.exit(1);
    }
    app.listen(PORT, () => {
        console.log(`Eck Playwright Scraper Service running on port ${PORT}`);
        console.log(`  Debug info: http://localhost:${PORT}/debug`);
        console.log(`  Add "debug": true to any POST body to see the browser window`);
    });
    startHealthMonitor();
});
