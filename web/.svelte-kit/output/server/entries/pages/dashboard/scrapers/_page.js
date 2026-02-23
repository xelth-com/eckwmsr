import { a as api } from "../../../../chunks/api.js";
async function load() {
  try {
    const [syncHistory] = await Promise.all([
      api.get("/api/delivery/sync/history")
    ]);
    return {
      syncHistory: syncHistory || []
    };
  } catch (e) {
    console.error("Load error:", e);
    return {
      syncHistory: [],
      error: e.message
    };
  }
}
export {
  load
};
