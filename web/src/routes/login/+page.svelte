<script>
    import { onMount } from 'svelte';
    import { authStore } from '$lib/stores/authStore';
    import { goto } from '$app/navigation';
    import { base } from '$app/paths';

    let email = '';
    let password = '';
    let error = '';
    let isLoading = false;

    let needsSetup = false;
    let setupEmail = '';
    let setupPassword = '';

    onMount(async () => {
        try {
            const pathBase = base || '/E';
            const res = await fetch(`${pathBase}/auth/setup-status`);
            if (res.ok) {
                const data = await res.json();
                if (data.needsSetup) {
                    needsSetup = true;
                    setupEmail = data.email;
                    setupPassword = data.password;
                    // Pre-fill the login form with setup credentials
                    email = data.email;
                    password = data.password;
                }
            }
        } catch (e) {
            // ignore — server may not expose setup-status
        }
    });

    async function handleLogin() {
        if (!email || !password) {
            error = 'Please fill in all fields';
            return;
        }

        isLoading = true;
        error = '';

        const result = await authStore.login(email, password);

        if (result.success) {
            const pathBase = base || '/E';
            goto(`${pathBase}/dashboard`);
        } else {
            error = result.error || 'Login failed';
        }
        isLoading = false;
    }
</script>

<div class="login-container">
    <div class="login-card">
        <div class="logo">
            <h1>eckWMS</h1>
            {#if needsSetup}
                <span class="version setup-mode">First Run</span>
            {:else}
                <span class="version">Rust Edition</span>
            {/if}
        </div>

        {#if needsSetup}
            <div class="setup-banner">
                <div class="setup-title">Initial Setup</div>
                <p>No users exist yet. Use the temporary credentials below to log in, then create your admin account.</p>
                <div class="setup-creds">
                    <div class="cred-row">
                        <span class="cred-label">Email</span>
                        <span class="cred-value">{setupEmail}</span>
                    </div>
                    <div class="cred-row">
                        <span class="cred-label">Password</span>
                        <span class="cred-value mono">{setupPassword}</span>
                    </div>
                </div>
                <p class="setup-hint">This banner disappears once you create a real admin account.</p>
            </div>
        {/if}

        <form on:submit|preventDefault={handleLogin}>
            <div class="form-group">
                <label for="email">Email</label>
                <input
                    type="text"
                    id="email"
                    bind:value={email}
                    placeholder="operator@eckwms.local"
                    disabled={isLoading}
                />
            </div>

            <div class="form-group">
                <label for="password">Password</label>
                <input
                    type="password"
                    id="password"
                    bind:value={password}
                    placeholder="••••••••"
                    disabled={isLoading}
                />
            </div>

            {#if error}
                <div class="error-msg">{error}</div>
            {/if}

            <button type="submit" disabled={isLoading}>
                {isLoading ? 'Authenticating...' : 'Login'}
            </button>
        </form>
    </div>
</div>

<style>
    .login-container {
        height: 100vh;
        display: flex;
        align-items: center;
        justify-content: center;
        background-color: #1e1e1e;
        background-image: radial-gradient(#2a2a2a 1px, transparent 1px);
        background-size: 20px 20px;
    }

    .login-card {
        background: #2c2c2e;
        padding: 2.5rem;
        border-radius: 8px;
        width: 100%;
        max-width: 420px;
        box-shadow: 0 10px 25px rgba(0,0,0,0.5);
        border: 1px solid #444;
    }

    .logo {
        text-align: center;
        margin-bottom: 2rem;
    }

    .logo h1 {
        margin: 0;
        color: #4a69bd;
        font-size: 2rem;
        font-weight: 700;
        letter-spacing: -1px;
    }

    .version {
        font-size: 0.8rem;
        color: #666;
        text-transform: uppercase;
        letter-spacing: 2px;
    }

    .setup-mode {
        color: #e8a838;
    }

    .setup-banner {
        background: rgba(232, 168, 56, 0.08);
        border: 1px solid rgba(232, 168, 56, 0.3);
        border-radius: 6px;
        padding: 1rem 1.2rem;
        margin-bottom: 1.5rem;
        font-size: 0.85rem;
        color: #ccc;
    }

    .setup-title {
        font-weight: 700;
        color: #e8a838;
        text-transform: uppercase;
        letter-spacing: 1px;
        font-size: 0.75rem;
        margin-bottom: 0.4rem;
    }

    .setup-banner p {
        margin: 0 0 0.75rem 0;
        line-height: 1.5;
    }

    .setup-creds {
        background: rgba(0,0,0,0.3);
        border-radius: 4px;
        padding: 0.6rem 0.8rem;
        margin-bottom: 0.75rem;
    }

    .cred-row {
        display: flex;
        gap: 0.75rem;
        align-items: baseline;
        padding: 0.2rem 0;
    }

    .cred-label {
        color: #888;
        min-width: 60px;
        font-size: 0.8rem;
    }

    .cred-value {
        color: #fff;
        font-size: 0.9rem;
    }

    .cred-value.mono {
        font-family: monospace;
        font-size: 1rem;
        color: #e8a838;
        letter-spacing: 1px;
    }

    .setup-hint {
        color: #666;
        font-size: 0.78rem;
        margin: 0;
    }

    .form-group {
        margin-bottom: 1.5rem;
    }

    label {
        display: block;
        margin-bottom: 0.5rem;
        color: #aaa;
        font-size: 0.9rem;
    }

    input {
        width: 100%;
        padding: 0.75rem;
        background: #1a1a1a;
        border: 1px solid #444;
        border-radius: 4px;
        color: #fff;
        font-size: 1rem;
        transition: border-color 0.2s;
        box-sizing: border-box;
    }

    input:focus {
        outline: none;
        border-color: #4a69bd;
    }

    button {
        width: 100%;
        padding: 0.8rem;
        background: #4a69bd;
        color: white;
        border: none;
        border-radius: 4px;
        font-size: 1rem;
        font-weight: 600;
        cursor: pointer;
        transition: background 0.2s;
    }

    button:hover:not(:disabled) {
        background: #3d5aa8;
    }

    button:disabled {
        opacity: 0.7;
        cursor: not-allowed;
    }

    .error-msg {
        color: #ff6b6b;
        background: rgba(255, 107, 107, 0.1);
        padding: 0.75rem;
        border-radius: 4px;
        margin-bottom: 1.5rem;
        font-size: 0.9rem;
        text-align: center;
    }
</style>
