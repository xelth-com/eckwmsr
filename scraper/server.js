const express = require('express');
const { chromium } = require('playwright');
const app = express();

app.use(express.json());

// Helper to launch browser, run logic, and close
async function runScraper(req, res, logicFn) {
    let browser;
    try {
        browser = await chromium.launch({ headless: true });
        const context = await browser.newContext({
            viewport: { width: 1920, height: 1080 },
            userAgent: 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) Chrome/120.0.0.0 Safari/537.36'
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

// DHL Route
app.post('/api/dhl/create', (req, res) => {
    runScraper(req, res, async (page, data) => {
        const { username, password, url, weight, receiver_address, order_number } = data;
        const targetUrl = url || 'https://geschaeftskunden.dhl.de';

        console.log(`[DHL] Navigating to ${targetUrl}`);
        await page.goto(targetUrl, { waitUntil: 'domcontentloaded' });

        // Accept cookies if present
        try {
            const cookieBtn = page.locator('#onetrust-accept-btn-handler').first();
            await cookieBtn.waitFor({ state: 'visible', timeout: 3000 });
            await cookieBtn.click();
        } catch (e) { /* no cookie banner */ }

        // Click login trigger if needed
        try {
            const loginTrigger = page.locator("button:has-text('Anmelden')").first();
            await loginTrigger.waitFor({ state: 'visible', timeout: 3000 });
            await loginTrigger.click();
        } catch (e) { /* no trigger */ }

        // Login
        await page.fill("input[type='email']", username);
        await page.fill("input[type='password']", password);
        await page.locator("button[type='submit']").first().click();
        await page.waitForNavigation({ waitUntil: 'networkidle' });

        // Navigate to Shipment Details
        await page.goto(`${targetUrl}/content/vls/vc/ShipmentDetails`, { waitUntil: 'networkidle' });

        // Fill receiver address
        await page.fill("input[id='receiver.name1']", receiver_address.name1 || '');
        await page.fill("input[id='receiver.street']", receiver_address.street || '');
        await page.fill("input[id='receiver.streetNumber']", receiver_address.house_number || '');
        await page.fill("input[id='receiver.plz']", receiver_address.zip || '');
        await page.fill("input[id='receiver.city']", receiver_address.city || '');

        // Fill weight (German decimal comma, strictly formatted to 1 decimal place)
        const weightVal = parseFloat(weight) || 1.0;
        const weightStr = weightVal.toFixed(1).replace('.', ',');
        await page.fill("input[id='shipment-weight']", weightStr);

        // Submit
        const submitBtn = page.locator("button:has-text('Versenden'), button:has-text('Drucken')").first();
        await submitBtn.click();
        await page.waitForLoadState('networkidle');

        // Extract tracking number
        const bodyText = await page.innerText('body');
        const match = bodyText.match(/(?:Sendungsnummer|Tracking|Paketnummer)[:\s]+(\d{10,20})/i);
        const tracking_number = match ? match[1] : `DHL-UNKNOWN-${order_number}`;

        if (!match) console.warn("[DHL] Tracking number not found in output.");

        return {
            tracking_number,
            raw_response: { status: "created", provider: "dhl", text_snippet: bodyText.substring(0, 200) }
        };
    });
});

// OPAL Route
app.post('/api/opal/create', (req, res) => {
    runScraper(req, res, async (page, data) => {
        const { username, password, url, weight, sender_address, receiver_address, ref_number, order_number } = data;
        const targetUrl = url || 'https://opal-kurier.de';

        console.log(`[OPAL] Navigating to ${targetUrl}`);
        await page.goto(targetUrl, { waitUntil: 'networkidle' });

        // Login
        await page.fill("input[name='username']", username);
        await page.fill("input[type='password']", password);
        await page.locator("input[type='submit'], button[type='submit']").first().click();
        await page.waitForLoadState('networkidle');

        // Click "Neuer Auftrag" in the top frame
        const topFrame = page.frame({ name: 'optop' });
        if (!topFrame) throw new Error("Frame 'optop' not found");
        const newOrderLink = topFrame.locator("a:has-text('Neuer Auftrag'), a[href*='new']").first();
        await newOrderLink.click();

        await page.waitForTimeout(2000);

        // Fill form in the main content frame
        const mainFrame = page.frame({ name: 'opmain' });
        if (!mainFrame) throw new Error("Frame 'opmain' not found");

        const fillArrayField = async (selector, idx, val) => {
            const locators = mainFrame.locator(selector);
            if (await locators.count() > idx) {
                await locators.nth(idx).fill(val || '');
            }
        };

        // Combine street and house number into one field for OPAL
        const senderStreet = `${sender_address.street || ''} ${sender_address.house_number || ''}`.trim();
        const receiverStreet = `${receiver_address.street || ''} ${receiver_address.house_number || ''}`.trim();

        // Sender (index 0)
        await fillArrayField("input[name='address_name1[]']", 0, sender_address.name1);
        await fillArrayField("input[name='address_str[]']", 0, senderStreet);
        await fillArrayField("input[name='address_plz[]']", 0, sender_address.zip);
        await fillArrayField("input[name='address_ort[]']", 0, sender_address.city);

        // Receiver (index 1)
        await fillArrayField("input[name='address_name1[]']", 1, receiver_address.name1);
        await fillArrayField("input[name='address_str[]']", 1, receiverStreet);
        await fillArrayField("input[name='address_plz[]']", 1, receiver_address.zip);
        await fillArrayField("input[name='address_ort[]']", 1, receiver_address.city);

        // Package details
        const weightVal = parseFloat(weight) || 1.0;
        const weightStr = weightVal.toFixed(1).replace('.', ',');
        await mainFrame.fill("input#segewicht", weightStr);
        await mainFrame.fill("input#seclref", ref_number || order_number);

        // Submit
        await mainFrame.locator("input[type='submit'], button[type='submit']").first().click();
        await page.waitForLoadState('networkidle');
        await page.waitForTimeout(2000);

        const bodyText = await mainFrame.innerText('body').catch(() => page.innerText('body'));
        const match = bodyText.match(/Sendungsnummer[:\s]*([A-Z0-9-]+)/i);
        const tracking_number = match ? match[1] : `OPAL-UNKNOWN-${order_number}`;

        if (!match) console.warn("[OPAL] Tracking number not found in output.");

        return {
            tracking_number,
            raw_response: { status: "created", provider: "opal", text_snippet: bodyText.substring(0, 200) }
        };
    });
});

const PORT = process.env.PORT || 3211;
app.listen(PORT, () => {
    console.log(`Eck Playwright Scraper Service running on port ${PORT}`);
});
