import { g as get } from "./index.js";
import { a as authStore } from "./authStore.js";
import { b as base } from "./server.js";
import "./url.js";
import "@sveltejs/kit/internal/server";
import "./root.js";
const BASE_URL = base || "/E";
let isRefreshing = false;
let failedQueue = [];
const processQueue = (error, token = null) => {
  failedQueue.forEach((prom) => {
    if (error) {
      prom.reject(error);
    } else {
      prom.resolve(token);
    }
  });
  failedQueue = [];
};
function redirectToLogin() {
  if (typeof window !== "undefined") {
    let basePath = "/E";
    if (window.location.pathname.includes("/dashboard")) {
      basePath = window.location.pathname.split("/dashboard")[0] || "/E";
    }
    window.location.href = `${basePath}/login`;
  }
}
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
  let response = await fetch(`${BASE_URL}${endpoint}`, config);
  if (response.status === 401) {
    const originalRequestConfig = config;
    if (isRefreshing) {
      return new Promise(function(resolve, reject) {
        failedQueue.push({ resolve, reject });
      }).then((token) => {
        originalRequestConfig.headers["Authorization"] = `Bearer ${token}`;
        return fetch(`${BASE_URL}${endpoint}`, originalRequestConfig).then(handleResponse);
      }).catch((err) => {
        throw err;
      });
    }
    const refreshToken = typeof window !== "undefined" ? localStorage.getItem("refresh_token") : null;
    if (!refreshToken) {
      authStore.logout();
      redirectToLogin();
      throw new Error("Unauthorized");
    }
    isRefreshing = true;
    try {
      const refreshRes = await fetch(`${BASE_URL}/auth/refresh`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ refreshToken })
      });
      if (!refreshRes.ok) {
        throw new Error("Refresh token invalid or expired");
      }
      const data = await refreshRes.json();
      authStore.setTokens(data.tokens.accessToken, data.tokens.refreshToken, data.user);
      processQueue(null, data.tokens.accessToken);
      originalRequestConfig.headers["Authorization"] = `Bearer ${data.tokens.accessToken}`;
      response = await fetch(`${BASE_URL}${endpoint}`, originalRequestConfig);
    } catch (err) {
      processQueue(err, null);
      authStore.logout();
      redirectToLogin();
      throw new Error("Session expired");
    } finally {
      isRefreshing = false;
    }
  }
  return handleResponse(response);
}
async function handleResponse(response) {
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
