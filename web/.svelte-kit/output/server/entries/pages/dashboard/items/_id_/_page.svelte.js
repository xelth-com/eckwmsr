import "clsx";
import "@sveltejs/kit/internal";
import "../../../../../chunks/url.js";
import "../../../../../chunks/utils.js";
import "@sveltejs/kit/internal/server";
import "../../../../../chunks/root.js";
import "../../../../../chunks/exports.js";
import "../../../../../chunks/state.svelte.js";
import "../../../../../chunks/authStore.js";
import "../../../../../chunks/toastStore.js";
function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    $$renderer2.push(`<div class="detail-page svelte-7rm230"><div class="header svelte-7rm230"><button class="back-btn svelte-7rm230">← Back</button> <div class="title-row svelte-7rm230">`);
    {
      $$renderer2.push("<!--[!-->");
      $$renderer2.push(`<h1 class="svelte-7rm230">Item Details</h1>`);
    }
    $$renderer2.push(`<!--]--></div></div> `);
    {
      $$renderer2.push("<!--[-->");
      $$renderer2.push(`<div class="loading">Loading details...</div>`);
    }
    $$renderer2.push(`<!--]--></div>`);
  });
}
export {
  _page as default
};
