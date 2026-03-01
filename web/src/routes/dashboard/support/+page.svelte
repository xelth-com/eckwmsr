<script>
    import { onMount } from 'svelte';
    import { api } from '$lib/api';
    import { goto } from '$app/navigation';
    import { base } from '$app/paths';
    import { toastStore } from '$lib/stores/toastStore.js';

    let tickets = [];
    let loading = true;
    let error = null;

    onMount(async () => {
        await loadTickets();
    });

    async function loadTickets() {
        loading = true;
        error = null;
        try {
            tickets = await api.get('/api/support/tickets');
        } catch (e) {
            console.error(e);
            error = e.message;
            toastStore.add('Failed to load support tickets', 'error');
        } finally {
            loading = false;
        }
    }

    function openTicket(ticketId) {
        goto(`${base}/dashboard/support/${ticketId}`);
    }

    function formatDate(str) {
        if (!str) return '-';
        try {
            return new Date(str).toLocaleString('de-DE', {
                day: '2-digit', month: '2-digit', year: 'numeric',
                hour: '2-digit', minute: '2-digit'
            });
        } catch { return str; }
    }

    function statusClass(status) {
        const s = (status || '').toLowerCase();
        if (s === 'open') return 'open';
        if (s === 'closed') return 'closed';
        if (s.includes('pending agent')) return 'urgent';
        if (s.includes('research')) return 'research';
        if (s === 'onhold' || s === 'on hold') return 'onhold';
        return 'other';
    }
</script>

