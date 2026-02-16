<script>
    import { onMount } from 'svelte';
    import { api } from '$lib/api';
    import { toastStore } from '$lib/stores/toastStore';
    import { base } from '$app/paths';

    let devices = [];
    let meshNodes = []; // Available servers
    let loading = true;
    let qrUrl = '';
    let showQr = false;
    let qrType = 'standard';

    async function loadData() {
        loading = true;
        try {
            const [devicesData, nodesData, statusData] = await Promise.all([
                api.get('/api/admin/devices?include_deleted=true'),
                api.get('/mesh/nodes'),
                api.get('/mesh/status')
            ]);
            devices = devicesData || [];

            // Process nodes for dropdown
            meshNodes = nodesData || [];

            // Add current node (self) since /mesh/nodes excludes self
            if (statusData && statusData.instance_id) {
                const selfNode = {
                    instance_id: statusData.instance_id,
                    role: statusData.role || 'peer',
                    base_url: statusData.base_url || 'http://localhost:3210',
                    is_self: true
                };
                // Add at the beginning
                meshNodes = [selfNode, ...meshNodes];
            }
        } catch (e) {
            toastStore.add('Failed to load data: ' + e.message, 'error');
        } finally {
            loading = false;
        }
    }

    async function updateStatus(deviceId, status) {
        try {
            await api.put(`/api/admin/devices/${deviceId}/status`, { status });
            toastStore.add(`Device ${status}`, 'success');
            // Update local state instead of full reload
            devices = devices.map(d => d.deviceId === deviceId ? { ...d, status } : d);
        } catch (e) {
            toastStore.add(e.message, 'error');
        }
    }

    async function updateHomeNode(deviceId, homeInstanceId) {
        try {
            await api.put(`/api/admin/devices/${deviceId}/home`, { homeInstanceId });
            toastStore.add('Home Node updated', 'success');
        } catch (e) {
            toastStore.add(e.message, 'error');
        }
    }

    async function deleteDevice(deviceId) {
        if (!confirm('Are you sure you want to delete this device? This will sync to all nodes.')) {
            return;
        }
        try {
            await api.delete(`/api/admin/devices/${deviceId}`);
            toastStore.add('Device deleted', 'success');
            await loadData();
        } catch (e) {
            toastStore.add(e.message, 'error');
        }
    }

    async function restoreDevice(deviceId) {
        try {
            await api.post(`/api/admin/devices/${deviceId}/restore`);
            toastStore.add('Device restored', 'success');
            await loadData();
        } catch (e) {
            toastStore.add(e.message, 'error');
        }
    }

    async function loadQr(type = 'standard') {
        if (showQr && qrType === type) {
            showQr = false;
            return;
        }
        qrType = type;
        try {
            const token = localStorage.getItem('auth_token');
            const url = type === 'vip'
                ? `${base}/api/internal/pairing-qr?type=vip`
                : `${base}/api/internal/pairing-qr`;

            const res = await fetch(url, {
                headers: { Authorization: `Bearer ${token}` }
            });
            const blob = await res.blob();
            qrUrl = URL.createObjectURL(blob);
            showQr = true;
        } catch (e) {
            toastStore.add('Failed to load Pairing QR', 'error');
        }
    }

    function getNodeName(instanceId, includeUrl = false) {
        if (!instanceId) return 'Unknown';

        const node = meshNodes.find(n => n.instance_id === instanceId);

        // Extract role from node data or instance_id pattern
        let role = 'UNKNOWN';
        if (node) {
            role = node.role.toUpperCase();
        } else {
            // Try to determine role from instance_id pattern
            if (instanceId.includes('production')) role = 'MASTER';
            else if (instanceId.includes('instance_')) role = 'PEER';
            else if (instanceId.includes('local')) role = 'PEER';
        }

        let identifier = '';

        // If node has a non-localhost URL, use domain as identifier
        if (node && node.base_url && !node.base_url.includes('localhost')) {
            try {
                const url = new URL(node.base_url);
                identifier = url.hostname; // e.g., "pda.repair"
            } catch (e) {
                // URL parsing failed, fall through to hash extraction
            }
        }

        // If no domain available, extract meaningful part from instance_id
        if (!identifier) {
            let hash = instanceId;

            // Remove common prefixes
            hash = hash.replace(/^production_/, '');
            hash = hash.replace(/^local_/, '');
            hash = hash.replace(/^instance_/, '');

            // For UUIDs, take first two segments
            if (hash.includes('-')) {
                const uuidParts = hash.split('-');
                if (uuidParts.length >= 5) {
                    hash = uuidParts.slice(0, 2).join('-');
                }
            }

            // Limit length if still too long
            if (hash.length > 20) {
                hash = hash.substring(0, 20);
            }

            identifier = hash;
        }

        return `${role}-${identifier}`;
    }

    onMount(() => {
        loadData();
    });
