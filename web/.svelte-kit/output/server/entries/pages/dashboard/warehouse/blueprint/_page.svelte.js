import { h as attr, e as escape_html, f as stringify } from "../../../../../chunks/index2.js";
import "../../../../../chunks/authStore.js";
import { b as base } from "../../../../../chunks/server.js";
import "../../../../../chunks/url.js";
import "@sveltejs/kit/internal/server";
import "../../../../../chunks/root.js";
import "../../../../../chunks/toastStore.js";
function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    let racks = [];
    $$renderer2.push(`<div class="blueprint-page svelte-1p6svkz"><header class="svelte-1p6svkz"><a${attr("href", `${stringify(base)}/dashboard/warehouse`)} class="back-link svelte-1p6svkz">Home</a> <h1 class="svelte-1p6svkz">Warehouse Blueprint</h1> <div class="rack-count svelte-1p6svkz">${escape_html(racks.length)} rack${escape_html(racks.length !== 1 ? "s" : "")}</div></header> `);
    {
      $$renderer2.push("<!--[-->");
      $$renderer2.push(`<div class="loading svelte-1p6svkz">Loading...</div>`);
    }
    $$renderer2.push(`<!--]--></div>`);
  });
}
export {
  _page as default
};
