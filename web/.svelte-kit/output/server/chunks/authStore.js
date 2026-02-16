import { w as writable } from "./index.js";
import { B as BROWSER } from "./false.js";
import { b as base } from "./server.js";
import "./url.js";
import "@sveltejs/kit/internal/server";
import "./root.js";
const initialState = {
  isAuthenticated: false,
  currentUser: null,
  token: null,
  isLoading: true
};
function createAuthStore() {
  const { subscribe, set, update } = writable(initialState);
  return {
    subscribe,
    init: () => {
      return;
    },
    login: async (email, password) => {
      update((s) => ({ ...s, isLoading: true }));
      try {
        const res = await fetch(`${base}/auth/login`, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ Email: email, Password: password })
        });
        if (!res.ok) throw new Error("Login failed");
        const data = await res.json();
        if (BROWSER) ;
        update((s) => ({
          ...s,
          isAuthenticated: true,
          currentUser: data.user,
          token: data.tokens.accessToken,
          isLoading: false
        }));
        return { success: true };
      } catch (e) {
        update((s) => ({ ...s, isLoading: false }));
        return { success: false, error: e.message };
      }
    },
    logout: () => {
      set({ ...initialState, isLoading: false });
    }
  };
}
const authStore = createAuthStore();
export {
  authStore as a
};
