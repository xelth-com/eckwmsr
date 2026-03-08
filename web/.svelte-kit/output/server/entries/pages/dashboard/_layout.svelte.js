import { a as ssr_context, b as ensure_array_like, c as store_get, d as attr_class, f as stringify, e as escape_html, u as unsubscribe_stores, h as attr, s as slot } from "../../../chunks/index2.js";
import { a as authStore } from "../../../chunks/authStore.js";
import { w as writable } from "../../../chunks/index.js";
import { t as toastStore } from "../../../chunks/toastStore.js";
import "clsx";
import "../../../chunks/url.js";
import "@sveltejs/kit/internal/server";
import "../../../chunks/root.js";
import { g as goto } from "../../../chunks/client.js";
import { p as page } from "../../../chunks/stores.js";
import { b as base } from "../../../chunks/server.js";
function onDestroy(fn) {
  /** @type {SSRContext} */
  ssr_context.r.on_destroy(fn);
}
function createWsStore() {
  const { subscribe, set, update } = writable({
    connected: false,
    lastMessage: null,
    error: null
  });
  function connect() {
    return;
  }
  function close() {
  }
  function send(msg) {
    {
      console.warn("[WS] Cannot send, not connected");
    }
  }
  return {
    subscribe,
    connect,
    close,
    send
  };
}
const wsStore = createWsStore();
function ToastContainer($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    var $$store_subs;
    $$renderer2.push(`<div class="toast-container svelte-cqwvc2"><!--[-->`);
    const each_array = ensure_array_like(store_get($$store_subs ??= {}, "$toastStore", toastStore));
    for (let $$index = 0, $$length = each_array.length; $$index < $$length; $$index++) {
      let toast = each_array[$$index];
      $$renderer2.push(`<div${attr_class(`toast ${stringify(toast.type)}`, "svelte-cqwvc2")} role="alert">${escape_html(toast.message)}</div>`);
    }
    $$renderer2.push(`<!--]--></div>`);
    if ($$store_subs) unsubscribe_stores($$store_subs);
  });
}
function MeshStatus($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    onDestroy(() => {
    });
    $$renderer2.push(`<div class="mesh-status svelte-10x5b0t">`);
    {
      $$renderer2.push("<!--[-->");
      $$renderer2.push(`<div class="mesh-node loading svelte-10x5b0t"><span class="node-icon svelte-10x5b0t">⏳</span> <span class="node-label svelte-10x5b0t">Loading...</span></div>`);
    }
    $$renderer2.push(`<!--]--></div>`);
  });
}
function _layout($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    var $$store_subs;
    let showAmbiguousModal = false;
    let ambiguousCandidates = [];
    onDestroy(() => {
    });
    function handleWsMessage(msg) {
      if (Date.now() - (msg._receivedAt || 0) > 2e3) return;
      if (msg.barcode || msg.data && msg.data.barcode) {
        const barcode = msg.barcode || msg.data.barcode;
        processScan(barcode);
        return;
      }
      if (msg.success && msg.data) {
        toastStore.add(`Operation Success`, "success");
      } else if (msg.type === "ERROR" || msg.error) {
        toastStore.add(msg.text || msg.error || "Error occurred", "error");
      } else if (msg.text) {
        toastStore.add(msg.text, "info");
      }
    }
    async function processScan(barcode) {
      toastStore.add("Scanning...", "info", 1e3);
      try {
        const res = await fetch("/api/scan", {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
            Authorization: `Bearer ${store_get($$store_subs ??= {}, "$authStore", authStore).token}`
          },
          body: JSON.stringify({ barcode })
        });
        if (!res.ok) {
          const err = await res.json();
          throw new Error(err.error || "Scan failed");
        }
        const data = await res.json();
        if (data.type === "ambiguous") {
          ambiguousCandidates = data.data?.candidates || [];
          showAmbiguousModal = true;
          toastStore.add("Multiple matches — please select", "warning");
          return;
        }
        if (data.trust === "soft") {
          toastStore.add("Opened via external code. Please verify data.", "warning", 4e3);
        }
        toastStore.add(data.message, "success");
        if (data.type === "order" && data.data?.id) {
          goto(`${base}/dashboard/repairs/${data.data.id}`);
        } else if (data.type === "item" && data.data?.id) {
          goto(`${base}/dashboard/items/${data.data.id}`);
        } else if (data.type === "box" && data.data?.id) {
          console.log("Box scanned:", data.data);
          toastStore.add(`Box ${data.data.name || data.data.id} scanned`, "success");
        } else if (data.type === "place" && data.data?.id) {
          goto(`${base}/dashboard/warehouse/${data.data.id}`);
        } else if (data.type === "product" && data.data?.id) {
          goto(`${base}/dashboard/items/${data.data.id}`);
        } else if (data.type === "label") {
          console.log("Label scanned:", data.data);
        }
      } catch (e) {
        console.error("Scan error:", e);
        toastStore.add(`Error: ${e.message}`, "error");
      }
    }
    if (store_get($$store_subs ??= {}, "$wsStore", wsStore).lastMessage) {
      handleWsMessage(store_get($$store_subs ??= {}, "$wsStore", wsStore).lastMessage);
    }
    $$renderer2.push(`<div class="dashboard-layout svelte-2agd5u"><aside class="sidebar svelte-2agd5u"><div class="brand svelte-2agd5u"><span class="brand-text svelte-2agd5u">eckWMS</span></div> <div class="mesh-section svelte-2agd5u"><div class="section-label svelte-2agd5u">Connected Servers:</div> `);
    MeshStatus($$renderer2);
    $$renderer2.push(`<!----></div> <nav class="svelte-2agd5u"><a${attr("href", `${stringify(
      // Prevent processing if message is too old (basic check)
      // Handle Scan Events
      // Play sound (optional, browser policy might block)
      // const audio = new Audio('/beep.mp3'); audio.play().catch(e=>{});
      // Handle ambiguous collision — multiple matches
      // Soft trust warning
      // Show result
      // Handle Navigation / Action based on type
      base
    )}/dashboard`)}${attr_class("svelte-2agd5u", void 0, {
      "active": store_get($$store_subs ??= {}, "$page", page).url.pathname === `${base}/dashboard` || store_get($$store_subs ??= {}, "$page", page).url.pathname === "/dashboard"
    })}>Dashboard</a> <a${attr("href", `${stringify(base)}/dashboard/items`)}${attr_class("svelte-2agd5u", void 0, {
      "active": store_get($$store_subs ??= {}, "$page", page).url.pathname.includes("/items")
    })}>Inventory</a> <a${attr("href", `${stringify(base)}/dashboard/warehouse`)}${attr_class("svelte-2agd5u", void 0, {
      "active": store_get($$store_subs ??= {}, "$page", page).url.pathname.includes("/warehouse")
    })}>Warehouse</a> <a${attr("href", `${stringify(base)}/dashboard/shipping`)}${attr_class("svelte-2agd5u", void 0, {
      "active": store_get($$store_subs ??= {}, "$page", page).url.pathname.includes("/shipping")
    })}>Shipping</a> <a${attr("href", `${stringify(base)}/dashboard/rma`)}${attr_class("svelte-2agd5u", void 0, {
      "active": store_get($$store_subs ??= {}, "$page", page).url.pathname.includes("/rma")
    })}>RMA Requests</a> <a${attr("href", `${stringify(base)}/dashboard/repairs`)}${attr_class("svelte-2agd5u", void 0, {
      "active": store_get($$store_subs ??= {}, "$page", page).url.pathname.includes("/repairs")
    })}>Repairs</a> <a${attr("href", `${stringify(base)}/dashboard/support`)}${attr_class("svelte-2agd5u", void 0, {
      "active": store_get($$store_subs ??= {}, "$page", page).url.pathname.includes("/support")
    })}>Support</a> <a${attr("href", `${stringify(base)}/dashboard/print`)}${attr_class("svelte-2agd5u", void 0, {
      "active": store_get($$store_subs ??= {}, "$page", page).url.pathname.includes("/print")
    })}>Printing</a> <a${attr("href", `${stringify(base)}/dashboard/devices`)}${attr_class("svelte-2agd5u", void 0, {
      "active": store_get($$store_subs ??= {}, "$page", page).url.pathname.includes("/devices")
    })}>Devices</a> <a${attr("href", `${stringify(base)}/dashboard/users`)}${attr_class("svelte-2agd5u", void 0, {
      "active": store_get($$store_subs ??= {}, "$page", page).url.pathname.includes("/users")
    })}>Users</a> <a${attr("href", `${stringify(base)}/dashboard/scrapers`)}${attr_class("svelte-2agd5u", void 0, {
      "active": store_get($$store_subs ??= {}, "$page", page).url.pathname.includes("/scrapers")
    })}>Scrapers</a> <a${attr("href", `${stringify(base)}/dashboard/analysis`)} style="margin-top: 1rem; border-top: 1px solid #333; padding-top: 1rem;"${attr_class("svelte-2agd5u", void 0, {
      "active": store_get($$store_subs ??= {}, "$page", page).url.pathname.includes("/analysis")
    })}>Analysis</a></nav> <div class="user-panel svelte-2agd5u"><div class="user-info svelte-2agd5u"><span class="username svelte-2agd5u">${escape_html(store_get($$store_subs ??= {}, "$authStore", authStore).currentUser?.username || "User")}</span> <span class="role svelte-2agd5u">${escape_html(store_get($$store_subs ??= {}, "$authStore", authStore).currentUser?.role || "Operator")}</span></div> <button class="logout-btn svelte-2agd5u">Logout</button></div></aside> <main class="content svelte-2agd5u"><!--[-->`);
    slot($$renderer2, $$props, "default", {});
    $$renderer2.push(`<!--]--></main> `);
    ToastContainer($$renderer2);
    $$renderer2.push(`<!----> `);
    if (showAmbiguousModal) {
      $$renderer2.push("<!--[-->");
      $$renderer2.push(`<div class="modal-overlay svelte-2agd5u"><div class="modal-card svelte-2agd5u"><h3 class="svelte-2agd5u">Multiple Matches Found</h3> <p class="modal-hint svelte-2agd5u">This barcode matched multiple records. Select the correct one:</p> <div class="candidates-list svelte-2agd5u"><!--[-->`);
      const each_array = ensure_array_like(ambiguousCandidates);
      for (let $$index = 0, $$length = each_array.length; $$index < $$length; $$index++) {
        let c = each_array[$$index];
        $$renderer2.push(`<button class="candidate-btn svelte-2agd5u"><span${attr_class("candidate-type svelte-2agd5u", void 0, {
          "type-order": c.type === "order",
          "type-item": c.type === "item"
        })}>${escape_html(c.type)}</span> <span class="candidate-title svelte-2agd5u">${escape_html(c.title)}</span> `);
        if (c.subtitle) {
          $$renderer2.push("<!--[-->");
          $$renderer2.push(`<span class="candidate-sub svelte-2agd5u">${escape_html(c.subtitle)}</span>`);
        } else {
          $$renderer2.push("<!--[!-->");
        }
        $$renderer2.push(`<!--]--></button>`);
      }
      $$renderer2.push(`<!--]--></div> <button class="cancel-btn svelte-2agd5u">Cancel</button></div></div>`);
    } else {
      $$renderer2.push("<!--[!-->");
    }
    $$renderer2.push(`<!--]--></div>`);
    if ($$store_subs) unsubscribe_stores($$store_subs);
  });
}
export {
  _layout as default
};
