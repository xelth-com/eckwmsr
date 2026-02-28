<script>
    import { page } from '$app/stores';
    import { onMount } from 'svelte';
    import { api } from '$lib/api';
    import { goto } from '$app/navigation';
    import { base } from '$app/paths';
    import { toastStore } from '$lib/stores/toastStore';

    let orderId = $page.params.id;
    let isNew = orderId === 'new';
    let loading = !isNew;

    // Form Data
    let formData = {
        orderType: 'repair',
        orderNumber: '',
        customerName: '',
        customerEmail: '',
        productSku: '',
        productName: '',
        serialNumber: '',
        issueDescription: '',
        status: 'pending',
        priority: 'normal',
        repairNotes: '',
        laborHours: 0,
        metadata: {},
    };

    onMount(async () => {
        if (!isNew) {
            await loadRepair();
        } else {
            formData.orderNumber = 'AUTO-GEN';
            // Pre-fill from URL params when coming from a Support ticket
            const params = $page.url.searchParams;
            const linkedTicketId = params.get('ticketId');
            if (linkedTicketId) {
                formData.metadata = { ticketId: linkedTicketId };
                formData.customerName     = params.get('name')  || '';
                formData.customerEmail    = params.get('email') || '';
                formData.issueDescription = params.get('issue') || '';
            }
        }
    });

    async function loadRepair() {
        try {
            const data = await api.get(`/rma/${orderId}`);
            formData = { ...data };
        } catch (e) {
            toastStore.add('Error loading Repair', 'error');
            goto('/dashboard/repairs');
        } finally {
            loading = false;
        }
    }

    async function handleSubmit() {
        try {
            if (isNew) {
                if (!formData.customerName || !formData.productSku) {
                    toastStore.add('Customer Name and Product SKU are required', 'warning');
                    return;
                }
                if (formData.orderNumber === 'AUTO-GEN') delete formData.orderNumber;
                formData.orderType = 'repair';
                formData.laborHours = parseFloat(formData.laborHours) || 0;
                await api.post('/rma', formData);
                toastStore.add('Repair Created Successfully', 'success');
            } else {
                formData.laborHours = parseFloat(formData.laborHours) || 0;
                await api.put(`/rma/${orderId}`, formData);
                toastStore.add('Repair Updated', 'success');
            }
            goto('/dashboard/repairs');
        } catch (e) {
            toastStore.add(`Error: ${e.message}`, 'error');
        }
    }

    async function deleteRepair() {
        if (!confirm('Are you sure you want to delete this Repair Order?')) return;
        try {
            await api.delete(`/rma/${orderId}`);
            toastStore.add('Repair Deleted', 'success');
            goto('/dashboard/repairs');
        } catch (e) {
            toastStore.add(e.message, 'error');
        }
    }

    function goBack() {
        goto('/dashboard/repairs');
    }
</script>

