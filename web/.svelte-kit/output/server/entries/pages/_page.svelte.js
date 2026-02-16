import { c as store_get, h as attr, e as escape_html, u as unsubscribe_stores, f as stringify } from "../../chunks/index2.js";
import { a as authStore } from "../../chunks/authStore.js";
import { b as base } from "../../chunks/server.js";
import "../../chunks/url.js";
import "@sveltejs/kit/internal/server";
import "../../chunks/root.js";
function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    var $$store_subs;
    $$renderer2.push(`<div class="landing-page svelte-1uha8ag"><nav class="navbar svelte-1uha8ag"><div class="logo svelte-1uha8ag">eckWMS <span class="badge svelte-1uha8ag">GO</span></div> <div class="nav-links"><a href="https://github.com/xelth-com/eckwmsgo" target="_blank" rel="noreferrer" class="github-link svelte-1uha8ag">GitHub</a></div></nav> <main class="hero svelte-1uha8ag"><div class="hero-content"><h1 class="svelte-1uha8ag">Warehouse Management <br/><span class="accent svelte-1uha8ag">Reimagined</span></h1> <p class="description svelte-1uha8ag">Welcome to <strong>eckWMS</strong> — a modern open-source warehouse management system.
                Built on microservices architecture using <strong>Go</strong> and <strong>SvelteKit</strong>.</p> <div class="cta-group svelte-1uha8ag">`);
    if (
      // Мы больше не делаем авто-редирект, чтобы пользователи могли прочитать о системе.
      // Состояние авторизации используется только для переключения кнопки Login/Dashboard.
      store_get($$store_subs ??= {}, "$authStore", authStore).isLoading
    ) {
      $$renderer2.push("<!--[-->");
      $$renderer2.push(`<button class="btn primary loading svelte-1uha8ag">Loading...</button>`);
    } else if (store_get($$store_subs ??= {}, "$authStore", authStore).isAuthenticated) {
      $$renderer2.push("<!--[1-->");
      $$renderer2.push(`<a${attr("href", `${stringify(base)}/dashboard`)} class="btn primary svelte-1uha8ag">Open Dashboard →</a>`);
    } else {
      $$renderer2.push("<!--[!-->");
      $$renderer2.push(`<a${attr("href", `${stringify(base)}/login`)} class="btn primary svelte-1uha8ag">Sign In</a>`);
    }
    $$renderer2.push(`<!--]--> <a href="https://github.com/xelth-com/eckwmsgo" target="_blank" rel="noreferrer" class="btn secondary svelte-1uha8ag">View Source</a></div></div> <div class="features-grid svelte-1uha8ag"><div class="feature-card svelte-1uha8ag"><h3 class="svelte-1uha8ag">🚀 High Performance</h3> <p class="svelte-1uha8ag">Go backend delivers blazing-fast request processing with minimal resource consumption.</p></div> <div class="feature-card svelte-1uha8ag"><h3 class="svelte-1uha8ag">📱 Smart Codes</h3> <p class="svelte-1uha8ag">Support for intelligent barcodes (i/b/p/l) enabling offline validation and instant scanning.</p></div> <div class="feature-card svelte-1uha8ag"><h3 class="svelte-1uha8ag">🔄 Odoo Sync</h3> <p class="svelte-1uha8ag">Two-way synchronization with Odoo 17 ERP. Full warehouse accounting integration.</p></div> <div class="feature-card svelte-1uha8ag"><h3 class="svelte-1uha8ag">🔒 Zero-Knowledge</h3> <p class="svelte-1uha8ag">Relay architecture enables data synchronization through untrusted networks with encryption.</p></div></div></main> <footer class="svelte-1uha8ag"><p>© ${escape_html((/* @__PURE__ */ new Date()).getFullYear())} xelth-com. Open Source Software.</p></footer></div>`);
    if ($$store_subs) unsubscribe_stores($$store_subs);
  });
}
export {
  _page as default
};
