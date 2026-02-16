import { h as attr, e as escape_html } from "../../../chunks/index2.js";
import "../../../chunks/authStore.js";
import "@sveltejs/kit/internal";
import "../../../chunks/url.js";
import "../../../chunks/utils.js";
import "@sveltejs/kit/internal/server";
import "../../../chunks/root.js";
import "../../../chunks/exports.js";
import "../../../chunks/state.svelte.js";
function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    let email = "";
    let password = "";
    let isLoading = false;
    $$renderer2.push(`<div class="login-container svelte-1x05zx6"><div class="login-card svelte-1x05zx6"><div class="logo svelte-1x05zx6"><h1 class="svelte-1x05zx6">eckWMS</h1> <span class="version svelte-1x05zx6">GO Edition</span></div> <form><div class="form-group svelte-1x05zx6"><label for="email" class="svelte-1x05zx6">Email</label> <input type="text" id="email"${attr("value", email)} placeholder="operator@eckwms.local"${attr("disabled", isLoading, true)} class="svelte-1x05zx6"/></div> <div class="form-group svelte-1x05zx6"><label for="password" class="svelte-1x05zx6">Password</label> <input type="password" id="password"${attr("value", password)} placeholder="••••••••"${attr("disabled", isLoading, true)} class="svelte-1x05zx6"/></div> `);
    {
      $$renderer2.push("<!--[!-->");
    }
    $$renderer2.push(`<!--]--> <button type="submit"${attr("disabled", isLoading, true)} class="svelte-1x05zx6">${escape_html("Login")}</button></form></div></div>`);
  });
}
export {
  _page as default
};