<div class="detail-page">
    <div class="header">
        <button class="back-btn" on:click={goBack}>← Back</button>
        <div class="title-row">
            <h1>{isNew ? 'New Repair Order' : `Repair ${formData.orderNumber}`}</h1>
            {#if !isNew}
                <button class="delete-btn" on:click={deleteRepair}>Delete</button>
            {/if}
        </div>
    </div>

    {#if loading}
        <div class="loading">Loading...</div>
    {:else}
        <form class="form-grid" on:submit|preventDefault={handleSubmit}>
            {#if formData.metadata?.ticketId}
                <div class="section full linked-banner">
                    <div class="linked-row">
                        <span class="linked-label">🔗 Linked Support Ticket</span>
                        <a class="linked-link" href="{base}/dashboard/support/{formData.metadata.ticketId}">
                            #{formData.metadata.ticketId} → View Ticket
                        </a>
                    </div>
                </div>
            {/if}

            <div class="section">
                <h2>Customer Information</h2>
                <div class="field">
                    <label>Customer Name *</label>
                    <input type="text" bind:value={formData.customerName} required />
                </div>
                <div class="field">
                    <label>Email</label>
                    <input type="email" bind:value={formData.customerEmail} />
                </div>
            </div>

            <div class="section">
                <h2>Device Details</h2>
                <div class="field">
                    <label>Device Model / SKU *</label>
                    <input type="text" bind:value={formData.productSku} required class="code-input" />
                </div>
                <div class="field">
                    <label>Serial Number</label>
                    <input type="text" bind:value={formData.serialNumber} class="code-input" />
                </div>
            </div>

            <div class="section full">
                <h2>Issue Description</h2>
                <textarea bind:value={formData.issueDescription} rows="3"></textarea>
            </div>

            <div class="section">
                <h2>Repair Details</h2>
                <div class="field">
                    <label>Labor Hours</label>
                    <input type="number" step="0.1" min="0" bind:value={formData.laborHours} />
                </div>
                <div class="field">
                    <label>Repair Notes (Internal)</label>
                    <textarea bind:value={formData.repairNotes} rows="4"></textarea>
                </div>
            </div>

            <div class="section">
                <h2>Status &amp; Priority</h2>
                <div class="field">
                    <label>Status</label>
                    <select bind:value={formData.status}>
                        <option value="pending">Pending</option>
                        <option value="received">Received</option>
                        <option value="processing">Processing (In Repair)</option>
                        <option value="completed">Completed</option>
                        <option value="cancelled">Cancelled</option>
                    </select>
                </div>
                <div class="field">
                    <label>Priority</label>
                    <select bind:value={formData.priority}>
                        <option value="low">Low</option>
                        <option value="normal">Normal</option>
                        <option value="high">High</option>
                        <option value="urgent">Urgent</option>
                    </select>
                </div>
            </div>

            <div class="actions full">
                <button type="button" class="cancel-btn" on:click={goBack}>Cancel</button>
                <button type="submit" class="save-btn">{isNew ? 'Create Order' : 'Save Changes'}</button>
            </div>
        </form>
    {/if}
</div>

<style>
    .detail-page { max-width: 900px; margin: 0 auto; padding-bottom: 2rem; }
    .header { margin-bottom: 2rem; }
    .back-btn { background: none; border: none; color: #888; cursor: pointer; font-size: 1rem; padding: 0; margin-bottom: 1rem; }
    .back-btn:hover { color: #fff; }
    .title-row { display: flex; justify-content: space-between; align-items: center; }
    h1 { color: #fff; font-size: 2rem; margin: 0; }
    h2 { color: #ccc; font-size: 1.1rem; margin-bottom: 1rem; border-bottom: 1px solid #333; padding-bottom: 0.5rem; }

    .form-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 1.5rem; }
    .section {
        background: #1e1e1e;
        border: 1px solid #333;
        border-radius: 8px;
        padding: 1.5rem;
        display: flex;
        flex-direction: column;
        gap: 1rem;
    }
    .section.full { grid-column: 1 / -1; }
    .field { display: flex; flex-direction: column; gap: 0.5rem; }
    label { color: #888; font-size: 0.85rem; font-weight: 500; }

    input, select, textarea {
        background: #121212;
        border: 1px solid #444;
        border-radius: 4px;
        padding: 0.8rem;
        color: #fff;
        font-size: 1rem;
        font-family: inherit;
    }
    input:focus, select:focus, textarea:focus { border-color: #4a69bd; outline: none; }
    .code-input { font-family: monospace; }

    .actions { margin-top: 1rem; display: flex; justify-content: flex-end; gap: 1rem; }
    .save-btn {
        background: #28a745;
        color: white;
        border: none;
        padding: 0.8rem 2rem;
        border-radius: 4px;
        font-weight: 600;
        cursor: pointer;
    }
    .save-btn:hover { background: #218838; }
    .cancel-btn {
        background: #333;
        color: #ccc;
        border: none;
        padding: 0.8rem 1.5rem;
        border-radius: 4px;
        cursor: pointer;
    }
    .cancel-btn:hover { background: #444; }
    .delete-btn {
        background: #d32f2f;
        color: white;
        border: none;
        padding: 0.6rem 1.2rem;
        border-radius: 4px;
        cursor: pointer;
    }

    .linked-banner { background: #1a2a3a; border-color: #3b82f6; padding: 0.9rem 1.25rem; }
    .linked-row { display: flex; align-items: center; gap: 1rem; flex-wrap: wrap; }
    .linked-label { color: #93c5fd; font-weight: 600; font-size: 0.9rem; }
    .linked-link {
        color: #bfdbfe;
        text-decoration: none;
        font-family: monospace;
        font-size: 0.85rem;
        border-bottom: 1px dashed #4a69bd;
    }
    .linked-link:hover { color: #fff; border-bottom-color: #fff; }

    @media (max-width: 700px) { .form-grid { grid-template-columns: 1fr; } }
</style>
