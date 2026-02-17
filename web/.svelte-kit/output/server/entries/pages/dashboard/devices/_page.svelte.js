import { d as attr_class } from "../../../../chunks/index2.js";
import "clsx";
import "../../../../chunks/authStore.js";
import "../../../../chunks/url.js";
import "@sveltejs/kit/internal/server";
import "../../../../chunks/root.js";
import "../../../../chunks/toastStore.js";
function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    let activeTab = "scanners";
    let showQr = false;
    $$renderer2.push(`<div class="page svelte-ecn2p3"><header class="svelte-ecn2p3"><h1 class="svelte-ecn2p3">Connectivity &amp; Devices</h1> <div class="tabs svelte-ecn2p3"><button${attr_class("tab svelte-ecn2p3", void 0, { "active": activeTab === "scanners" })}>Scanners (PDAs)</button> <button${attr_class("tab svelte-ecn2p3", void 0, { "active": activeTab === "servers" })}>Mesh Servers</button></div></header> `);
    {
      $$renderer2.push("<!--[-->");
      $$renderer2.push(`<div class="toolbar svelte-ecn2p3"><div class="action-group svelte-ecn2p3"><button${attr_class("btn secondary svelte-ecn2p3", void 0, { "active": showQr })}>Standard QR</button> <button${attr_class("btn primary svelte-ecn2p3", void 0, { "active": showQr })}>Auto-Approve QR</button></div> <button class="btn secondary svelte-ecn2p3">Refresh</button></div> `);
      {
        $$renderer2.push("<!--[!-->");
      }
      $$renderer2.push(`<!--]--> <div class="list-container svelte-ecn2p3">`);
      {
        $$renderer2.push("<!--[-->");
        $$renderer2.push(`<div class="loading svelte-ecn2p3">Loading devices...</div>`);
      }
      $$renderer2.push(`<!--]--></div>`);
    }
    $$renderer2.push(`<!--]--></div>`);
  });
}
export {
  _page as default
};
