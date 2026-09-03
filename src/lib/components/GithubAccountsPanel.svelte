<script lang="ts">
  import Badge from '$lib/components/ui/Badge.svelte';
  import Banner from '$lib/components/ui/Banner.svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import EmptyState from '$lib/components/ui/EmptyState.svelte';
  import Panel from '$lib/components/ui/Panel.svelte';
  import {
    githubAccountKey,
    githubAuthenticationLabels,
    groupGithubAccounts,
    type GithubAccount,
    type GithubAccountsStatus,
    type GithubAuthenticationState,
  } from '$lib/domain/github';

  type Props = {
    status: GithubAccountsStatus | null;
    loading: boolean;
    error: string | null;
    switchingAccountKey: string | null;
    viewingAccountKey: string | null;
    loginLoading: boolean;
    loginInProgress: boolean;
    showUpdateAction: boolean;
    updateLoading: boolean;
    onRefresh: () => void | Promise<void>;
    onSwitch: (account: GithubAccount) => void | Promise<void>;
    onView: (account: GithubAccount) => void | Promise<void>;
    onLogin: () => void | Promise<void>;
    onOpenLoginPage: () => void | Promise<void>;
    onUpdate: () => void | Promise<void>;
  };

  let {
    status,
    loading,
    error,
    switchingAccountKey,
    viewingAccountKey,
    loginLoading,
    loginInProgress,
    showUpdateAction,
    updateLoading,
    onRefresh,
    onSwitch,
    onView,
    onLogin,
    onOpenLoginPage,
    onUpdate,
  }: Props = $props();

  const groups = $derived(groupGithubAccounts(status?.accounts ?? []));
  const operationPending = $derived(
    loading ||
      switchingAccountKey !== null ||
      viewingAccountKey !== null ||
      loginLoading ||
      updateLoading,
  );

  const healthTone = {
    success: 'positive',
    error: 'danger',
    timeout: 'warning',
  } as const satisfies Record<GithubAuthenticationState, 'positive' | 'danger' | 'warning'>;
</script>

<Panel
  eyebrow="GitHub CLI"
  heading="GitHub accounts"
  headingId="github-accounts-heading"
  description="Credentials stay managed by GitHub CLI and your operating system."
  busy={operationPending}
>
  {#snippet actions()}
    <Button
      onclick={onRefresh}
      disabled={operationPending}
      pending={loading}
      pendingLabel="Refreshing…"
    >
      Refresh
    </Button>
    <Button
      variant="primary"
      onclick={onLogin}
      disabled={operationPending || status === null || loginInProgress}
    >
      {loginLoading ? 'Opening browser…' : loginInProgress ? 'Login in progress' : 'Add account'}
    </Button>
  {/snippet}

  {#if loginInProgress}
    <Banner tone="info">
      {#snippet action()}
        <Button
          variant="secondary"
          onclick={onOpenLoginPage}
          disabled={operationPending}
          pending={loginLoading}
          pendingLabel="Opening…"
        >
          Open browser
        </Button>
      {/snippet}
      Browser login started. The one-time code is in your clipboard. Complete the flow in your browser;
      accounts refresh automatically, or you can refresh manually.
    </Banner>
  {/if}

  {#if error}
    <Banner tone="danger">
      {#snippet action()}
        {#if showUpdateAction}
          <Button
            variant="secondary"
            onclick={onUpdate}
            disabled={operationPending}
            pending={updateLoading}
            pendingLabel="Opening…"
          >
            Update GitHub CLI
          </Button>
        {/if}
      {/snippet}
      {error}
    </Banner>
  {/if}

  {#if status && groups.length > 0}
    <div class="divide-y divide-edge">
      {#each groups as group (group.hostname)}
        <section aria-labelledby={`host-${group.hostname}`}>
          <div class="bg-surface-raised px-6 py-3">
            <h4 id={`host-${group.hostname}`} class="font-mono text-xs font-medium text-fg-muted">
              {group.hostname}
            </h4>
          </div>

          <ul class="divide-y divide-edge">
            {#each group.accounts as account (githubAccountKey(account))}
              <li class="grid gap-4 px-6 py-5 sm:grid-cols-[1fr_auto] sm:items-center">
                <div>
                  <div class="flex flex-wrap items-center gap-2.5">
                    <p class="font-medium text-fg-strong">@{account.username}</p>
                    {#if account.active}
                      <Badge tone="accent">Active</Badge>
                    {/if}
                    <Badge tone={healthTone[account.state]}>
                      {githubAuthenticationLabels[account.state]}
                    </Badge>
                  </div>
                  <p class="mt-2 font-mono text-xs text-fg-muted">
                    {account.email ?? 'No public email'}
                  </p>
                  <button
                    type="button"
                    class="mt-1 text-xs font-medium text-fg-muted underline decoration-fg-faint underline-offset-4 transition hover:text-accent-text hover:decoration-accent disabled:cursor-wait disabled:opacity-60"
                    onclick={() => onView(account)}
                    disabled={operationPending}
                    aria-label={`View @${account.username} on ${account.hostname} in browser`}
                  >
                    {viewingAccountKey === githubAccountKey(account)
                      ? 'Opening…'
                      : 'View on browser'}
                  </button>
                  {#if account.state !== 'success'}
                    <p class="mt-2 text-sm text-fg-muted">
                      Refresh or re-authenticate this account with GitHub CLI before switching.
                    </p>
                  {/if}
                </div>

                <div class="flex items-center justify-end gap-2">
                  {#if account.state !== 'success'}
                    <span class="mr-1 text-sm text-fg-subtle">Unavailable</span>
                  {/if}

                  {#if !account.active && account.state === 'success'}
                    <Button
                      onclick={() => onSwitch(account)}
                      disabled={operationPending}
                      pending={switchingAccountKey === githubAccountKey(account)}
                      pendingLabel="Switching…"
                      aria-label={`Switch to @${account.username} on ${account.hostname}`}
                    >
                      Switch
                    </Button>
                  {/if}
                </div>
              </li>
            {/each}
          </ul>
        </section>
      {/each}
    </div>
  {:else if loading && !status}
    <div class="space-y-4 px-6 py-6" aria-label="Loading GitHub accounts">
      <div class="h-16 animate-pulse rounded-xl bg-hover"></div>
      <div class="h-16 animate-pulse rounded-xl bg-hover"></div>
    </div>
  {:else if status && status.accounts.length === 0}
    <EmptyState
      title="No GitHub accounts found"
      description="Add an account to start GitHub CLI’s official browser login. Git Identity Manager never receives or stores the resulting token."
    />
  {:else if !error}
    <p class="px-6 py-8 text-sm text-fg-muted">GitHub account status is not available yet.</p>
  {/if}
</Panel>
