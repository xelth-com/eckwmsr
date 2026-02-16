import { h as attr, f as stringify } from "../../../../chunks/index2.js";
import "../../../../chunks/authStore.js";
import { b as base } from "../../../../chunks/server.js";
import "../../../../chunks/url.js";
import "@sveltejs/kit/internal/server";
import "../../../../chunks/root.js";
import "@sveltejs/kit/internal";
import "../../../../chunks/utils.js";
import "../../../../chunks/exports.js";
import "../../../../chunks/state.svelte.js";
import "../../../../chunks/toastStore.js";
function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    $$renderer2.push(`<div class="warehouse-page"><header class="svelte-1n2yjwv"><h1 class="svelte-1n2yjwv">Warehouses</h1> <div class="actions"><a${attr("href", `${stringify(base)}/dashboard/warehouse/blueprint`)} class="action-btn secondary svelte-1n2yjwv">Blueprint Editor</a> <button class="action-btn primary svelte-1n2yjwv">+ New Warehouse</button></div></header> `);
    {
      $$renderer2.push("<!--[-->");
      $$renderer2.push(`<div class="loading">Loading...</div>`);
    }
    $$renderer2.push(`<!--]--></div>`);
  });
}
export {
  _page as default
};
