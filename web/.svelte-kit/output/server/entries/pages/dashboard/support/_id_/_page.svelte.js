import { c as store_get, u as unsubscribe_stores } from "../../../../../chunks/index2.js";
import { p as page } from "../../../../../chunks/stores.js";
import "clsx";
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
    let contact;
    store_get($$store_subs ??= {}, "$page", page).params.id;
    let threads = [];
    contact = {};
    contact.fullName || [contact.firstName, contact.lastName].filter(Boolean).join(" ") || threads[0]?.payload?.from || "";
    contact.email || "";
    contact.phone || "";
    $$renderer2.push(`<div class="detail-page svelte-15vwi97"><div class="back-link"><button class="back-btn svelte-15vwi97">← Back to tickets</button></div> `);
    {
      $$renderer2.push("<!--[-->");
      $$renderer2.push(`<div class="loading svelte-15vwi97">Loading threads...</div>`);
    }
    $$renderer2.push(`<!--]--></div>`);
    if ($$store_subs) unsubscribe_stores($$store_subs);
  });
}
export {
  _page as default
};
