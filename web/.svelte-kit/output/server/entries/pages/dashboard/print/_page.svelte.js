import { h as attr, d as attr_class, i as attr_style, b as ensure_array_like, e as escape_html, f as stringify } from "../../../../chunks/index2.js";
import "../../../../chunks/toastStore.js";
function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    let layoutKey, labelDims;
    let selectedType = null;
    let isTightMode = true;
    let cols = 3;
    let rows = 7;
    let pages = 1;
    let count = 21;
    let startNumber = 1;
    let marginTop = 4;
    let marginBottom = 4;
    let marginLeft = 4;
    let marginRight = 4;
    let gapX = 8;
    let gapY = 6;
    function calculateLabelDims(mt, mb, ml, mr, gx, gy, c, r) {
      const pageWidth = 210;
      const pageHeight = 297;
      const workingWidth = pageWidth - ml - mr;
      const workingHeight = pageHeight - mt - mb;
      const totalGapWidth = (c - 1) * gx;
      const totalGapHeight = (r - 1) * gy;
      const w = Math.max(0, (workingWidth - totalGapWidth) / c);
      const h = Math.max(0, (workingHeight - totalGapHeight) / r);
      return {
        w: w.toFixed(1),
        h: h.toFixed(1),
        aspect: h === 0 ? 1 : w / h
      };
    }
    layoutKey = `${cols}x${rows}`;
    count = pages * cols * rows;
    labelDims = calculateLabelDims(marginTop, marginBottom, marginLeft, marginRight, gapX, gapY, cols, rows);
    if (typeof window !== "undefined") {
      localStorage.setItem("print_settings", JSON.stringify({
        marginTop,
        marginBottom,
        marginLeft,
        marginRight,
        gapX,
        gapY,
        cols,
        rows,
        selectedType,
        isTightMode
      }));
    }
    $$renderer2.push(`<div class="print-page svelte-bica9f"><header class="svelte-bica9f"><h1 class="svelte-bica9f">Printing Center</h1></header> <div class="card svelte-bica9f"><h2 class="svelte-bica9f">Page Layout (A4)</h2> <div class="config-bar svelte-bica9f"><div class="field-group svelte-bica9f"><label class="svelte-bica9f">Columns</label> <input type="number"${attr("value", cols)} min="1" max="10" class="svelte-bica9f"/></div> <div class="field-group svelte-bica9f"><label class="svelte-bica9f">Rows</label> <input type="number"${attr("value", rows)} min="1" max="20" class="svelte-bica9f"/></div> <div class="field-group svelte-bica9f"><label class="svelte-bica9f">Count</label> <input type="number"${attr("value", count)} class="svelte-bica9f"/></div> <div class="field-group svelte-bica9f"><label class="svelte-bica9f">Start #</label> <input type="number"${attr("value", startNumber)} class="svelte-bica9f"/></div> <div class="toggle-group svelte-bica9f"><label><input type="checkbox"${attr("checked", isTightMode, true)} class="svelte-bica9f"/> Tight Mode (Overlap)</label></div></div> <div${attr_class("visual-editor-container svelte-bica9f", void 0, { "is-safe": !isTightMode })}><div class="visual-page svelte-bica9f"${attr_style(` --mt: ${stringify(marginTop)}px; --mb: ${stringify(marginBottom)}px; --ml: ${stringify(marginLeft)}px; --mr: ${stringify(marginRight)}px; --gx: ${stringify(gapX)}px; --gy: ${stringify(gapY)}px; --cols: ${stringify(cols)}; --rows: ${stringify(rows)}; `)}><div class="margin-control top svelte-bica9f"><input type="number"${attr("value", marginTop)} min="0" class="svelte-bica9f"/> <span class="svelte-bica9f">Top</span></div> <div class="margin-control left svelte-bica9f"><input type="number"${attr("value", marginLeft)} min="0" class="svelte-bica9f"/> <span class="svelte-bica9f">Left</span></div> <div class="margin-control right svelte-bica9f"><input type="number"${attr("value", marginRight)} min="0" class="svelte-bica9f"/> <span class="svelte-bica9f">Right</span></div> <div class="margin-control bottom svelte-bica9f"><input type="number"${attr("value", marginBottom)} min="0" class="svelte-bica9f"/> <span class="svelte-bica9f">Bottom</span></div> <div class="grid-area svelte-bica9f"><!--[-->`);
    const each_array = ensure_array_like(Array(cols * rows));
    for (let i = 0, $$length = each_array.length; i < $$length; i++) {
      each_array[i];
      $$renderer2.push(`<div class="label-cell svelte-bica9f"><div class="label-content svelte-bica9f"><span class="lbl-text">L${escape_html(i + 1)}</span></div></div>`);
    }
    $$renderer2.push(`<!--]--> `);
    {
      $$renderer2.push("<!--[-->");
      $$renderer2.push(`<div class="gap-handle x svelte-bica9f"><span class="svelte-bica9f">↔</span> <input type="number"${attr("value", gapX)} min="0" max="50" step="0.5" class="svelte-bica9f"/></div>`);
    }
    $$renderer2.push(`<!--]--> `);
    {
      $$renderer2.push("<!--[-->");
      $$renderer2.push(`<div class="gap-handle y svelte-bica9f"><span class="svelte-bica9f">↕</span> <input type="number"${attr("value", gapY)} min="0" max="50" step="0.5" class="svelte-bica9f"/></div>`);
    }
    $$renderer2.push(`<!--]--></div></div></div> <div class="dims-info svelte-bica9f">Label Size: <strong>${escape_html(labelDims.w)} x ${escape_html(labelDims.h)} mm</strong></div></div> <div class="card svelte-bica9f"><h2 class="svelte-bica9f">Select Template</h2> <div class="type-grid svelte-bica9f"><button${attr_class("type-card svelte-bica9f", void 0, { "active": selectedType === "i" })}><h3 class="svelte-bica9f">Items</h3></button> <button${attr_class("type-card svelte-bica9f", void 0, { "active": selectedType === "b" })}><h3 class="svelte-bica9f">Boxes</h3></button> <button${attr_class("type-card svelte-bica9f", void 0, { "active": selectedType === "p" })}><h3 class="svelte-bica9f">Places</h3></button> <button${attr_class("type-card svelte-bica9f", void 0, { "active": selectedType === "l" })}><h3 class="svelte-bica9f">Labels</h3></button></div></div> `);
    {
      $$renderer2.push("<!--[!-->");
    }
    $$renderer2.push(`<!--]--> <div class="card svelte-bica9f"><h2 class="svelte-bica9f">Content Positioning</h2> <div class="styling-layout svelte-bica9f"><div class="preview-box svelte-bica9f"><h3 style="color: #888; font-size: 0.9rem; margin: 0;">Live Preview (${escape_html(layoutKey)})</h3> <div class="label-preview svelte-bica9f"${attr_style(`aspect-ratio: ${stringify(labelDims.aspect)};`)}>`);
    {
      $$renderer2.push("<!--[!-->");
      $$renderer2.push(`<div style="position: absolute; top: 50%; left: 50%; transform: translate(-50%, -50%); color: #888; text-align: center;"><p style="margin: 0; font-size: 0.9rem;">Select a label type above</p></div>`);
    }
    $$renderer2.push(`<!--]--></div> <p class="preview-hint svelte-bica9f">Actual size: ${escape_html(labelDims.w)} x ${escape_html(labelDims.h)} mm</p></div> <div class="styling-controls svelte-bica9f">`);
    {
      $$renderer2.push("<!--[!-->");
      $$renderer2.push(`<div style="padding: 40px 20px; text-align: center; color: #666;"><p style="margin: 0; font-size: 0.9rem;">Select a label type to customize positioning</p></div>`);
    }
    $$renderer2.push(`<!--]--></div></div></div> <div class="actions svelte-bica9f"><button class="btn primary large svelte-bica9f"${attr("disabled", !selectedType, true)}>${escape_html("Generate PDF")}</button></div></div>`);
  });
}
export {
  _page as default
};
