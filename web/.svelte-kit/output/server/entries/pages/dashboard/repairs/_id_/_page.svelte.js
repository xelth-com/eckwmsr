import { c as store_get, e as escape_html, h as attr, u as unsubscribe_stores, f as stringify } from "../../../../../chunks/index2.js";
import { p as page } from "../../../../../chunks/stores.js";
import "../../../../../chunks/authStore.js";
import { b as base } from "../../../../../chunks/server.js";
import "../../../../../chunks/url.js";
import "@sveltejs/kit/internal/server";
import "../../../../../chunks/root.js";
import "@sveltejs/kit/internal";
import "../../../../../chunks/utils.js";
import "../../../../../chunks/exports.js";
import "../../../../../chunks/state.svelte.js";
import "../../../../../chunks/toastStore.js";
function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    var $$store_subs;
    let orderId = store_get($$store_subs ??= {}, "$page", page).params.id;
    let isNew = orderId === "new";
    let loading = !isNew;
    let formData = {
      orderNumber: "",
      customerName: "",
      customerEmail: "",
      productSku: "",
      serialNumber: "",
      issueDescription: "",
      status: "pending",
      priority: "normal",
      repairNotes: "",
      laborHours: 0,
      metadata: {}
    };
    $$renderer2.push(`<div class="detail-page svelte-y09o7m"><div class="header svelte-y09o7m"><button class="back-btn svelte-y09o7m">← Back</button> <div class="title-row svelte-y09o7m"><h1 class="svelte-y09o7m">${escape_html(isNew ? "New Repair Order" : `Repair ${formData.orderNumber}`)}</h1> `);
    if (!isNew) {
      $$renderer2.push("<!--[-->");
      $$renderer2.push(`<button class="delete-btn svelte-y09o7m">Delete</button>`);
    } else {
      $$renderer2.push("<!--[!-->");
    }
    $$renderer2.push(`<!--]--></div></div> `);
    if (loading) {
      $$renderer2.push("<!--[-->");
      $$renderer2.push(`<div class="loading">Loading...</div>`);
    } else {
      $$renderer2.push("<!--[!-->");
      $$renderer2.push(`<form class="form-grid svelte-y09o7m">`);
      if (formData.metadata?.ticketId) {
        $$renderer2.push("<!--[-->");
        $$renderer2.push(`<div class="section full linked-banner svelte-y09o7m"><div class="linked-row svelte-y09o7m"><span class="linked-label svelte-y09o7m">🔗 Linked Support Ticket</span> <a class="linked-link svelte-y09o7m"${attr("href", `${stringify(base)}/dashboard/support/${stringify(formData.metadata.ticketId)}`)}>#${escape_html(formData.metadata.ticketId)} → View Ticket</a></div></div>`);
      } else {
        $$renderer2.push("<!--[!-->");
      }
      $$renderer2.push(`<!--]--> <div class="section svelte-y09o7m"><h2 class="svelte-y09o7m">Customer Information</h2> <div class="field svelte-y09o7m"><label class="svelte-y09o7m">Customer Name *</label> <input type="text"${attr("value", formData.customerName)} required="" class="svelte-y09o7m"/></div> <div class="field svelte-y09o7m"><label class="svelte-y09o7m">Email</label> <input type="email"${attr("value", formData.customerEmail)} class="svelte-y09o7m"/></div></div> <div class="section svelte-y09o7m"><h2 class="svelte-y09o7m">Device Details</h2> <div class="field svelte-y09o7m"><label class="svelte-y09o7m">Device Model / SKU *</label> <input type="text"${attr("value", formData.productSku)} required="" class="code-input svelte-y09o7m"/></div> <div class="field svelte-y09o7m"><label class="svelte-y09o7m">Serial Number</label> <input type="text"${attr("value", formData.serialNumber)} class="code-input svelte-y09o7m"/></div></div> <div class="section full svelte-y09o7m"><h2 class="svelte-y09o7m">Issue Description</h2> <textarea rows="3" class="svelte-y09o7m">`);
      const $$body = escape_html(formData.issueDescription);
      if ($$body) {
        $$renderer2.push(`${$$body}`);
      }
      $$renderer2.push(`</textarea></div> <div class="section svelte-y09o7m"><h2 class="svelte-y09o7m">Repair Details</h2> <div class="field svelte-y09o7m"><label class="svelte-y09o7m">Labor Hours</label> <input type="number" step="0.1" min="0"${attr("value", formData.laborHours)} class="svelte-y09o7m"/></div> <div class="field svelte-y09o7m"><label class="svelte-y09o7m">Repair Notes (Internal)</label> <textarea rows="4" class="svelte-y09o7m">`);
      const $$body_1 = escape_html(formData.repairNotes);
      if ($$body_1) {
        $$renderer2.push(`${$$body_1}`);
      }
      $$renderer2.push(`</textarea></div></div> <div class="section svelte-y09o7m"><h2 class="svelte-y09o7m">Status &amp; Priority</h2> <div class="field svelte-y09o7m"><label class="svelte-y09o7m">Status</label> `);
      $$renderer2.select(
        { value: formData.status, class: "" },
        ($$renderer3) => {
          $$renderer3.option({ value: "pending" }, ($$renderer4) => {
            $$renderer4.push(`Pending`);
          });
          $$renderer3.option({ value: "received" }, ($$renderer4) => {
            $$renderer4.push(`Received`);
          });
          $$renderer3.option({ value: "processing" }, ($$renderer4) => {
            $$renderer4.push(`Processing (In Repair)`);
          });
          $$renderer3.option({ value: "completed" }, ($$renderer4) => {
            $$renderer4.push(`Completed`);
          });
          $$renderer3.option({ value: "cancelled" }, ($$renderer4) => {
            $$renderer4.push(`Cancelled`);
          });
        },
        "svelte-y09o7m"
      );
      $$renderer2.push(`</div> <div class="field svelte-y09o7m"><label class="svelte-y09o7m">Priority</label> `);
      $$renderer2.select(
        { value: formData.priority, class: "" },
        ($$renderer3) => {
          $$renderer3.option({ value: "low" }, ($$renderer4) => {
            $$renderer4.push(`Low`);
          });
          $$renderer3.option({ value: "normal" }, ($$renderer4) => {
            $$renderer4.push(`Normal`);
          });
          $$renderer3.option({ value: "high" }, ($$renderer4) => {
            $$renderer4.push(`High`);
          });
          $$renderer3.option({ value: "urgent" }, ($$renderer4) => {
            $$renderer4.push(`Urgent`);
          });
        },
        "svelte-y09o7m"
      );
      $$renderer2.push(`</div></div> <div class="actions full svelte-y09o7m"><button type="button" class="cancel-btn svelte-y09o7m">Cancel</button> <button type="submit" class="save-btn svelte-y09o7m">${escape_html(isNew ? "Create Order" : "Save Changes")}</button></div></form>`);
    }
    $$renderer2.push(`<!--]--></div>`);
    if ($$store_subs) unsubscribe_stores($$store_subs);
  });
}
export {
  _page as default
};
