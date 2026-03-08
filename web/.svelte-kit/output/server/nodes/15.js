

export const index = 15;
let component_cache;
export const component = async () => component_cache ??= (await import('../entries/pages/dashboard/shipping/_page.svelte.js')).default;
export const universal = {
  "ssr": false,
  "prerender": false,
  "load": null
};
export const universal_id = "src/routes/dashboard/shipping/+page.js";
export const imports = ["i/immutable/nodes/15.AOHUYilP.js","i/immutable/chunks/DNXyEpYL.js","i/immutable/chunks/DV1hcMbH.js","i/immutable/chunks/-lFqnQEu.js","i/immutable/chunks/CbcCQL3D.js","i/immutable/chunks/BSjqXs2U.js","i/immutable/chunks/BMGWN91I.js","i/immutable/chunks/DWfxUpmd.js","i/immutable/chunks/Bp3jHsEm.js","i/immutable/chunks/N8Tn0IXF.js","i/immutable/chunks/C-0jhG_O.js","i/immutable/chunks/CcY_sBwz.js","i/immutable/chunks/Bfc47y5P.js","i/immutable/chunks/CEtTWV9_.js","i/immutable/chunks/BTApk60I.js","i/immutable/chunks/B0WR_3JV.js","i/immutable/chunks/Purm7nLw.js","i/immutable/chunks/7yDM04DU.js"];
export const stylesheets = ["i/immutable/assets/15.DEiSQ6NF.css"];
export const fonts = [];
