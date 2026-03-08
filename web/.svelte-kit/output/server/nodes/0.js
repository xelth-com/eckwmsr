

export const index = 0;
let component_cache;
export const component = async () => component_cache ??= (await import('../entries/pages/_layout.svelte.js')).default;
export const universal = {
  "ssr": false,
  "prerender": false
};
export const universal_id = "src/routes/+layout.js";
export const imports = ["i/immutable/nodes/0.DFXxNZ-5.js","i/immutable/chunks/BSjqXs2U.js","i/immutable/chunks/DV1hcMbH.js","i/immutable/chunks/BMGWN91I.js","i/immutable/chunks/BnwrctrY.js","i/immutable/chunks/DhKVu9yT.js","i/immutable/chunks/JMgvFkvy.js"];
export const stylesheets = ["i/immutable/assets/0.DjzaIuau.css"];
export const fonts = [];
