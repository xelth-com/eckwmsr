

export const index = 0;
let component_cache;
export const component = async () => component_cache ??= (await import('../entries/pages/_layout.svelte.js')).default;
export const universal = {
  "ssr": false,
  "prerender": false
};
export const universal_id = "src/routes/+layout.js";
export const imports = ["i/immutable/nodes/0.BzNoTyE5.js","i/immutable/chunks/CVSPybqJ.js","i/immutable/chunks/CqpwgXEk.js","i/immutable/chunks/BdqhJSGs.js","i/immutable/chunks/NHymQ4I1.js","i/immutable/chunks/DbFcPqlP.js","i/immutable/chunks/CQ3cvpm9.js"];
export const stylesheets = ["i/immutable/assets/0.DjzaIuau.css"];
export const fonts = [];
