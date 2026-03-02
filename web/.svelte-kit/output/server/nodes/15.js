

export const index = 15;
let component_cache;
export const component = async () => component_cache ??= (await import('../entries/pages/dashboard/shipping/_page.svelte.js')).default;
export const universal = {
  "ssr": false,
  "prerender": false,
  "load": null
};
export const universal_id = "src/routes/dashboard/shipping/+page.js";
export const imports = ["i/immutable/nodes/15.Dkjg3ln7.js","i/immutable/chunks/DmAc8qp0.js","i/immutable/chunks/C1ffpMTo.js","i/immutable/chunks/DcaLnOSn.js","i/immutable/chunks/C-MoLchR.js","i/immutable/chunks/D6A3UyO8.js","i/immutable/chunks/D7BOgdYp.js","i/immutable/chunks/BOAnDIdj.js","i/immutable/chunks/D58zJJ-1.js","i/immutable/chunks/DpZacG7h.js","i/immutable/chunks/DiJA1Qsa.js","i/immutable/chunks/B3AdoyCx.js","i/immutable/chunks/Bfc47y5P.js","i/immutable/chunks/BchGmecV.js","i/immutable/chunks/_OKctHgU.js","i/immutable/chunks/BDuJKAP8.js","i/immutable/chunks/ndQYkU6X.js","i/immutable/chunks/bXK4EGwz.js"];
export const stylesheets = ["i/immutable/assets/15.DEiSQ6NF.css"];
export const fonts = [];
