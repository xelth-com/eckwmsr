import { g as get } from "./index.js";
import { a as authStore } from "./authStore.js";
import { b as base } from "./server.js";
import "./url.js";
import "@sveltejs/kit/internal/server";
import "./root.js";
const BASE_URL = base || "/E";
async function request(endpoint, options = {}) {
  const state = get(authStore);
  const headers = {
    "Content-Type": "application/json",
    ...options.headers
  };
  if (state.token) {
    headers["Authorization"] = `Bearer ${state.token}`;
  }
  const config = {
    ...options,
    headers
  };
  const response = await fetch(`${BASE_URL}${endpoint}`, config);
  if (response.status === 401) {
    authStore.logout();
    if (typeof window !== "undefined") {
      let basePath = "/E";
      if (window.location.pathname.includes("/dashboard")) {
        basePath = window.location.pathname.split("/dashboard")[0] || "/E";
      }
      window.location.href = `${basePath}/login`;
    }
    throw new Error("Unauthorized");
  }
  if (!response.ok) {
    const errorData = await response.json().catch(() => ({}));
    throw new Error(errorData.error || `Request failed: ${response.status}`);
  }
  return response.json();
}
const api = {
  get: (endpoint) => request(endpoint, { method: "GET" }),
  post: (endpoint, body) => request(endpoint, { method: "POST", body: JSON.stringify(body) }),
  put: (endpoint, body) => request(endpoint, { method: "PUT", body: JSON.stringify(body) }),
  delete: (endpoint) => request(endpoint, { method: "DELETE" })
};
export {
  api as a
};
