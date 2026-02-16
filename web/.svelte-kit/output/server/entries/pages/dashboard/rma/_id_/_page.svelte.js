import { c as store_get, e as escape_html, h as attr, u as unsubscribe_stores } from "../../../../../chunks/index2.js";
import { p as page } from "../../../../../chunks/stores.js";
import "../../../../../chunks/authStore.js";
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
    let rmaId = store_get($$store_subs ??= {}, "$page", page).params.id;
    let isNew = rmaId === "new";
    let loading = !isNew;
    let formData = {
      rmaNumber: "",
      customerName: "",
      customerEmail: "",
      productSku: "",
      productName: "",
      issueDescription: "",
      status: "pending",
      priority: "normal"
    };
    $$renderer2.push(`<div class="detail-page svelte-3lraag"><div class="header svelte-3lraag"><button class="back-btn svelte-3lraag">← Back</button> <div class="title-row svelte-3lraag"><h1 class="svelte-3lraag">${escape_html(isNew ? "New RMA Request" : `RMA ${formData.rmaNumber}`)}</h1> `);
    if (!isNew) {
      $$renderer2.push("<!--[-->");
      $$renderer2.push(`<button class="delete-btn svelte-3lraag">Delete</button>`);
    } else {
      $$renderer2.push("<!--[!-->");
    }
    $$renderer2.push(`<!--]--></div></div> `);
    if (loading) {
      $$renderer2.push("<!--[-->");
      $$renderer2.push(`<div class="loading">Loading...</div>`);
    } else {
      $$renderer2.push("<!--[!-->");
      $$renderer2.push(`<form class="form-grid svelte-3lraag"><div class="section svelte-3lraag"><h2 class="svelte-3lraag">Customer Information</h2> <div class="field svelte-3lraag"><label class="svelte-3lraag">Customer Name *</label> <input type="text"${attr("value", formData.customerName)} required="" class="svelte-3lraag"/></div> <div class="field svelte-3lraag"><label class="svelte-3lraag">Email</label> <input type="email"${attr("value", formData.customerEmail)} class="svelte-3lraag"/></div></div> <div class="section svelte-3lraag"><h2 class="svelte-3lraag">Product Details</h2> <div class="field svelte-3lraag"><label class="svelte-3lraag">Product SKU *</label> <input type="text"${attr("value", formData.productSku)} required="" class="code-input svelte-3lraag"/></div> <div class="field svelte-3lraag"><label class="svelte-3lraag">Product Name</label> <input type="text"${attr("value", formData.productName)} class="svelte-3lraag"/></div></div> <div class="section full svelte-3lraag"><h2 class="svelte-3lraag">Issue Description</h2> <textarea rows="4" class="svelte-3lraag">`);
      const $$body = escape_html(formData.issueDescription);
      if ($$body) {
        $$renderer2.push(`${$$body}`);
      }
      $$renderer2.push(`</textarea></div> `);
      if (!isNew) {
        $$renderer2.push("<!--[-->");
        $$renderer2.push(`<div class="section svelte-3lraag"><h2 class="svelte-3lraag">Status</h2> <div class="field svelte-3lraag"><label class="svelte-3lraag">Current Status</label> `);
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
              $$renderer4.push(`Processing`);
            });
            $$renderer3.option({ value: "completed" }, ($$renderer4) => {
              $$renderer4.push(`Completed`);
            });
            $$renderer3.option({ value: "cancelled" }, ($$renderer4) => {
              $$renderer4.push(`Cancelled`);
            });
          },
          "svelte-3lraag"
        );
        $$renderer2.push(`</div> <div class="field svelte-3lraag"><label class="svelte-3lraag">Priority</label> `);
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
          "svelte-3lraag"
        );
        $$renderer2.push(`</div></div>`);
      } else {
        $$renderer2.push("<!--[!-->");
      }
      $$renderer2.push(`<!--]--> <div class="actions full svelte-3lraag"><button type="button" class="cancel-btn svelte-3lraag">Cancel</button> <button type="submit" class="save-btn svelte-3lraag">${escape_html(isNew ? "Create Request" : "Save Changes")}</button></div></form>`);
    }
    $$renderer2.push(`<!--]--></div>`);
    if ($$store_subs) unsubscribe_stores($$store_subs);
  });
}
export {
  _page as default
};
