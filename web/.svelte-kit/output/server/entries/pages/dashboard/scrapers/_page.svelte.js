import { h as attr, e as escape_html, d as attr_class, b as ensure_array_like, j as bind_props } from "../../../../chunks/index2.js";
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
    let scraperStarting = false;
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
    let zohoLimit = 10;
    let zohoThreadTicketId = "";
    let zohoThreadRunning = false;
    let excelRepairs = [];
    let excelTotal = 0;
    let excelLoading = false;
    let excelLimit = 30;
    let excelSelected = /* @__PURE__ */ new Set();
    let excelMode = "import";
    $$renderer2.push(`<div class="scrapers-page svelte-1rxsw4v"><header class="svelte-1rxsw4v"><h1 class="svelte-1rxsw4v">🤖 Scrapers &amp; Integrations</h1> <div class="header-actions svelte-1rxsw4v"><button class="refresh-btn svelte-1rxsw4v"${attr("disabled", loading, true)}>${escape_html("↻ Refresh")}</button></div></header> <div class="tabs svelte-1rxsw4v"><button${attr_class("tab svelte-1rxsw4v", void 0, { "active": activeTab === "scraper" })}>🎛️ Scraper Admin</button> <button${attr_class("tab svelte-1rxsw4v", void 0, { "active": activeTab === "sync" })}>🔄 Sync History</button></div> `);
    if (error) {
      $$renderer2.push("<!--[-->");
      $$renderer2.push(`<div class="error svelte-1rxsw4v">Failed to load data: ${escape_html(error)}</div>`);
    } else {
      $$renderer2.push("<!--[1-->");
      $$renderer2.push(`<div class="scraper-section svelte-1rxsw4v"><div class="scraper-status-bar svelte-1rxsw4v"><div class="status-left svelte-1rxsw4v"><span${attr_class("status-dot svelte-1rxsw4v", void 0, {
        "online": scraperOnline === true,
        "offline": scraperOnline === false,
        "unknown": scraperOnline === null,
        "starting": scraperStarting
      })}></span> <span class="status-label svelte-1rxsw4v">`);
      {
        $$renderer2.push("<!--[!-->");
        $$renderer2.push(`Scraper status unknown`);
      }
      $$renderer2.push(`<!--]--></span></div> <div class="status-actions svelte-1rxsw4v">`);
      {
        $$renderer2.push("<!--[-->");
        $$renderer2.push(`<button class="run-btn start-scraper-btn svelte-1rxsw4v">Start Scraper</button>`);
      }
      $$renderer2.push(`<!--]--> <button class="refresh-btn small svelte-1rxsw4v"${attr("disabled", scraperStarting, true)}>↻ Check Status</button></div></div> `);
      {
        $$renderer2.push("<!--[!-->");
      }
      $$renderer2.push(`<!--]--> `);
      {
        $$renderer2.push("<!--[!-->");
      }
      $$renderer2.push(`<!--]--> <div class="provider-cards svelte-1rxsw4v"><div class="provider-card opal-card svelte-1rxsw4v"><div class="card-header svelte-1rxsw4v"><span class="card-title svelte-1rxsw4v">🟢 OPAL Kurier</span> <span class="card-hint svelte-1rxsw4v">opal-kurier.de</span></div> <div class="card-controls svelte-1rxsw4v"><label class="control-row svelte-1rxsw4v"><span class="svelte-1rxsw4v">Limit</span> `);
      $$renderer2.select(
        { value: opalLimit, disabled: opalRunning, class: "" },
        ($$renderer3) => {
          $$renderer3.option(
            { value: 5, class: "" },
            ($$renderer4) => {
              $$renderer4.push(`5`);
            },
            "svelte-1rxsw4v"
          );
          $$renderer3.option(
            { value: 10, class: "" },
            ($$renderer4) => {
              $$renderer4.push(`10`);
            },
            "svelte-1rxsw4v"
          );
          $$renderer3.option(
            { value: 25, class: "" },
            ($$renderer4) => {
              $$renderer4.push(`25`);
            },
            "svelte-1rxsw4v"
          );
          $$renderer3.option(
            { value: 50, class: "" },
            ($$renderer4) => {
              $$renderer4.push(`50`);
            },
            "svelte-1rxsw4v"
          );
        },
        "svelte-1rxsw4v"
      );
      $$renderer2.push(`</label> <label class="toggle-row svelte-1rxsw4v"><input type="checkbox"${attr("checked", opalDebug, true)}${attr("disabled", opalRunning, true)} class="svelte-1rxsw4v"/> <span${attr_class("toggle-label svelte-1rxsw4v", void 0, { "debug-on": opalDebug })}>${escape_html("Headless")}</span></label></div> `);
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
      $$renderer2.push(`<!--]--></div> <div class="provider-card dhl-card svelte-1rxsw4v"><div class="card-header svelte-1rxsw4v"><span class="card-title svelte-1rxsw4v">🟡 DHL</span> <span class="card-hint svelte-1rxsw4v">geschaeftskunden.dhl.de</span></div> <div class="card-controls svelte-1rxsw4v"><label class="control-row svelte-1rxsw4v"><span class="svelte-1rxsw4v">Limit</span> `);
      $$renderer2.select(
        { value: dhlLimit, disabled: dhlRunning, class: "" },
        ($$renderer3) => {
          $$renderer3.option(
            { value: 5, class: "" },
            ($$renderer4) => {
              $$renderer4.push(`5`);
            },
            "svelte-1rxsw4v"
          );
          $$renderer3.option(
            { value: 10, class: "" },
            ($$renderer4) => {
              $$renderer4.push(`10`);
            },
            "svelte-1rxsw4v"
          );
          $$renderer3.option(
            { value: 25, class: "" },
            ($$renderer4) => {
              $$renderer4.push(`25`);
            },
            "svelte-1rxsw4v"
          );
          $$renderer3.option(
            { value: 50, class: "" },
            ($$renderer4) => {
              $$renderer4.push(`50`);
            },
            "svelte-1rxsw4v"
          );
        },
        "svelte-1rxsw4v"
      );
      $$renderer2.push(`</label> <label class="toggle-row svelte-1rxsw4v"><input type="checkbox"${attr("checked", dhlDebug, true)}${attr("disabled", dhlRunning, true)} class="svelte-1rxsw4v"/> <span${attr_class("toggle-label svelte-1rxsw4v", void 0, { "debug-on": dhlDebug })}>${escape_html("Headless")}</span></label></div> `);
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
      $$renderer2.push(`<!--]--></div> <div class="provider-card exact-card svelte-1rxsw4v"><div class="card-header svelte-1rxsw4v"><span class="card-title svelte-1rxsw4v">🔵 Exact Online</span> <span class="card-hint svelte-1rxsw4v">start.exactonline.de</span></div> <div class="stub-warning svelte-1rxsw4v">⚠️ Stub — 2FA not implemented yet</div> <div class="card-controls svelte-1rxsw4v"><label class="toggle-row svelte-1rxsw4v"><input type="checkbox"${attr("checked", exactDebug, true)}${attr("disabled", exactRunning, true)} class="svelte-1rxsw4v"/> <span${attr_class("toggle-label svelte-1rxsw4v", void 0, { "debug-on": exactDebug })}>${escape_html("Headless")}</span></label></div> `);
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
      $$renderer2.push(`<!--]--></div> <div class="provider-card zoho-card svelte-1rxsw4v"><div class="card-header svelte-1rxsw4v"><span class="card-title svelte-1rxsw4v">🟣 Zoho Desk</span> <span class="card-hint svelte-1rxsw4v">desk.inbodysupport.eu</span></div> <div class="card-controls svelte-1rxsw4v"><label class="control-row svelte-1rxsw4v"><span class="svelte-1rxsw4v">Limit</span> `);
      $$renderer2.select(
        { value: zohoLimit, disabled: zohoRunning, class: "" },
        ($$renderer3) => {
          $$renderer3.option(
            { value: 10, class: "" },
            ($$renderer4) => {
              $$renderer4.push(`10`);
            },
            "svelte-1rxsw4v"
          );
          $$renderer3.option(
            { value: 50, class: "" },
            ($$renderer4) => {
              $$renderer4.push(`50`);
            },
            "svelte-1rxsw4v"
          );
          $$renderer3.option(
            { value: 100, class: "" },
            ($$renderer4) => {
              $$renderer4.push(`100`);
            },
            "svelte-1rxsw4v"
          );
          $$renderer3.option(
            { value: 500, class: "" },
            ($$renderer4) => {
              $$renderer4.push(`500`);
            },
            "svelte-1rxsw4v"
          );
          $$renderer3.option(
            { value: 1e3, class: "" },
            ($$renderer4) => {
              $$renderer4.push(`1000`);
            },
            "svelte-1rxsw4v"
          );
        },
        "svelte-1rxsw4v"
      );
      $$renderer2.push(`</label> <label class="toggle-row svelte-1rxsw4v"><input type="checkbox"${attr("checked", zohoDebug, true)}${attr("disabled", zohoRunning, true)} class="svelte-1rxsw4v"/> <span${attr_class("toggle-label svelte-1rxsw4v", void 0, { "debug-on": zohoDebug })}>${escape_html("Headless")}</span></label></div> `);
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
      $$renderer2.push(`<!--]--> <div class="threads-section svelte-1rxsw4v"><div class="threads-row svelte-1rxsw4v"><input type="text"${attr("value", zohoThreadTicketId)} placeholder="Ticket ID for email threads"${attr("disabled", zohoThreadRunning, true)} class="ticket-id-input svelte-1rxsw4v"/> <button class="run-btn zoho-run svelte-1rxsw4v"${attr("disabled", !zohoThreadTicketId, true)}>`);
      {
        $$renderer2.push("<!--[!-->");
        $$renderer2.push(`📧 Fetch Threads`);
      }
      $$renderer2.push(`<!--]--></button></div> `);
      {
        $$renderer2.push("<!--[!-->");
      }
      $$renderer2.push(`<!--]--> `);
      {
        $$renderer2.push("<!--[!-->");
      }
      $$renderer2.push(`<!--]--></div></div></div> <div class="excel-section svelte-1rxsw4v"><div class="excel-header svelte-1rxsw4v"><span class="excel-title svelte-1rxsw4v">📋 Excel Reparaturliste</span> <button class="run-btn excel-info-btn svelte-1rxsw4v"${attr("disabled", scraperOnline !== true, true)}>${escape_html("i")} Info</button></div> `);
      {
        $$renderer2.push("<!--[!-->");
      }
      $$renderer2.push(`<!--]--> <div class="excel-mode-tabs svelte-1rxsw4v"><button${attr_class("excel-tab svelte-1rxsw4v", void 0, { "active": excelMode === "import" })}>📥 Import (Excel → DB)</button> <button${attr_class("excel-tab svelte-1rxsw4v", void 0, { "active": excelMode === "export" })}>📤 Export (DB → Excel)</button></div> `);
      {
        $$renderer2.push("<!--[-->");
        $$renderer2.push(`<div class="excel-panel svelte-1rxsw4v"><div class="excel-controls-row svelte-1rxsw4v"><label class="control-row svelte-1rxsw4v"><span class="svelte-1rxsw4v">Show last</span> `);
        $$renderer2.select(
          { value: excelLimit, disabled: excelLoading, class: "" },
          ($$renderer3) => {
            $$renderer3.option(
              { value: 10, class: "" },
              ($$renderer4) => {
                $$renderer4.push(`10`);
              },
              "svelte-1rxsw4v"
            );
            $$renderer3.option(
              { value: 30, class: "" },
              ($$renderer4) => {
                $$renderer4.push(`30`);
              },
              "svelte-1rxsw4v"
            );
            $$renderer3.option(
              { value: 50, class: "" },
              ($$renderer4) => {
                $$renderer4.push(`50`);
              },
              "svelte-1rxsw4v"
            );
            $$renderer3.option(
              { value: 100, class: "" },
              ($$renderer4) => {
                $$renderer4.push(`100`);
              },
              "svelte-1rxsw4v"
            );
            $$renderer3.option(
              { value: 500, class: "" },
              ($$renderer4) => {
                $$renderer4.push(`500`);
              },
              "svelte-1rxsw4v"
            );
          },
          "svelte-1rxsw4v"
        );
        $$renderer2.push(`</label> <button class="run-btn excel-run svelte-1rxsw4v"${attr("disabled", scraperOnline !== true, true)}>`);
        {
          $$renderer2.push("<!--[!-->");
          $$renderer2.push(`📖 Read Excel`);
        }
        $$renderer2.push(`<!--]--></button></div> `);
        {
          $$renderer2.push("<!--[!-->");
        }
        $$renderer2.push(`<!--]--> `);
        if (excelRepairs.length > 0) {
          $$renderer2.push("<!--[-->");
          $$renderer2.push(`<div class="excel-table-info svelte-1rxsw4v">Showing ${escape_html(excelRepairs.length)} of ${escape_html(excelTotal)} repairs (newest first)</div> <div class="excel-table-wrap svelte-1rxsw4v"><table class="excel-table svelte-1rxsw4v"><thead class="svelte-1rxsw4v"><tr class="svelte-1rxsw4v"><th class="svelte-1rxsw4v"><input type="checkbox"${attr("checked", excelSelected.size === excelRepairs.length && excelRepairs.length > 0, true)} class="svelte-1rxsw4v"/></th><th class="svelte-1rxsw4v">Row</th><th class="svelte-1rxsw4v">Repair #</th><th class="svelte-1rxsw4v">Ticket</th><th class="svelte-1rxsw4v">Model</th><th class="svelte-1rxsw4v">Serial</th><th class="svelte-1rxsw4v">Customer</th><th class="svelte-1rxsw4v">Error</th><th class="svelte-1rxsw4v">Received</th><th class="svelte-1rxsw4v">Status</th></tr></thead><tbody class="svelte-1rxsw4v"><!--[-->`);
          const each_array_3 = ensure_array_like(excelRepairs);
          for (let $$index_3 = 0, $$length = each_array_3.length; $$index_3 < $$length; $$index_3++) {
            let r = each_array_3[$$index_3];
            $$renderer2.push(`<tr${attr_class("svelte-1rxsw4v", void 0, { "selected": excelSelected.has(r.repairNumber) })}><td class="svelte-1rxsw4v"><input type="checkbox"${attr("checked", excelSelected.has(r.repairNumber), true)} class="svelte-1rxsw4v"/></td><td class="muted svelte-1rxsw4v">${escape_html(r.excelRow)}</td><td class="mono svelte-1rxsw4v">${escape_html(r.repairNumber)}</td><td class="muted svelte-1rxsw4v">${escape_html(r.ticketNumber || "-")}</td><td class="svelte-1rxsw4v">${escape_html(r.model || "-")}</td><td class="mono svelte-1rxsw4v">${escape_html(r.serialNumber || "-")}</td><td class="truncate svelte-1rxsw4v">${escape_html(r.customerName || "-")}</td><td class="truncate svelte-1rxsw4v">${escape_html(r.errorDescription || "-")}</td><td class="svelte-1rxsw4v">${escape_html(r.dateOfReceipt || "-")}</td><td class="svelte-1rxsw4v"><span${attr_class("status-dot svelte-1rxsw4v", void 0, {
              "done": r.status === "completed",
              "wip": r.status !== "completed"
            })}>${escape_html(r.status === "completed" ? "✅" : "🔧")}</span></td></tr>`);
          }
          $$renderer2.push(`<!--]--></tbody></table></div> <div class="excel-actions-row svelte-1rxsw4v"><button class="run-btn excel-run svelte-1rxsw4v"${attr("disabled", excelSelected.size === 0, true)}>`);
          {
            $$renderer2.push("<!--[!-->");
            $$renderer2.push(`📥 Import ${escape_html(excelSelected.size)} selected to DB`);
          }
          $$renderer2.push(`<!--]--></button> <button class="toggle-json svelte-1rxsw4v">${escape_html("▶")} Raw JSON</button></div> `);
          {
            $$renderer2.push("<!--[!-->");
          }
          $$renderer2.push(`<!--]--> `);
          {
            $$renderer2.push("<!--[!-->");
          }
          $$renderer2.push(`<!--]-->`);
        } else {
          $$renderer2.push("<!--[!-->");
        }
        $$renderer2.push(`<!--]--></div>`);
      }
      $$renderer2.push(`<!--]--></div> <div class="creds-note svelte-1rxsw4v">Credentials are read from server <code class="svelte-1rxsw4v">.env</code> (OPAL_USERNAME / DHL_USERNAME). Excel file path: <code class="svelte-1rxsw4v">EXCEL_REPAIR_FILE</code>.</div></div>`);
    }
    $$renderer2.push(`<!--]--></div>`);
    bind_props($$props, { data });
  });
}
export {
  _page as default
};
