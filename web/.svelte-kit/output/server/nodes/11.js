

export const index = 11;
let component_cache;
export const component = async () => component_cache ??= (await import('../entries/pages/dashboard/scrapers/_page.svelte.js')).default;
export const universal = {
  "ssr": false,
  "prerender": false,
  "load": null
};
export const universal_id = "src/routes/dashboard/scrapers/+page.js";
export const imports = ["i/immutable/nodes/11.B0SFBxhi.js","i/immutable/chunks/x12HBnBk.js","i/immutable/chunks/D1u8ExaR.js","i/immutable/chunks/KFlKTQAS.js","i/immutable/chunks/Dz5WjnMf.js","i/immutable/chunks/J38xHHLL.js","i/immutable/chunks/B-WR732x.js","i/immutable/chunks/CRDhd8sU.js","i/immutable/chunks/L-br2Pp7.js","i/immutable/chunks/0GYKm5Ee.js","i/immutable/chunks/DW07t846.js","i/immutable/chunks/meR-1Spd.js","i/immutable/chunks/CjLB_7JY.js","i/immutable/chunks/Bfc47y5P.js","i/immutable/chunks/But61IZ0.js","i/immutable/chunks/Cepk95Ci.js","i/immutable/chunks/DQc6JCXP.js","i/immutable/chunks/DDvTgRkA.js"];
export const stylesheets = ["i/immutable/assets/11.BCLUrDr8.css"];
export const fonts = [];
