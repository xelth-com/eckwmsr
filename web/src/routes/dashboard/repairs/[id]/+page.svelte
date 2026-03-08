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
        partsUsed:[],
        metadata: {},
    };

    // System keys to hide from dynamic attributes list
    const hiddenMetaKeys =['ticketId', 'trackingNumber', 'importedFromExcel', 'excelRow'];

    // State for new custom fields
    let newFieldKey = '';
    let newFieldValue = '';
    let newPart = '';

    onMount(async () => {
        if (!isNew) {
            await loadRepair();
        } else {
            formData.orderNumber = 'AUTO-GEN';
            // Pre-fill from URL params when coming from a Support ticket
            const params = $page.url.searchParams;
            const linkedTicketId = params.get('ticketId');
            const linkedTracking = params.get('tracking');

            if (linkedTicketId) {
                formData.metadata = { ...formData.metadata, ticketId: linkedTicketId };
                formData.customerName     = params.get('name')  || '';
                formData.customerEmail    = params.get('email') || '';
                formData.issueDescription = params.get('issue') || '';
            }
            if (linkedTracking) {
                formData.metadata = { ...formData.metadata, trackingNumber: linkedTracking };
                if (!formData.customerName) formData.customerName = params.get('name') || '';
                if (!formData.issueDescription) formData.issueDescription = params.get('issue') || '';
            }

            const linkedSerial = params.get('serial');
            const linkedModel = params.get('model');
            if (linkedSerial) formData.serialNumber = linkedSerial;
            if (linkedModel) formData.productSku = linkedModel;
        }
    });

    async function loadRepair() {
        try {
            const data = await api.get(`/rma/${orderId}`);
            formData = { ...data };
            if (!Array.isArray(formData.partsUsed)) {
                formData.partsUsed =[];
            }
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

    // --- Dynamic Fields Logic ---

    function formatKey(key) {
        // camelCase to Title Case
        const result = key.replace(/([A-Z])/g, ' $1');
        return result.charAt(0).toUpperCase() + result.slice(1);
    }

    function addPart() {
        if (!newPart.trim()) return;
        formData.partsUsed =[...formData.partsUsed, newPart.trim()];
        newPart = '';
    }

    function removePart(index) {
        formData.partsUsed = formData.partsUsed.filter((_, i) => i !== index);
    }

    function addCustomField() {
        if (!newFieldKey.trim()) return;
        let key = newFieldKey.trim().replace(/\s+/g, '_');
        // Simple type inference
        let val = newFieldValue;
        if (val.toLowerCase() === 'true') val = true;
        if (val.toLowerCase() === 'false') val = false;
        if (!isNaN(val) && val !== '') val = Number(val);

        formData.metadata = { ...formData.metadata, [key]: val };
        newFieldKey = '';
        newFieldValue = '';
    }

    function updateMetadata(key, value) {
        formData.metadata = { ...formData.metadata, [key]: value };
    }

    function updateNestedMetadata(parentKey, childKey, value) {
        formData.metadata = {
            ...formData.metadata,
            [parentKey]: {
                ...formData.metadata[parentKey],
                [childKey]: value
            }
        };
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
            {#if formData.metadata?.ticketId || formData.metadata?.trackingNumber}
                <div class="section full linked-banner">
                    <div class="linked-row">
                        {#if formData.metadata?.ticketId}
                            <span class="linked-label">Linked Support Ticket</span>
                            <a class="linked-link" href="{base}/dashboard/support/{formData.metadata.ticketId}">
                                #{formData.metadata.ticketId} -> View Ticket
                            </a>
                        {/if}
                        {#if formData.metadata?.trackingNumber}
                            <span class="linked-label" style="margin-left: 1rem;">Linked Shipment</span>
                            <span class="linked-link" style="border-bottom: none; color: #fff; cursor: default;">
                                {formData.metadata.trackingNumber}
                            </span>
                        {/if}
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

            <!-- Replaced Parts Section -->
            <div class="section full">
                <div class="section-header">
                    <h2>Replaced Parts</h2>
                </div>
                <div class="tags-container">
                    {#each formData.partsUsed as part, i}
                        <span class="part-tag">
                            {part}
                            <button type="button" class="remove-tag" on:click={() => removePart(i)}>&times;</button>
                        </span>
                    {/each}
                </div>
                <div class="add-tag-row">
                    <input type="text" bind:value={newPart} placeholder="Scan or type part name..." on:keydown={(e) => e.key === 'Enter' && (e.preventDefault(), addPart())} />
                    <button type="button" class="btn secondary" on:click={addPart}>Add Part</button>
                </div>
            </div>

            <!-- Dynamic Attributes (Metadata) -->
            <div class="section full dynamic-section">
                <div class="section-header">
                    <h2>Dynamic Attributes</h2>
                    <span class="badge metadata-badge">Metadata</span>
                </div>
                <p class="section-hint">Device-specific parameters imported from Excel or generated by AI schemas.</p>

                <div class="dynamic-grid">
                    {#each Object.entries(formData.metadata || {}).filter(([k]) => !hiddenMetaKeys.includes(k)) as [key, value]}
                        {#if typeof value === 'object' && value !== null && !Array.isArray(value)}
                            <div class="nested-group">
                                <h4>{formatKey(key)}</h4>
                                <div class="nested-fields">
                                    {#each Object.entries(value) as [subKey, subVal]}
                                        <div class="field">
                                            <label>{formatKey(subKey)}</label>
                                            <input type="text" value={subVal} on:input={(e) => updateNestedMetadata(key, subKey, e.target.value)} />
                                        </div>
                                    {/each}
                                </div>
                            </div>
                        {:else if typeof value === 'boolean'}
                            <div class="field boolean-field">
                                <label class="checkbox-label">
                                    <input type="checkbox" checked={value} on:change={(e) => updateMetadata(key, e.target.checked)} />
                                    {formatKey(key)}
                                </label>
                            </div>
                        {:else}
                            <div class="field">
                                <label>{formatKey(key)}</label>
                                <input type="text" value={value} on:input={(e) => updateMetadata(key, e.target.value)} />
                            </div>
                        {/if}
                    {/each}
                </div>

                <!-- Add custom field -->
                <div class="add-custom-field">
                    <input type="text" bind:value={newFieldKey} placeholder="New Field Key (e.g. batteryCycles)" />
                    <input type="text" bind:value={newFieldValue} placeholder="Value" on:keydown={(e) => e.key === 'Enter' && (e.preventDefault(), addCustomField())} />
                    <button type="button" class="btn secondary" on:click={addCustomField}>+ Add</button>
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
    .section-header { display: flex; justify-content: space-between; align-items: center; border-bottom: 1px solid #333; padding-bottom: 0.5rem; margin-bottom: 0.5rem; }
    .section-header h2 { border-bottom: none; margin-bottom: 0; padding-bottom: 0; }
    .section-hint { color: #888; font-size: 0.85rem; margin: 0 0 1rem 0; font-style: italic; }

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
    input[type="checkbox"] { width: auto; margin-right: 0.5rem; transform: scale(1.2); }
    .checkbox-label { display: flex; align-items: center; color: #ccc; font-size: 0.95rem; cursor: pointer; }
    .code-input { font-family: monospace; }

    .btn { padding: 0.8rem 1.5rem; border-radius: 4px; border: none; font-weight: 600; cursor: pointer; font-size: 0.95rem; }
    .btn.secondary { background: #333; color: #ccc; }
    .btn.secondary:hover { background: #444; }

    /* Tags */
    .tags-container { display: flex; flex-wrap: wrap; gap: 0.5rem; margin-bottom: 0.5rem; }
    .part-tag { background: rgba(74, 105, 189, 0.2); color: #93c5fd; border: 1px solid #4a69bd; padding: 0.4rem 0.8rem; border-radius: 6px; font-size: 0.9rem; display: flex; align-items: center; gap: 0.5rem; }
    .remove-tag { background: none; border: none; color: #93c5fd; cursor: pointer; font-size: 1.1rem; padding: 0; line-height: 1; }
    .remove-tag:hover { color: #ff6b6b; }
    .add-tag-row { display: flex; gap: 0.5rem; }
    .add-tag-row input { flex: 1; }

    /* Dynamic Grid */
    .dynamic-section { background: rgba(30, 30, 30, 0.5); }
    .dynamic-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(280px, 1fr)); gap: 1rem; margin-bottom: 1.5rem; }
    .metadata-badge { background: #3a2a0a; color: #fbbf24; border: 1px solid #f59e0b; padding: 0.2rem 0.5rem; border-radius: 4px; font-size: 0.7rem; text-transform: uppercase; }
    .nested-group { grid-column: 1 / -1; background: #1a1a1a; padding: 1rem; border-radius: 6px; border: 1px solid #2a2a2a; }
    .nested-group h4 { margin: 0 0 1rem 0; color: #a3bffa; font-size: 0.9rem; text-transform: uppercase; }
    .nested-fields { display: grid; grid-template-columns: repeat(auto-fill, minmax(200px, 1fr)); gap: 1rem; }
    .boolean-field { justify-content: center; background: #1a1a1a; padding: 0.8rem; border-radius: 4px; border: 1px solid #2a2a2a; }
    .add-custom-field { display: flex; gap: 0.5rem; align-items: stretch; border-top: 1px dashed #444; padding-top: 1rem; }

    .actions { margin-top: 1rem; display: flex; justify-content: flex-end; gap: 1rem; }
    .save-btn { background: #28a745; color: white; border: none; padding: 0.8rem 2rem; border-radius: 4px; font-weight: 600; cursor: pointer; }
    .save-btn:hover { background: #218838; }
    .cancel-btn { background: #333; color: #ccc; border: none; padding: 0.8rem 1.5rem; border-radius: 4px; cursor: pointer; }
    .cancel-btn:hover { background: #444; }
    .delete-btn { background: #d32f2f; color: white; border: none; padding: 0.6rem 1.2rem; border-radius: 4px; cursor: pointer; }

    .linked-banner { background: #1a2a3a; border-color: #3b82f6; padding: 0.9rem 1.25rem; }
    .linked-row { display: flex; align-items: center; gap: 1rem; flex-wrap: wrap; }
    .linked-label { color: #93c5fd; font-weight: 600; font-size: 0.9rem; }
    .linked-link { color: #bfdbfe; text-decoration: none; font-family: monospace; font-size: 0.85rem; border-bottom: 1px dashed #4a69bd; }
    .linked-link:hover { color: #fff; border-bottom-color: #fff; }

    @media (max-width: 700px) { .form-grid { grid-template-columns: 1fr; } .add-custom-field { flex-direction: column; } }
</style>
