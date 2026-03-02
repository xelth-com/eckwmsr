

export const index = 0;
let component_cache;
export const component = async () => component_cache ??= (await import('../entries/pages/_layout.svelte.js')).default;
export const universal = {
  "ssr": false,
  "prerender": false
};
export const universal_id = "src/routes/+layout.js";
export const imports = ["i/immutable/nodes/0.DF92SBH2.js","i/immutable/chunks/D6A3UyO8.js","i/immutable/chunks/C1ffpMTo.js","i/immutable/chunks/D7BOgdYp.js","i/immutable/chunks/BUyhFv1w.js","i/immutable/chunks/D-cQCM4x.js","i/immutable/chunks/C8eyfVEy.js"];
export const stylesheets = ["i/immutable/assets/0.DjzaIuau.css"];
export const fonts = [];
