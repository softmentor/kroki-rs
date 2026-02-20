const express = require('express');
const { chromium } = require('playwright');
const genericPool = require('generic-pool');

const app = express();
// Increase payload limit to accept huge diagrams
app.use(express.json({ limit: '50mb' }));

// Pool limits dictated by environment injected from Rust `BrowserManager`
const poolSize = parseInt(process.env.KROKI_BROWSER_POOL_SIZE || '4', 10);
const maxUses = parseInt(process.env.KROKI_BROWSER_CONTEXT_TTL || '100', 10);

let browser;

const factory = {
    create: async () => {
        const context = await browser.newContext();
        const page = await context.newPage();

        page.on('console', msg => console.log(`BROWSER CONSOLE: ${msg.text()}`));
        page.on('pageerror', err => console.log(`BROWSER ERROR: ${err.message}`));

        await page.setContent(`<!DOCTYPE html><html><body><div id="container"></div><div id="graphDiv"></div></body></html>`);

        // Fast pre-loading directly from the node_modules distribution packages
        await page.addScriptTag({ path: require.resolve('mermaid/dist/mermaid.min.js') }).catch(e => console.error("Failed to load mermaid.min.js", e));
        await page.addScriptTag({ path: require.resolve('bpmn-js/dist/bpmn-viewer.production.min.js') }).catch(e => console.error("Failed to load bpmn-viewer.production.min.js", e));

        return {
            context,
            page,
            uses: 0,
            id: Math.random().toString(36).substr(2, 9)
        };
    },
    destroy: async (client) => {
        try {
            await client.page.close();
            await client.context.close();
        } catch (err) {
            // Ignore teardown errors
        }
    },
    validate: async (client) => {
        // Enforce TTL Limits: Return true if the client can be safely reused,
        // otherwise `generic-pool` will systematically evict it, killing the bloated
        // Chromium page and launching a clean one.
        return client.uses < maxUses;
    }
};

let pool;

app.post('/evaluate', async (req, res) => {
    const { type, source, format } = req.body;
    let client;
    try {
        client = await pool.acquire();
        client.uses++;
        console.log(`[worker] Processing ${type} request (use #${client.uses})`);

        let svgResult = '';
        if (type === 'mermaid') {
            // CDP Playwright interactions are highly asynchronous. This allows the Node.js
            // worker to efficiently orchestrate over 60+ concurrent evaluation requests 
            // without blocking the main event loop.
            svgResult = await client.page.evaluate(async (code) => {
                if (!window.mermaid) throw new Error("Mermaid library not loaded in browser context");

                // CRITICAL FIX: Reset the DOM footprint before rendering.
                // Reusing cached `BrowserContext`s continuously across hundreds of requests 
                // causes DOM pollution leading to `getElementById` lookup failures. 
                // Manually reconstructing the target anchors eliminates context leak crashes.
                document.body.innerHTML = '<div id="graphDiv"></div>';

                console.log(`[worker] Rendering mermaid diagram...`);
                window.mermaid.initialize({ startOnLoad: false });
                const { svg } = await window.mermaid.render('mermaid-eval-' + Date.now(), code);
                document.getElementById('graphDiv').innerHTML = svg;
                return svg;
            }, source);
        } else if (type === 'bpmn') {
            svgResult = await client.page.evaluate(async (code) => {
                if (!window.BpmnJS) throw new Error("BpmnJS library not loaded in browser context");

                // Reset the DOM footprint to ensure the container anchor is reliably clean.
                document.body.innerHTML = '<div id="container"></div>';

                return new Promise((resolve, reject) => {
                    const BpmnViewer = window.BpmnJS;
                    const viewer = new BpmnViewer({ container: '#container' });
                    viewer.importXML(code).then(() => {
                        viewer.saveSVG({ format: true }).then((r) => resolve(r.svg)).catch(reject);
                    }).catch(reject);
                });
            }, source);
        } else {
            throw new Error(`Unsupported browser evaluation type: ${type}`);
        }

        if (format === 'png') {
            const selector = type === 'mermaid' ? '#graphDiv svg' : '#container svg';
            const bodyBounding = await client.page.evaluate((sel) => {
                const svg = document.querySelector(sel);
                if (!svg) return null;
                // Important to fix SVG dimensions for proper screenshotting
                const bbox = svg.getBoundingClientRect();
                svg.setAttribute('width', bbox.width);
                svg.setAttribute('height', bbox.height);
                return { width: bbox.width, height: bbox.height };
            }, selector);

            const element = await client.page.$(selector);
            if (!element) throw new Error(`Could not locate ${selector} for PNG snapshot`);

            const buffer = await element.screenshot({ type: 'png', omitBackground: true });
            res.status(200).set('Content-Type', 'image/png').send(buffer);
        } else {
            // Default to SVG text
            res.status(200).set('Content-Type', 'image/svg+xml').send(svgResult);
        }

        pool.release(client);
    } catch (err) {
        console.error(`Evaluation error for ${type}:`, err);
        if (client) {
            // If the context crashed or evaluation threw a hard error, destroy it to avoid polluting the pool
            await pool.destroy(client).catch(() => { });
        }
        res.status(400).send({ error: err.message });
    }
});

app.get('/health', (req, res) => {
    res.status(200).send({ status: 'ok', active: pool.borrowed, spare: pool.available, pending: pool.pending });
});

(async () => {
    try {
        browser = await chromium.launch({
            headless: true,
            executablePath: process.env.PLAYWRIGHT_EXECUTABLE_PATH || undefined,
            // Optimization flags for containerized environments
            args: [
                '--no-sandbox',
                '--disable-setuid-sandbox',
                '--disable-dev-shm-usage',
                '--disable-gpu',
            ]
        });

        pool = genericPool.createPool(factory, {
            max: poolSize,
            min: 1, // keep at least 1 warm
            testOnBorrow: true
        });

        // Warm up the pool initially
        pool.acquire().then(c => pool.release(c)).catch(e => {
            console.error("Pool initial acquire failed:", e);
            process.exit(1);
        });

        // Binding to 0 automatically finds a high-port natively
        const server = app.listen(0, '127.0.0.1', () => {
            const port = server.address().port;
            // The magic string that Rust expects to find on stdout
            console.log(`KROKI_BROWSER_WORKER_PORT=${port}`);
        });

        process.on('SIGTERM', async () => {
            server.close();
            await pool.drain();
            await pool.clear();
            await browser.close();
            process.exit(0);
        });
    } catch (launchErr) {
        console.error("Failed to launch Playwright browser. Missing dependencies?:", launchErr);
        process.exit(1);
    }
})();
