

export const index = 0;
let component_cache;
export const component = async () => component_cache ??= (await import('../entries/pages/_layout.svelte.js')).default;
export const universal = {
  "ssr": false,
  "prerender": false
};
export const universal_id = "src/routes/+layout.js";
export const imports = ["i/immutable/nodes/0.Fe5qM6T6.js","i/immutable/chunks/BygaA0WS.js","i/immutable/chunks/CN5Z31N9.js","i/immutable/chunks/BqIW1Qmb.js","i/immutable/chunks/BHSAXxoa.js","i/immutable/chunks/CgTr7O7k.js","i/immutable/chunks/BODtoanW.js"];
export const stylesheets = ["i/immutable/assets/0.DjzaIuau.css"];
export const fonts = [];
