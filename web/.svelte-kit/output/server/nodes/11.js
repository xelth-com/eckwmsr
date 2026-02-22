

export const index = 11;
let component_cache;
export const component = async () => component_cache ??= (await import('../entries/pages/dashboard/shipping/_page.svelte.js')).default;
export const universal = {
  "ssr": false,
  "prerender": false,
  "load": null
};
export const universal_id = "src/routes/dashboard/shipping/+page.js";
export const imports = ["i/immutable/nodes/11.nMupBl7t.js","i/immutable/chunks/aC8y2JMS.js","i/immutable/chunks/CN5Z31N9.js","i/immutable/chunks/BQPPeqpU.js","i/immutable/chunks/7no41MLp.js","i/immutable/chunks/BygaA0WS.js","i/immutable/chunks/BqIW1Qmb.js","i/immutable/chunks/DAV6nA3v.js","i/immutable/chunks/CU63fefz.js","i/immutable/chunks/DFBiLcyC.js","i/immutable/chunks/BZk0zrkd.js","i/immutable/chunks/DZHojTuh.js","i/immutable/chunks/De01lyDX.js","i/immutable/chunks/DwKrkxP9.js","i/immutable/chunks/Bfc47y5P.js","i/immutable/chunks/gYmV0BzS.js","i/immutable/chunks/DycCOpGa.js"];
export const stylesheets = ["i/immutable/assets/11.c7wo1a4e.css"];
export const fonts = [];