</script>

<div class="page">
    <header>
        <h1>Device Management</h1>
        <div class="action-group">
            <button class="btn secondary" class:active={showQr && qrType === 'standard'} on:click={() => loadQr('standard')}>
                Show Standard QR
            </button>
            <button class="btn primary" class:active={showQr && qrType === 'vip'} on:click={() => loadQr('vip')}>
                Show Auto-Approve QR
            </button>
            <button class="btn secondary" on:click={loadData}>↻ Refresh</button>
        </div>
    </header>

    {#if showQr && qrUrl}
        <div class="qr-panel" class:vip={qrType === 'vip'}>
            <h3>{qrType === 'vip' ? '⚡ Auto-Approve Pairing' : '🔒 Standard Pairing'}</h3>
            <img src={qrUrl} alt="Pairing QR" />
            <p class="hint">
                {#if qrType === 'vip'}
                    <strong>Warning:</strong> Devices scanning this code will be <u>immediately authorized</u>. Valid for 24 hours.
                {:else}
                    Devices scanning this code will appear as <strong>Pending</strong> below.
                {/if}
            </p>
            <button class="btn-text" on:click={() => showQr = false}>Close</button>
        </div>
    {/if}

    <div class="device-list">
        {#if loading}
            <div class="loading">Loading devices...</div>
        {:else if devices.length === 0}
            <div class="empty">No devices registered. Scan a QR code to add one.</div>
        {:else}
            <table>
                <thead>
                    <tr>
                        <th>Status</th>
                        <th>Device Name</th>
                        <th>ID / Key</th>
                        <th>Home Node (Proxy Target)</th>
                        <th>Last Seen</th>
                        <th>Actions</th>
                    </tr>
                </thead>
                <tbody>
                    {#each devices as device}
                        <tr class:deleted={device.deletedAt}>
                            <td>
                                {#if device.deletedAt}
                                    <span class="badge deleted">Deleted</span>
                                {:else}
                                    <span class="badge {device.status}">{device.status}</span>
                                {/if}
                            </td>
                            <td>
                                <div class="device-name">{device.name || 'Unknown'}</div>
                            </td>
                            <td>
                                <div class="mono-id" title={device.deviceId}>{device.deviceId.substring(0, 8)}...</div>
                                <div class="mono-key" title="Public Key">{device.publicKey ? device.publicKey.substring(0, 8) + '...' : '-'}</div>
                            </td>
                            <td>
                                <div class="home-node-control">
                                    <select
                                        value={device.homeInstanceId}
                                        on:change={(e) => updateHomeNode(device.deviceId, e.target.value)}
                                        disabled={!!device.deletedAt}
                                    >
                                        <option value={device.homeInstanceId}>{getNodeName(device.homeInstanceId)} (Current)</option>
                                        {#each meshNodes as node}
                                            {#if node.instance_id !== device.homeInstanceId}
                                                <option value={node.instance_id}>
                                                    {getNodeName(node.instance_id, true)}
                                                </option>
                                            {/if}
                                        {/each}
                                    </select>
                                </div>
                            </td>
                            <td>{new Date(device.lastSeenAt).toLocaleString()}</td>
                            <td class="actions">
                                {#if device.deletedAt}
                                    <button class="btn-icon restore" title="Restore" on:click={() => restoreDevice(device.deviceId)}>♻️</button>
                                {:else}
                                    {#if device.status === 'pending' || device.status === 'blocked'}
                                        <button class="btn-icon approve" title="Approve" on:click={() => updateStatus(device.deviceId, 'active')}>✅</button>
                                    {/if}
                                    {#if device.status === 'active' || device.status === 'pending'}
                                        <button class="btn-icon block" title="Block" on:click={() => updateStatus(device.deviceId, 'blocked')}>⛔</button>
                                    {/if}
                                    <button class="btn-icon delete" title="Delete" on:click={() => deleteDevice(device.deviceId)}>🗑️</button>
                                {/if}
                            </td>
                        </tr>
                    {/each}
                </tbody>
            </table>
        {/if}
    </div>
</div>

<style>
    header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 2rem; flex-wrap: wrap; gap: 1rem; }
    h1 { color: #fff; margin: 0; }
    .action-group { display: flex; gap: 10px; }

    .qr-panel { background: #fff; padding: 2rem; border-radius: 12px; text-align: center; margin-bottom: 2rem; color: #000; max-width: 400px; margin-left: auto; margin-right: auto; border: 4px solid transparent; }
    .qr-panel.vip { border-color: #f39c12; background: #fff9e6; }
    .qr-panel img { max-width: 100%; height: auto; display: block; margin: 0 auto; border: 1px solid #eee; }
    .hint { margin-top: 1rem; font-size: 0.9rem; color: #555; }

    table { width: 100%; border-collapse: collapse; background: #1e1e1e; border-radius: 8px; overflow: hidden; }
    th, td { padding: 1rem; text-align: left; border-bottom: 1px solid #333; color: #eee; vertical-align: middle; }
    th { background: #252525; font-weight: 600; color: #888; text-transform: uppercase; font-size: 0.8rem; }

    tr.deleted { opacity: 0.5; background: #2a1a1a; }

    .mono-id { font-family: monospace; color: #fff; font-weight: bold; }
    .mono-key { font-family: monospace; color: #666; font-size: 0.8em; }

    .badge { padding: 4px 8px; border-radius: 4px; font-size: 0.75rem; font-weight: bold; text-transform: uppercase; }
    .badge.active { background: rgba(40, 167, 69, 0.2); color: #28a745; }
    .badge.pending { background: rgba(255, 193, 7, 0.2); color: #ffc107; }
    .badge.blocked { background: rgba(220, 53, 69, 0.2); color: #dc3545; }
    .badge.deleted { background: #333; color: #aaa; text-decoration: line-through; }

    /* Controls */
    .home-node-control select {
        background: #111; border: 1px solid #444; color: #ddd; padding: 6px; border-radius: 4px; max-width: 200px;
    }

    .btn { padding: 0.6rem 1.2rem; border-radius: 6px; border: 1px solid transparent; font-weight: 600; cursor: pointer; transition: all 0.2s; }
    .btn.active { transform: translateY(2px); box-shadow: inset 0 2px 4px rgba(0,0,0,0.2); }
    .btn.primary { background: #f39c12; color: #000; }
    .btn.secondary { background: #2a2a2a; color: #fff; border-color: #444; }
    .btn-text { background: none; border: none; color: #666; text-decoration: underline; margin-top: 10px; cursor: pointer; }

    .btn-icon { background: none; border: none; font-size: 1.2rem; cursor: pointer; padding: 4px; transition: transform 0.2s; }
    .btn-icon:hover { transform: scale(1.2); }
    .btn-icon.restore { background: #2a2a2a; border-radius: 50%; width: 30px; height: 30px; display: flex; align-items: center; justify-content: center; }

    .empty, .loading { text-align: center; padding: 3rem; color: #666; font-style: italic; }
</style>
