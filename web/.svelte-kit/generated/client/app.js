export { matchers } from './matchers.js';

export const nodes = [
	() => import('./nodes/0'),
	() => import('./nodes/1'),
	() => import('./nodes/2'),
	() => import('./nodes/3'),
	() => import('./nodes/4'),
	() => import('./nodes/5'),
	() => import('./nodes/6'),
	() => import('./nodes/7'),
	() => import('./nodes/8'),
	() => import('./nodes/9'),
	() => import('./nodes/10'),
	() => import('./nodes/11'),
	() => import('./nodes/12'),
	() => import('./nodes/13'),
	() => import('./nodes/14'),
	() => import('./nodes/15'),
	() => import('./nodes/16'),
	() => import('./nodes/17'),
	() => import('./nodes/18'),
	() => import('./nodes/19'),
	() => import('./nodes/20'),
	() => import('./nodes/21'),
	() => import('./nodes/22')
];

export const server_loads = [];

export const dictionary = {
		"/": [3],
		"/dashboard": [4,[2]],
		"/dashboard/analysis": [5,[2]],
		"/dashboard/devices": [6,[2]],
		"/dashboard/items": [7,[2]],
		"/dashboard/items/[id]": [8,[2]],
		"/dashboard/print": [9,[2]],
		"/dashboard/repairs": [10,[2]],
		"/dashboard/repairs/[id]": [11,[2]],
		"/dashboard/rma": [12,[2]],
		"/dashboard/rma/[id]": [13,[2]],
		"/dashboard/scrapers": [14,[2]],
		"/dashboard/shipping": [15,[2]],
		"/dashboard/support": [16,[2]],
		"/dashboard/support/[id]": [17,[2]],
		"/dashboard/users": [18,[2]],
		"/dashboard/warehouse": [19,[2]],
		"/dashboard/warehouse/blueprint": [20,[2]],
		"/dashboard/warehouse/[id]": [21,[2]],
		"/login": [22]
	};

export const hooks = {
	handleError: (({ error }) => { console.error(error) }),
	
	reroute: (() => {}),
	transport: {}
};

export const decoders = Object.fromEntries(Object.entries(hooks.transport).map(([k, v]) => [k, v.decode]));
export const encoders = Object.fromEntries(Object.entries(hooks.transport).map(([k, v]) => [k, v.encode]));

export const hash = false;

export const decode = (type, value) => decoders[type](value);

export { default as root } from '../root.js';