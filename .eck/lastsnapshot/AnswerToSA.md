# Report: Add "Copy for AI" button to Support Ticket detail
**Executor:** Claude Opus 4.6
**Status:** SUCCESS
**Changes:**
- Modified `web/src/routes/dashboard/support/[id]/+page.svelte` — added `copyForAI()` function that strips HTML from threads, prepends AI system prompt, and copies to clipboard
- Added "🤖 Copy for AI" button in `.header-actions` alongside existing AI/RMA/Repair buttons
- Added `.copy-ai-btn` CSS (dark neutral style to distinguish from the purple AI summarize button)
