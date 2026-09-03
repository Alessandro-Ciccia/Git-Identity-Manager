<script lang="ts">
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

  function healthBadgeClasses(state: GithubAuthenticationState): string {
    switch (state) {
      case 'success':
        return 'border-emerald-400/20 bg-emerald-400/10 text-emerald-300';
      case 'error':
        return 'border-rose-400/20 bg-rose-400/10 text-rose-200';
      case 'timeout':
        return 'border-amber-400/20 bg-amber-400/10 text-amber-200';
    }
  }
</script>

<section
  class="overflow-hidden rounded-2xl border border-white/10 bg-stone-900/50"
  aria-labelledby="github-accounts-heading"
  aria-busy={operationPending}
>
  <header
    class="flex flex-wrap items-start justify-between gap-5 border-b border-white/10 px-6 py-5"
  >
    <div>
      <p class="text-xs font-semibold uppercase tracking-[0.2em] text-stone-500">GitHub CLI</p>
      <h3 id="github-accounts-heading" class="mt-2 text-lg font-semibold text-white">
        GitHub accounts
      </h3>
      <p class="mt-1 max-w-2xl text-sm text-stone-400">
        Credentials stay managed by GitHub CLI and your operating system.
      </p>
    </div>

    <div class="flex items-center gap-2">
      <button
        type="button"
        class="rounded-lg border border-white/10 bg-white/5 px-3 py-2 text-sm font-medium text-stone-200 transition hover:border-white/20 hover:bg-white/10 disabled:cursor-wait disabled:opacity-60"
        onclick={onRefresh}
        disabled={operationPending}
      >
        {loading ? 'Refreshing…' : 'Refresh'}
      </button>
      <button
        type="button"
        class="rounded-lg bg-sky-300 px-3 py-2 text-sm font-semibold text-stone-950 transition hover:bg-sky-200 disabled:cursor-wait disabled:opacity-60"
        onclick={onLogin}
        disabled={operationPending || status === null || loginInProgress}
      >
        {loginLoading ? 'Opening browser…' : loginInProgress ? 'Login in progress' : 'Add account'}
      </button>
    </div>
  </header>

  <div aria-live="polite">
    {#if loginInProgress}
      <div
        class="flex flex-wrap items-center justify-between gap-4 border-b border-sky-300/20 bg-sky-300/5 px-6 py-4 text-sm text-sky-100"
      >
        <span>
          Browser login started. The one-time code is in your clipboard. Complete the flow in your
          browser; accounts refresh automatically, or you can refresh manually.
        </span>
        <button
          type="button"
          class="shrink-0 rounded-lg border border-sky-200/20 bg-sky-200/10 px-3 py-2 font-semibold text-sky-100 transition hover:bg-sky-200/15 disabled:cursor-wait disabled:opacity-60"
          onclick={onOpenLoginPage}
          disabled={operationPending}
        >
          {loginLoading ? 'Opening…' : 'Open browser'}
        </button>
      </div>
    {/if}

    {#if error}
      <div
        class="flex flex-wrap items-center justify-between gap-4 border-b border-rose-400/20 bg-rose-400/5 px-6 py-4 text-sm text-rose-200"
        role="alert"
      >
        <span>{error}</span>
        {#if showUpdateAction}
          <button
            type="button"
            class="shrink-0 rounded-lg border border-rose-300/20 bg-rose-200/10 px-3 py-2 font-semibold text-rose-100 transition hover:bg-rose-200/15 disabled:cursor-wait disabled:opacity-60"
            onclick={onUpdate}
            disabled={operationPending}
          >
            {updateLoading ? 'Opening…' : 'Update GitHub CLI'}
          </button>
        {/if}
      </div>
    {/if}

    {#if status && groups.length > 0}
      <div class="divide-y divide-white/10">
        {#each groups as group (group.hostname)}
          <section aria-labelledby={`host-${group.hostname}`}>
            <div class="bg-black/10 px-6 py-3">
              <h4
                id={`host-${group.hostname}`}
                class="font-mono text-xs font-medium text-stone-400"
              >
                {group.hostname}
              </h4>
            </div>

            <ul class="divide-y divide-white/10">
              {#each group.accounts as account (githubAccountKey(account))}
                <li class="grid gap-4 px-6 py-5 sm:grid-cols-[1fr_auto] sm:items-center">
                  <div>
                    <div class="flex flex-wrap items-center gap-2.5">
                      <p class="font-medium text-white">@{account.username}</p>
                      {#if account.active}
                        <span
                          class="rounded-full border border-sky-300/20 bg-sky-300/10 px-2 py-0.5 text-xs font-semibold text-sky-200"
                        >
                          Active
                        </span>
                      {/if}
                      <span
                        class={`rounded-full border px-2 py-0.5 text-xs font-semibold ${healthBadgeClasses(account.state)}`}
                      >
                        {githubAuthenticationLabels[account.state]}
                      </span>
                    </div>
                    <p class="mt-2 font-mono text-xs text-stone-400">
                      {account.email ?? 'No public email'}
                    </p>
                    <button
                      type="button"
                      class="mt-1 text-xs font-medium text-stone-400 underline decoration-stone-600 underline-offset-4 transition hover:text-sky-200 hover:decoration-sky-300 disabled:cursor-wait disabled:opacity-60"
                      onclick={() => onView(account)}
                      disabled={operationPending}
                      aria-label={`View @${account.username} on ${account.hostname} in browser`}
                    >
                      {viewingAccountKey === githubAccountKey(account)
                        ? 'Opening…'
                        : 'View on browser'}
                    </button>
                    {#if account.state !== 'success'}
                      <p class="mt-2 text-sm text-stone-400">
                        Refresh or re-authenticate this account with GitHub CLI before switching.
                      </p>
                    {/if}
                  </div>

                  <div class="flex items-center justify-end gap-2">
                    {#if account.state !== 'success'}
                      <span class="mr-1 text-sm text-stone-500">Unavailable</span>
                    {/if}

                    {#if !account.active && account.state === 'success'}
                      <button
                        type="button"
                        class="rounded-lg border border-white/10 bg-white/5 px-3 py-2 text-sm font-medium text-stone-200 transition hover:border-sky-300/30 hover:bg-sky-300/10 hover:text-white disabled:cursor-wait disabled:opacity-60"
                        onclick={() => onSwitch(account)}
                        disabled={operationPending}
                        aria-label={`Switch to @${account.username} on ${account.hostname}`}
                      >
                        {switchingAccountKey === githubAccountKey(account)
                          ? 'Switching…'
                          : 'Switch'}
                      </button>
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
        <div class="h-16 animate-pulse rounded-xl bg-white/5"></div>
        <div class="h-16 animate-pulse rounded-xl bg-white/5"></div>
      </div>
    {:else if status && status.accounts.length === 0}
      <div class="px-6 py-8">
        <p class="font-medium text-white">No GitHub accounts found</p>
        <p class="mt-2 max-w-xl text-sm leading-6 text-stone-400">
          Add an account to start GitHub CLI’s official browser login. Git Identity Manager never
          receives or stores the resulting token.
        </p>
      </div>
    {:else if !error}
      <p class="px-6 py-8 text-sm text-stone-400">GitHub account status is not available yet.</p>
    {/if}
  </div>
</section>
