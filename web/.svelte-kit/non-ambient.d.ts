
// this file is generated — do not edit it


declare module "svelte/elements" {
	export interface HTMLAttributes<T> {
		'data-sveltekit-keepfocus'?: true | '' | 'off' | undefined | null;
		'data-sveltekit-noscroll'?: true | '' | 'off' | undefined | null;
		'data-sveltekit-preload-code'?:
			| true
			| ''
			| 'eager'
			| 'viewport'
			| 'hover'
			| 'tap'
			| 'off'
			| undefined
			| null;
		'data-sveltekit-preload-data'?: true | '' | 'hover' | 'tap' | 'off' | undefined | null;
		'data-sveltekit-reload'?: true | '' | 'off' | undefined | null;
		'data-sveltekit-replacestate'?: true | '' | 'off' | undefined | null;
	}
}

export {};


declare module "$app/types" {
	export interface AppTypes {
		RouteId(): "/" | "/dashboard" | "/dashboard/devices" | "/dashboard/items" | "/dashboard/items/[id]" | "/dashboard/print" | "/dashboard/rma" | "/dashboard/rma/[id]" | "/dashboard/shipping" | "/dashboard/users" | "/dashboard/warehouse" | "/dashboard/warehouse/blueprint" | "/dashboard/warehouse/[id]" | "/login";
		RouteParams(): {
			"/dashboard/items/[id]": { id: string };
			"/dashboard/rma/[id]": { id: string };
			"/dashboard/warehouse/[id]": { id: string }
		};
		LayoutParams(): {
			"/": { id?: string };
			"/dashboard": { id?: string };
			"/dashboard/devices": Record<string, never>;
			"/dashboard/items": { id?: string };
			"/dashboard/items/[id]": { id: string };
			"/dashboard/print": Record<string, never>;
			"/dashboard/rma": { id?: string };
			"/dashboard/rma/[id]": { id: string };
			"/dashboard/shipping": Record<string, never>;
			"/dashboard/users": Record<string, never>;
			"/dashboard/warehouse": { id?: string };
			"/dashboard/warehouse/blueprint": Record<string, never>;
			"/dashboard/warehouse/[id]": { id: string };
			"/login": Record<string, never>
		};
		Pathname(): "/" | "/dashboard" | "/dashboard/devices" | "/dashboard/items" | `/dashboard/items/${string}` & {} | "/dashboard/print" | "/dashboard/rma" | `/dashboard/rma/${string}` & {} | "/dashboard/shipping" | "/dashboard/users" | "/dashboard/warehouse" | "/dashboard/warehouse/blueprint" | `/dashboard/warehouse/${string}` & {} | "/login";
		ResolvedPathname(): `${"" | `/${string}`}${ReturnType<AppTypes['Pathname']>}`;
		Asset(): string & {};
	}
}