import { d as attr_class } from "../../../../chunks/index2.js";
import "clsx";
import "../../../../chunks/authStore.js";
import "../../../../chunks/url.js";
import "@sveltejs/kit/internal/server";
import "../../../../chunks/root.js";
import "../../../../chunks/toastStore.js";
function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    let showQr = false;
    $$renderer2.push(`<div class="page"><header class="svelte-ecn2p3"><h1 class="svelte-ecn2p3">Device Management</h1> <div class="action-group svelte-ecn2p3"><button${attr_class("btn secondary svelte-ecn2p3", void 0, { "active": showQr })}>Show Standard QR</button> <button${attr_class("btn primary svelte-ecn2p3", void 0, { "active": showQr })}>Show Auto-Approve QR</button> <button class="btn secondary svelte-ecn2p3">↻ Refresh</button></div></header> `);
    {
      $$renderer2.push("<!--[!-->");
    }
    $$renderer2.push(`<!--]--> <div class="device-list">`);
    {
      $$renderer2.push("<!--[-->");
      $$renderer2.push(`<div class="loading svelte-ecn2p3">Loading devices...</div>`);
    }
    $$renderer2.push(`<!--]--></div></div>`);
  });
}
export {
  _page as default
};
