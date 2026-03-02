import { h as attr, e as escape_html, d as attr_class } from "../../../../chunks/index2.js";
import "../../../../chunks/authStore.js";
import "../../../../chunks/url.js";
import "@sveltejs/kit/internal/server";
import "../../../../chunks/root.js";
import "../../../../chunks/toastStore.js";
function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    let loading = false;
    let dumpData = [];
    let systemPrompt = `You are an expert technical support analyst for InBody devices.
I will provide you with a log of our recent support tickets.
Please analyze these conversations and identify:
1. The most common hardware issues.
2. The standard solutions we provided that successfully resolved the issues.
3. Generate a step-by-step troubleshooting guide for the top 3 most frequent problems.

Here is the data:
---
`;
    let filterStatus = "Closed";
    $$renderer2.push(`<div class="analysis-page svelte-1mr2qs"><header class="svelte-1mr2qs"><h1 class="svelte-1mr2qs">AI Analysis &amp; Research</h1> <p class="subtitle svelte-1mr2qs">Sandbox for analyzing database records, finding patterns, and generating LLM prompts.</p></header> <div class="grid svelte-1mr2qs"><div class="card svelte-1mr2qs"><div class="card-header svelte-1mr2qs"><h2 class="svelte-1mr2qs">Support Knowledge Extractor</h2> <span class="badge svelte-1mr2qs">Ready</span></div> <p class="desc svelte-1mr2qs">Pulls all support ticket conversations, strips HTML formatting, and builds a massive text block.
                You can copy this directly into Claude or ChatGPT to identify recurring issues or generate troubleshooting guides.</p> <div class="controls svelte-1mr2qs"><button class="btn secondary svelte-1mr2qs"${attr("disabled", loading, true)}>${escape_html("1. Fetch Database Records")}</button> <span class="record-count svelte-1mr2qs">`);
    if (dumpData.length > 0) {
      $$renderer2.push("<!--[-->");
      $$renderer2.push(`${escape_html(dumpData.length)} records loaded in memory`);
    } else {
      $$renderer2.push("<!--[!-->");
    }
    $$renderer2.push(`<!--]--></span></div> <div${attr_class("prompt-builder svelte-1mr2qs", void 0, { "disabled": dumpData.length === 0 })}><label class="svelte-1mr2qs">Filter Tickets to Include:</label> `);
    $$renderer2.select(
      { value: filterStatus, class: "" },
      ($$renderer3) => {
        $$renderer3.option({ value: "Closed" }, ($$renderer4) => {
          $$renderer4.push(`Closed / Resolved Only`);
        });
        $$renderer3.option({ value: "All" }, ($$renderer4) => {
          $$renderer4.push(`All Tickets`);
        });
        $$renderer3.option({ value: "Open" }, ($$renderer4) => {
          $$renderer4.push(`Open Only`);
        });
      },
      "svelte-1mr2qs"
    );
    $$renderer2.push(` <label class="svelte-1mr2qs">System Prompt (Instructions for AI):</label> <textarea rows="8" class="svelte-1mr2qs">`);
    const $$body = escape_html(systemPrompt);
    if ($$body) {
      $$renderer2.push(`${$$body}`);
    }
    $$renderer2.push(`</textarea> <button class="btn primary svelte-1mr2qs"${attr("disabled", dumpData.length === 0, true)}>2. Copy Full Prompt to AI</button></div></div> <div class="card wip svelte-1mr2qs"><div class="card-header svelte-1mr2qs"><h2 class="svelte-1mr2qs">Repair Statistics</h2> <span class="badge wip-badge svelte-1mr2qs">WIP</span></div> <p class="desc svelte-1mr2qs">Future module: Will aggregate data from the <code>orders</code> table (Repairs) to show charts:
                average labor hours, most replaced parts by device model, and warranty vs non-warranty ratios.</p> <div class="placeholder-box svelte-1mr2qs">Data Visualization coming soon...</div></div> <div class="card wip svelte-1mr2qs"><div class="card-header svelte-1mr2qs"><h2 class="svelte-1mr2qs">Automated Issue Resolution (RAG)</h2> <span class="badge wip-badge svelte-1mr2qs">WIP</span></div> <p class="desc svelte-1mr2qs">Future module: Instead of manually copying text to ChatGPT, this tool will use our internal Gemini API integration
                (via <code>rig-core</code>) to automatically search past tickets and suggest a solution for a specific problem.</p> <div class="placeholder-box svelte-1mr2qs">Internal AI Vector Search coming soon...</div></div></div></div>`);
  });
}
export {
  _page as default
};
