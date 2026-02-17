const express = require('express');
const { chromium } = require('playwright');
const fs = require('fs').promises;
const path = require('path');
const app = express();

app.use(express.json());

// Helper to launch browser, run logic, and close
async function runScraper(req, res, logicFn) {
    let browser;
    try {
        browser = await chromium.launch({
            headless: true,
            args: ['--no-sandbox', '--disable-setuid-sandbox', '--disable-blink-features=AutomationControlled']
        });
        const context = await browser.newContext({
            viewport: { width: 1920, height: 1080 },
            userAgent: 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36',
            locale: 'de-DE',
            timezoneId: 'Europe/Berlin',
            acceptDownloads: true
        });
        const page = await context.newPage();
        const result = await logicFn(page, req.body);
        res.json(result);
    } catch (error) {
        console.error("Scraper Error:", error);
        res.status(500).json({ error: error.message });
    } finally {
        if (browser) await browser.close();
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
        const { username, password, url } = data;
        const targetUrl = url || 'https://geschaeftskunden.dhl.de';

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
        'Empfängername': 'recipient_name',
        'Empfängerstraße (inkl. Hausnummer)': 'recipient_street',
        'Empfänger-PLZ': 'recipient_zip',
        'Empfänger-Ort': 'recipient_city',
        'Empfänger-Land': 'recipient_country',
        'Status': 'status',
        'Datum Status': 'status_date',
        'Produkt': 'product',
    };

    return lines.slice(1).map(line => {
        const values = line.split(';').map(v => v.trim());
        const obj = {};
        headers.forEach((h, i) => {
            const key = headerMap[h] || h.toLowerCase().replace(/[^a-z0-9]/g, '_');
            obj[key] = values[i] || '';
        });
        return obj;
    }).filter(s => s.tracking_number);
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
        const { username, password, url, limit = 50 } = data;
        const targetUrl = url || 'https://opal-kurier.de';

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
        pickup_name: '', pickup_street: '', pickup_zip: '', pickup_city: '', pickup_country: 'DE',
        delivery_name: '', delivery_street: '', delivery_zip: '', delivery_city: '', delivery_country: 'DE',
        weight: null, status: '', status_date: '', status_time: ''
    };
    const lines = text.split('\n').map(l => l.trim()).filter(l => l);

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
    }

    const parseSection = (sectionName) => {
        const idx = lines.findIndex(l => l === sectionName);
        if (idx < 0) return {};
        const result = {};
        for (let i = idx + 1; i < Math.min(idx + 15, lines.length); i++) {
            const l = lines[i];
            if (l.startsWith('Name1')) result.name = l.replace('Name1', '').trim();
            if (l.startsWith('Straße/Hs')) result.street = l.replace('Straße/Hs', '').trim();
            if (l.startsWith('LKZ-Land')) {
                const m = l.replace('LKZ-Land', '').trim().match(/([A-Z]{2})-(\d{4,5})\s+(.+)/);
                if (m) { result.country = m[1]; result.zip = m[2]; result.city = m[3]; }
            }
            if (l === 'Zustellung' || l === 'Abholtermin') break;
        }
        return result;
    };

    const pickup = parseSection('Abholung');
    const delivery = parseSection('Zustellung');
    Object.assign(order, {
        pickup_name: pickup.name || '', pickup_street: pickup.street || '',
        pickup_zip: pickup.zip || '', pickup_city: pickup.city || '',
        pickup_country: pickup.country || 'DE',
        delivery_name: delivery.name || '', delivery_street: delivery.street || '',
        delivery_zip: delivery.zip || '', delivery_city: delivery.city || '',
        delivery_country: delivery.country || 'DE',
    });

    const weightMatch = text.match(/(\d+)\s+([\d,]+)\s+/m);
    if (weightMatch) order.weight = parseFloat(weightMatch[2].replace(',', '.'));

    const statusMatch = text.match(/(\d{12}|OCU-[\d-]+)\s+(\d{2}\.\d{2}\.\d{2})\s+(\d{2}:\d{2})\s+(Zugestellt|Abgeholt|Storniert|AKTIV|geliefert|ausgeliefert|Fehlanfahrt)/i);
    if (statusMatch) { order.status = statusMatch[4]; order.status_date = statusMatch[2]; order.status_time = statusMatch[3]; }

    return order;
}

const PORT = process.env.PORT || 3211;
app.listen(PORT, () => {
    console.log(`Eck Playwright Scraper Service running on port ${PORT}`);
});
