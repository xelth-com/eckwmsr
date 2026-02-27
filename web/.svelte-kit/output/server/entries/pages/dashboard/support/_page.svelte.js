import { h as attr, e as escape_html } from "../../../../chunks/index2.js";
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
    let loading = true;
    $$renderer2.push(`<div class="support-page svelte-1u3890j"><header class="svelte-1u3890j"><h1 class="svelte-1u3890j">💬 Support Tickets</h1> <button class="refresh-btn svelte-1u3890j"${attr("disabled", loading, true)}>${escape_html("↻ Loading...")}</button></header> `);
    {
      $$renderer2.push("<!--[-->");
      $$renderer2.push(`<div class="loading svelte-1u3890j">Loading tickets...</div>`);
    }
    $$renderer2.push(`<!--]--></div>`);
  });
}
export {
  _page as default
};
