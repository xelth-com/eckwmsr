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
    store_get($$store_subs ??= {}, "$page", page).params.id;
    $$renderer2.push(`<div class="blueprint-page svelte-174jrif"><div class="header svelte-174jrif"><button class="back-btn svelte-174jrif">← Back</button> <div class="title-row svelte-174jrif">`);
    {
      $$renderer2.push("<!--[!-->");
      $$renderer2.push(`<h1 class="svelte-174jrif">Warehouse Blueprint</h1>`);
    }
    $$renderer2.push(`<!--]--></div></div> `);
    {
      $$renderer2.push("<!--[-->");
      $$renderer2.push(`<div class="loading">Loading blueprint...</div>`);
    }
    $$renderer2.push(`<!--]--></div>`);
    if ($$store_subs) unsubscribe_stores($$store_subs);
  });
}
export {
  _page as default
};
