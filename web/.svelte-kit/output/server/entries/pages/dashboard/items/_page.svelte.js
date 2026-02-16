import "clsx";
import "../../../../chunks/authStore.js";
import "../../../../chunks/url.js";
import "@sveltejs/kit/internal/server";
import "../../../../chunks/root.js";
import "@sveltejs/kit/internal";
import "../../../../chunks/utils.js";
import "../../../../chunks/exports.js";
import "../../../../chunks/state.svelte.js";
function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    $$renderer2.push(`<div class="inventory-page"><header class="svelte-1rwn9xw"><h1 class="svelte-1rwn9xw">Inventory</h1> <div class="actions"><button class="action-btn primary svelte-1rwn9xw">+ Add Item</button></div></header> `);
    {
      $$renderer2.push("<!--[-->");
      $$renderer2.push(`<div class="loading">Loading inventory...</div>`);
    }
    $$renderer2.push(`<!--]--></div>`);
  });
}
export {
  _page as default
};
