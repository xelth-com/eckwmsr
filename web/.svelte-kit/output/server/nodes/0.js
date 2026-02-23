

export const index = 0;
let component_cache;
export const component = async () => component_cache ??= (await import('../entries/pages/_layout.svelte.js')).default;
export const universal = {
  "ssr": false,
  "prerender": false
};
export const universal_id = "src/routes/+layout.js";
export const imports = ["i/immutable/nodes/0.D4po7dJg.js","i/immutable/chunks/J38xHHLL.js","i/immutable/chunks/D1u8ExaR.js","i/immutable/chunks/B-WR732x.js","i/immutable/chunks/BICEQZNp.js","i/immutable/chunks/KFlKTQAS.js","i/immutable/chunks/Dz5WjnMf.js"];
export const stylesheets = ["i/immutable/assets/0.DjzaIuau.css"];
export const fonts = [];
