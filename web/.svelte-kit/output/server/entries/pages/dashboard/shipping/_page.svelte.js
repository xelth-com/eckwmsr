import { h as attr, e as escape_html, d as attr_class, b as ensure_array_like, i as attr_style, f as stringify, j as bind_props } from "../../../../chunks/index2.js";
import "../../../../chunks/authStore.js";
import "../../../../chunks/url.js";
import "@sveltejs/kit/internal/server";
import "../../../../chunks/root.js";
import "../../../../chunks/toastStore.js";
import "@sveltejs/kit/internal";
import "../../../../chunks/utils.js";
import "../../../../chunks/exports.js";
import "../../../../chunks/state.svelte.js";
function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    let data = $$props["data"];
    let pickings = data.pickings || [];
    let shipments = data.shipments || [];
    let providersConfig = data.providersConfig || { opal: false, dhl: false };
    let loading = false;
    let error = data.error || null;
    let activeTab = "pickings";
    let processingPickings = /* @__PURE__ */ new Set();
    function formatDate(dateStr) {
      if (!dateStr) return "-";
      return new Date(dateStr).toLocaleDateString("de-DE", {
        day: "2-digit",
        month: "2-digit",
        year: "numeric",
        hour: "2-digit",
        minute: "2-digit"
      });
    }
    function getStateColor(state) {
      const c = {
        draft: "#6c757d",
        assigned: "#ffc107",
        confirmed: "#17a2b8",
        done: "#28a745",
        cancel: "#dc3545"
      };
      return c[state] || "#6c757d";
    }
    $$renderer2.push(`<div class="shipping-page svelte-1dkwspw"><header class="svelte-1dkwspw"><h1 class="svelte-1dkwspw">📦 Shipping &amp; Delivery</h1> <div class="header-actions svelte-1dkwspw">`);
    if (providersConfig?.opal === true) {
      $$renderer2.push("<!--[-->");
      $$renderer2.push(`<button class="action-btn opal-btn svelte-1dkwspw"${attr("disabled", loading, true)}>${escape_html("🟢 Sync OPAL")}</button>`);
    } else {
      $$renderer2.push("<!--[!-->");
    }
    $$renderer2.push(`<!--]--> `);
    if (providersConfig?.dhl === true) {
      $$renderer2.push("<!--[-->");
      $$renderer2.push(`<button class="action-btn dhl-btn svelte-1dkwspw"${attr("disabled", loading, true)}>${escape_html("🟡 Sync DHL")}</button>`);
    } else {
      $$renderer2.push("<!--[!-->");
    }
    $$renderer2.push(`<!--]--> <button class="refresh-btn svelte-1dkwspw"${attr("disabled", loading, true)}>${escape_html("↻ Refresh")}</button></div></header> <div class="tabs svelte-1dkwspw"><button${attr_class("tab svelte-1dkwspw", void 0, { "active": activeTab === "pickings" })}>📋 Ready to Ship (${escape_html(pickings.length)})</button> <button${attr_class("tab svelte-1dkwspw", void 0, { "active": activeTab === "shipments" })}>🚚 Shipments (${escape_html(shipments.length)})</button></div> `);
    if (error) {
      $$renderer2.push("<!--[1-->");
      $$renderer2.push(`<div class="error svelte-1dkwspw">Failed to load data: ${escape_html(error)}</div>`);
    } else {
      $$renderer2.push("<!--[2-->");
      $$renderer2.push(`<div class="pickings-section"><p class="section-desc svelte-1dkwspw">These are Odoo Transfer Orders ready to be shipped. Click "Ship with OPAL" to create a delivery shipment.</p> `);
      if (pickings.length === 0) {
        $$renderer2.push("<!--[-->");
        $$renderer2.push(`<div class="empty-state svelte-1dkwspw"><p class="svelte-1dkwspw">✅ No pickings ready to ship</p> <small class="svelte-1dkwspw">Pickings with status "assigned" will appear here</small></div>`);
      } else {
        $$renderer2.push("<!--[!-->");
        $$renderer2.push(`<div class="table-container svelte-1dkwspw"><table class="svelte-1dkwspw"><thead class="svelte-1dkwspw"><tr><th class="svelte-1dkwspw">Picking #</th><th class="svelte-1dkwspw">Origin</th><th class="svelte-1dkwspw">Partner</th><th class="svelte-1dkwspw">Location</th><th class="svelte-1dkwspw">State</th><th class="svelte-1dkwspw">Scheduled</th><th class="svelte-1dkwspw">Actions</th></tr></thead><tbody class="svelte-1dkwspw"><!--[-->`);
        const each_array = ensure_array_like(pickings);
        for (let $$index = 0, $$length = each_array.length; $$index < $$length; $$index++) {
          let picking = each_array[$$index];
          $$renderer2.push(`<tr class="svelte-1dkwspw"><td class="picking-name svelte-1dkwspw">${escape_html(picking.name)}</td><td class="svelte-1dkwspw">${escape_html(picking.origin || "-")}</td><td class="svelte-1dkwspw">${escape_html(picking.partner_id || "-")}</td><td class="svelte-1dkwspw"><div class="location-cell svelte-1dkwspw"><span class="from svelte-1dkwspw">${escape_html(picking.location_id || "-")}</span> <span class="arrow svelte-1dkwspw">→</span> <span class="to svelte-1dkwspw">${escape_html(picking.location_dest_id || "-")}</span></div></td><td class="svelte-1dkwspw"><span class="state-badge svelte-1dkwspw"${attr_style(`background-color: ${stringify(getStateColor(picking.state))}`)}>${escape_html(picking.state)}</span></td><td class="svelte-1dkwspw">${escape_html(formatDate(picking.scheduled_date))}</td><td class="svelte-1dkwspw"><button class="action-btn ship-btn svelte-1dkwspw"${attr("disabled", processingPickings.has(picking.id), true)}>${escape_html(processingPickings.has(picking.id) ? "⏳ Processing..." : "🚚 Ship with OPAL")}</button></td></tr>`);
        }
        $$renderer2.push(`<!--]--></tbody></table></div>`);
      }
      $$renderer2.push(`<!--]--></div>`);
    }
    $$renderer2.push(`<!--]--></div>`);
    bind_props($$props, { data });
  });
}
export {
  _page as default
};
