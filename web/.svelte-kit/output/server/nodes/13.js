

export const index = 13;
let component_cache;
export const component = async () => component_cache ??= (await import('../entries/pages/dashboard/scrapers/_page.svelte.js')).default;
export const universal = {
  "ssr": false,
  "prerender": false,
  "load": null
};
export const universal_id = "src/routes/dashboard/scrapers/+page.js";
export const imports = ["i/immutable/nodes/13.fpCiijUI.js","i/immutable/chunks/Gs9S95rt.js","i/immutable/chunks/5f1SCnuC.js","i/immutable/chunks/BGirObW-.js","i/immutable/chunks/Cc9qOar4.js","i/immutable/chunks/iR2dgGIM.js","i/immutable/chunks/C5gafh_b.js","i/immutable/chunks/C-IVEp84.js","i/immutable/chunks/Dwgbs3Qr.js","i/immutable/chunks/lnrZtdjg.js","i/immutable/chunks/BfuDPpiA.js","i/immutable/chunks/DfzfxLnL.js","i/immutable/chunks/BzJTQ2Am.js","i/immutable/chunks/Bfc47y5P.js","i/immutable/chunks/CwI3xiQG.js","i/immutable/chunks/CgIgtQJU.js","i/immutable/chunks/C0YZ9jco.js","i/immutable/chunks/gZ7TAWYC.js"];
export const stylesheets = ["i/immutable/assets/13.BRq7KfBt.css"];
export const fonts = [];
