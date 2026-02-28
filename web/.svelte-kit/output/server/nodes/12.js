

export const index = 12;
let component_cache;
export const component = async () => component_cache ??= (await import('../entries/pages/dashboard/shipping/_page.svelte.js')).default;
export const universal = {
  "ssr": false,
  "prerender": false,
  "load": null
};
export const universal_id = "src/routes/dashboard/shipping/+page.js";
export const imports = ["i/immutable/nodes/12.dKOZ9wHE.js","i/immutable/chunks/B1TkV2Nx.js","i/immutable/chunks/5f1SCnuC.js","i/immutable/chunks/cODhT-3T.js","i/immutable/chunks/DMSQfz0-.js","i/immutable/chunks/iR2dgGIM.js","i/immutable/chunks/C5gafh_b.js","i/immutable/chunks/C-IVEp84.js","i/immutable/chunks/Dwgbs3Qr.js","i/immutable/chunks/lnrZtdjg.js","i/immutable/chunks/BZrB3iCd.js","i/immutable/chunks/Bfc47y5P.js","i/immutable/chunks/CwI3xiQG.js","i/immutable/chunks/CgIgtQJU.js","i/immutable/chunks/C0YZ9jco.js","i/immutable/chunks/gZ7TAWYC.js"];
export const stylesheets = ["i/immutable/assets/12.bpuuUHI-.css"];
export const fonts = [];
