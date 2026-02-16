import "clsx";
import "../../../../chunks/authStore.js";
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
    $$renderer2.push(`<div class="rma-page"><header class="svelte-rb4uhc"><h1 class="svelte-rb4uhc">RMA Requests</h1> <div class="actions"><button class="action-btn primary svelte-rb4uhc">+ New Request</button></div></header> `);
    {
      $$renderer2.push("<!--[-->");
      $$renderer2.push(`<div class="loading">Loading requests...</div>`);
    }
    $$renderer2.push(`<!--]--></div>`);
  });
}
export {
  _page as default
};
