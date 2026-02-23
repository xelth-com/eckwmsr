import { h as attr, e as escape_html, d as attr_class, j as bind_props } from "../../../../chunks/index2.js";
import "../../../../chunks/authStore.js";
import "../../../../chunks/url.js";
import "@sveltejs/kit/internal/server";
import "../../../../chunks/root.js";
import "../../../../chunks/toastStore.js";
function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    let data = $$props["data"];
    data.syncHistory || [];
    let loading = false;
    let error = data.error || null;
    let activeTab = "scraper";
    let scraperOnline = null;
    let opalDebug = false;
    let opalLimit = 10;
    let opalRunning = false;
    let dhlDebug = false;
    let dhlLimit = 10;
    let dhlRunning = false;
    let exactDebug = false;
    let exactRunning = false;
    let zohoDebug = false;
    let zohoRunning = false;
    let zohoLimit = 50;
    $$renderer2.push(`<div class="scrapers-page svelte-1rxsw4v"><header class="svelte-1rxsw4v"><h1 class="svelte-1rxsw4v">🤖 Scrapers &amp; Integrations</h1> <div class="header-actions svelte-1rxsw4v"><button class="refresh-btn svelte-1rxsw4v"${attr("disabled", loading, true)}>${escape_html("↻ Refresh")}</button></div></header> <div class="tabs svelte-1rxsw4v"><button${attr_class("tab svelte-1rxsw4v", void 0, { "active": activeTab === "scraper" })}>🎛️ Scraper Admin</button> <button${attr_class("tab svelte-1rxsw4v", void 0, { "active": activeTab === "sync" })}>🔄 Sync History</button></div> `);
    if (error) {
      $$renderer2.push("<!--[-->");
      $$renderer2.push(`<div class="error svelte-1rxsw4v">Failed to load data: ${escape_html(error)}</div>`);
    } else {
      $$renderer2.push("<!--[1-->");
      $$renderer2.push(`<div class="scraper-section svelte-1rxsw4v"><div class="scraper-status-bar svelte-1rxsw4v"><div class="status-left svelte-1rxsw4v"><span${attr_class("status-dot svelte-1rxsw4v", void 0, {
        "online": scraperOnline === true,
        "offline": scraperOnline === false,
        "unknown": scraperOnline === null
      })}></span> <span class="status-label svelte-1rxsw4v">`);
      {
        $$renderer2.push("<!--[!-->");
        $$renderer2.push(`Scraper status unknown`);
      }
      $$renderer2.push(`<!--]--></span></div> <button class="refresh-btn small svelte-1rxsw4v">↻ Check Status</button></div> `);
      {
        $$renderer2.push("<!--[!-->");
      }
      $$renderer2.push(`<!--]--> <div class="provider-cards svelte-1rxsw4v"><div class="provider-card opal-card svelte-1rxsw4v"><div class="card-header svelte-1rxsw4v"><span class="card-title svelte-1rxsw4v">🟢 OPAL Kurier</span> <span class="card-hint svelte-1rxsw4v">opal-kurier.de</span></div> <div class="card-controls svelte-1rxsw4v"><label class="control-row svelte-1rxsw4v"><span>Limit</span> `);
      $$renderer2.select(
        { value: opalLimit, disabled: opalRunning, class: "" },
        ($$renderer3) => {
          $$renderer3.option({ value: 5 }, ($$renderer4) => {
            $$renderer4.push(`5`);
          });
          $$renderer3.option({ value: 10 }, ($$renderer4) => {
            $$renderer4.push(`10`);
          });
          $$renderer3.option({ value: 25 }, ($$renderer4) => {
            $$renderer4.push(`25`);
          });
          $$renderer3.option({ value: 50 }, ($$renderer4) => {
            $$renderer4.push(`50`);
          });
        },
        "svelte-1rxsw4v"
      );
      $$renderer2.push(`</label> <label class="toggle-row svelte-1rxsw4v"><input type="checkbox"${attr("checked", opalDebug, true)}${attr("disabled", opalRunning, true)}/> <span${attr_class("toggle-label svelte-1rxsw4v", void 0, { "debug-on": opalDebug })}>${escape_html("Headless")}</span></label></div> `);
      {
        $$renderer2.push("<!--[!-->");
      }
      $$renderer2.push(`<!--]--> <button class="run-btn opal-run svelte-1rxsw4v"${attr("disabled", scraperOnline !== true, true)}>`);
      {
        $$renderer2.push("<!--[!-->");
        $$renderer2.push(`🚀 Run Fetch`);
      }
      $$renderer2.push(`<!--]--></button> `);
      {
        $$renderer2.push("<!--[!-->");
      }
      $$renderer2.push(`<!--]--></div> <div class="provider-card dhl-card svelte-1rxsw4v"><div class="card-header svelte-1rxsw4v"><span class="card-title svelte-1rxsw4v">🟡 DHL</span> <span class="card-hint svelte-1rxsw4v">geschaeftskunden.dhl.de</span></div> <div class="card-controls svelte-1rxsw4v"><label class="control-row svelte-1rxsw4v"><span>Limit</span> `);
      $$renderer2.select(
        { value: dhlLimit, disabled: dhlRunning, class: "" },
        ($$renderer3) => {
          $$renderer3.option({ value: 5 }, ($$renderer4) => {
            $$renderer4.push(`5`);
          });
          $$renderer3.option({ value: 10 }, ($$renderer4) => {
            $$renderer4.push(`10`);
          });
          $$renderer3.option({ value: 25 }, ($$renderer4) => {
            $$renderer4.push(`25`);
          });
          $$renderer3.option({ value: 50 }, ($$renderer4) => {
            $$renderer4.push(`50`);
          });
        },
        "svelte-1rxsw4v"
      );
      $$renderer2.push(`</label> <label class="toggle-row svelte-1rxsw4v"><input type="checkbox"${attr("checked", dhlDebug, true)}${attr("disabled", dhlRunning, true)}/> <span${attr_class("toggle-label svelte-1rxsw4v", void 0, { "debug-on": dhlDebug })}>${escape_html("Headless")}</span></label></div> `);
      {
        $$renderer2.push("<!--[!-->");
      }
      $$renderer2.push(`<!--]--> <button class="run-btn dhl-run svelte-1rxsw4v"${attr("disabled", scraperOnline !== true, true)}>`);
      {
        $$renderer2.push("<!--[!-->");
        $$renderer2.push(`🚀 Run Fetch`);
      }
      $$renderer2.push(`<!--]--></button> `);
      {
        $$renderer2.push("<!--[!-->");
      }
      $$renderer2.push(`<!--]--></div> <div class="provider-card exact-card svelte-1rxsw4v"><div class="card-header svelte-1rxsw4v"><span class="card-title svelte-1rxsw4v">🔵 Exact Online</span> <span class="card-hint svelte-1rxsw4v">start.exactonline.de</span></div> <div class="stub-warning svelte-1rxsw4v">⚠️ Stub — 2FA not implemented yet</div> <div class="card-controls svelte-1rxsw4v"><label class="toggle-row svelte-1rxsw4v"><input type="checkbox"${attr("checked", exactDebug, true)}${attr("disabled", exactRunning, true)}/> <span${attr_class("toggle-label svelte-1rxsw4v", void 0, { "debug-on": exactDebug })}>${escape_html("Headless")}</span></label></div> `);
      {
        $$renderer2.push("<!--[!-->");
      }
      $$renderer2.push(`<!--]--> <button class="run-btn exact-run svelte-1rxsw4v"${attr("disabled", scraperOnline !== true, true)}>`);
      {
        $$renderer2.push("<!--[!-->");
        $$renderer2.push(`🚀 Run Fetch`);
      }
      $$renderer2.push(`<!--]--></button> `);
      {
        $$renderer2.push("<!--[!-->");
      }
      $$renderer2.push(`<!--]--></div> <div class="provider-card zoho-card svelte-1rxsw4v"><div class="card-header svelte-1rxsw4v"><span class="card-title svelte-1rxsw4v">🟣 Zoho Desk</span> <span class="card-hint svelte-1rxsw4v">desk.inbodysupport.eu</span></div> <div class="card-controls svelte-1rxsw4v"><label class="control-row svelte-1rxsw4v"><span>Limit</span> `);
      $$renderer2.select(
        { value: zohoLimit, disabled: zohoRunning, class: "" },
        ($$renderer3) => {
          $$renderer3.option({ value: 10 }, ($$renderer4) => {
            $$renderer4.push(`10`);
          });
          $$renderer3.option({ value: 50 }, ($$renderer4) => {
            $$renderer4.push(`50`);
          });
          $$renderer3.option({ value: 100 }, ($$renderer4) => {
            $$renderer4.push(`100`);
          });
        },
        "svelte-1rxsw4v"
      );
      $$renderer2.push(`</label> <label class="toggle-row svelte-1rxsw4v"><input type="checkbox"${attr("checked", zohoDebug, true)}${attr("disabled", zohoRunning, true)}/> <span${attr_class("toggle-label svelte-1rxsw4v", void 0, { "debug-on": zohoDebug })}>${escape_html("Headless")}</span></label></div> `);
      {
        $$renderer2.push("<!--[!-->");
      }
      $$renderer2.push(`<!--]--> <button class="run-btn zoho-run svelte-1rxsw4v"${attr("disabled", scraperOnline !== true, true)}>`);
      {
        $$renderer2.push("<!--[!-->");
        $$renderer2.push(`🚀 Fetch Tickets`);
      }
      $$renderer2.push(`<!--]--></button> `);
      {
        $$renderer2.push("<!--[!-->");
      }
      $$renderer2.push(`<!--]--></div></div> <div class="creds-note svelte-1rxsw4v">Credentials are read from server <code class="svelte-1rxsw4v">.env</code> (OPAL_USERNAME / DHL_USERNAME). To test with different creds,
                use curl directly on <code class="svelte-1rxsw4v">POST /S/api/opal/fetch</code> with <code class="svelte-1rxsw4v">"username"</code> and <code class="svelte-1rxsw4v">"password"</code> fields.</div></div>`);
    }
    $$renderer2.push(`<!--]--></div>`);
    bind_props($$props, { data });
  });
}
export {
  _page as default
};
