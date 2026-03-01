

export const index = 0;
let component_cache;
export const component = async () => component_cache ??= (await import('../entries/pages/_layout.svelte.js')).default;
export const universal = {
  "ssr": false,
  "prerender": false
};
export const universal_id = "src/routes/+layout.js";
export const imports = ["i/immutable/nodes/0.DA2UdnU7.js","i/immutable/chunks/iR2dgGIM.js","i/immutable/chunks/5f1SCnuC.js","i/immutable/chunks/C5gafh_b.js","i/immutable/chunks/CMZYMwr5.js","i/immutable/chunks/Dc09eRik.js","i/immutable/chunks/Tb3oZlf1.js"];
export const stylesheets = ["i/immutable/assets/0.DjzaIuau.css"];
export const fonts = [];