<div class="support-page">
    <header>
        <h1>Support Tickets</h1>
        <button class="refresh-btn" on:click={loadTickets} disabled={loading}>
            {loading ? 'Loading...' : 'Refresh'}
        </button>
    </header>

    {#if loading}
        <div class="loading">Loading tickets...</div>
    {:else if error}
        <div class="error-box">Failed to load: {error}</div>
    {:else if tickets.length === 0}
        <div class="empty-state">
            <p>No support tickets imported yet</p>
            <small>Use the Scrapers page - Zoho Desk - Fetch Threads - Save to System</small>
        </div>
    {:else}
        <div class="table-container">
            <table>
                <thead>
                    <tr>
                        <th>Ticket #</th>
                        <th>Subject / Device</th>
                        <th>Customer</th>
                        <th>Status</th>
                        <th class="center">Threads</th>
                        <th>Latest Update</th>
                    </tr>
                </thead>
                <tbody>
                    {#each tickets as ticket}
                        <tr class="ticket-row" on:click={() => openTicket(ticket.ticket_id)}>
                            <td class="mono highlight">#{ticket.ticket_number || ticket.ticket_id.substring(0,8)}</td>
                            <td class="subject-cell">
                                <div class="subject">{ticket.subject}</div>
                                {#if ticket.device_model || ticket.serial_number}
                                    <div class="device-badge">
                                        {#if ticket.device_model}{ticket.device_model}{/if}
                                        {#if ticket.device_model && ticket.serial_number} | {/if}
                                        {#if ticket.serial_number}SN: <span class="mono">{ticket.serial_number}</span>{/if}
                                    </div>
                                {/if}
                            </td>
                            <td class="customer-cell">
                                <div class="c-name">{ticket.customer || 'Unknown'}</div>
                                {#if ticket.company}<div class="c-company">{ticket.company}</div>{/if}
                                {#if ticket.email || ticket.phone}
                                    <div class="c-contact">
                                        {#if ticket.email}<span class="c-email">{ticket.email}</span>{/if}
                                        {#if ticket.phone}<span class="c-phone">{ticket.phone}</span>{/if}
                                    </div>
                                {/if}
                            </td>
                            <td>
                                <span class="status-badge {statusClass(ticket.status)}">
                                    {ticket.status}
                                </span>
                            </td>
                            <td class="center mono">{ticket.thread_count}</td>
                            <td class="mono date">{formatDate(ticket.latest_update)}</td>
                        </tr>
                    {/each}
                </tbody>
            </table>
        </div>
    {/if}
</div>

<style>
    .support-page { padding: 0; }

    header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 1.5rem;
    }
    h1 { font-size: 1.8rem; color: #fff; margin: 0; }

    .refresh-btn {
        padding: 0.6rem 1.2rem;
        border-radius: 4px;
        border: 1px solid #4a69bd;
        background: transparent;
        color: #4a69bd;
        font-weight: 600;
        cursor: pointer;
        transition: all 0.2s;
    }
    .refresh-btn:hover:not(:disabled) { background: #4a69bd; color: white; }
    .refresh-btn:disabled { opacity: 0.5; cursor: not-allowed; }

    .loading { color: #aaa; text-align: center; padding: 3rem; }

    .error-box {
        text-align: center;
        padding: 2rem;
        color: #ff6b6b;
        background: #1e1e1e;
        border: 1px solid #ff6b6b;
        border-radius: 8px;
    }

    .empty-state {
        text-align: center;
        padding: 4rem 2rem;
        color: #666;
        background: #1e1e1e;
        border-radius: 8px;
        border: 1px dashed #333;
    }
    .empty-state p { font-size: 1.2rem; margin: 0 0 0.5rem 0; color: #aaa; }
    .empty-state small { color: #555; }

    .table-container {
        background: #1e1e1e;
        border-radius: 8px;
        border: 1px solid #333;
        overflow-x: auto;
    }
    table { width: 100%; border-collapse: collapse; }
    thead { background: #252525; }
    th {
        padding: 0.9rem 1rem;
        text-align: left;
        font-weight: 600;
        color: #aaa;
        text-transform: uppercase;
        font-size: 0.73rem;
        letter-spacing: 0.5px;
        border-bottom: 2px solid #333;
    }
    td {
        padding: 0.9rem 1rem;
        border-bottom: 1px solid #2a2a2a;
        color: #e0e0e0;
        font-size: 0.9rem;
    }

    .ticket-row { cursor: pointer; transition: background 0.15s; }
    .ticket-row:hover { background: #252525; }
    .ticket-row:last-child td { border-bottom: none; }

    .mono { font-family: monospace; }
    .highlight { color: #6bc5f0; font-weight: bold; }
    .date { color: #888; font-size: 0.85rem; }
    .center { text-align: center; }
    .subject-cell { display: flex; flex-direction: column; gap: 0.4rem; max-width: 380px; }
    .subject { font-weight: 500; color: #fff; line-height: 1.4; }
    .device-badge { font-size: 0.75rem; color: #a3bffa; background: #1a2a4a; padding: 0.2rem 0.5rem; border-radius: 4px; display: inline-block; width: fit-content; border: 1px solid #4a69bd; }

    .customer-cell { display: flex; flex-direction: column; gap: 0.25rem; }
    .c-name { font-weight: 600; color: #ccc; }
    .c-company { font-size: 0.8rem; color: #fbbf24; }
    .c-contact { display: flex; flex-direction: column; gap: 0.15rem; font-size: 0.75rem; color: #888; }
    .c-email, .c-phone { white-space: nowrap; }

    .status-badge {
        display: inline-block;
        padding: 0.2rem 0.7rem;
        border-radius: 12px;
        font-size: 0.75rem;
        font-weight: 600;
        text-transform: capitalize;
    }
    .status-badge.open    { background: #1a3a1a; color: #4ade80; border: 1px solid #22c55e; }
    .status-badge.closed  { background: #2a2a2a; color: #888;    border: 1px solid #444; }
    .status-badge.onhold  { background: #3a2a0a; color: #fbbf24; border: 1px solid #f59e0b; }
    .status-badge.urgent  { background: #4a1010; color: #ff6b6b; border: 1px solid #dc3545; }
    .status-badge.research { background: #1a2a4a; color: #a3bffa; border: 1px solid #4a69bd; }
    .status-badge.other   { background: #1a2a3a; color: #93c5fd; border: 1px solid #3b82f6; }
</style>
