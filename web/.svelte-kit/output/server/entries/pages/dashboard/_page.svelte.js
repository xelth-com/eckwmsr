import { e as escape_html } from "../../../chunks/index2.js";
import "clsx";
import "../../../chunks/authStore.js";
import "../../../chunks/url.js";
import "@sveltejs/kit/internal/server";
import "../../../chunks/root.js";
function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    $$renderer2.push(`<div class="dashboard-home"><header class="svelte-x1i5gj"><h1 class="svelte-x1i5gj">System Overview</h1></header> <div class="stats-grid svelte-x1i5gj"><div class="stat-card primary svelte-x1i5gj"><div class="stat-value svelte-x1i5gj">${escape_html("...")}</div> <div class="stat-label svelte-x1i5gj">Total Items</div></div> <div class="stat-card secondary svelte-x1i5gj"><div class="stat-value svelte-x1i5gj">--</div> <div class="stat-label svelte-x1i5gj">Active Scanners</div></div> <div class="stat-card accent svelte-x1i5gj"><div class="stat-value svelte-x1i5gj">--</div> <div class="stat-label svelte-x1i5gj">Pending RMAs</div></div></div> <div class="activity-section"><h2 class="svelte-x1i5gj">Recent Activity</h2> <div class="empty-state svelte-x1i5gj">No recent activity recorded.</div></div></div>`);
  });
}
export {
  _page as default
};
