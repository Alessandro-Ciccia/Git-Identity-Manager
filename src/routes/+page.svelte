<script lang="ts">
  import { onMount } from 'svelte';

  import EnvironmentStatusPanel from '$lib/components/EnvironmentStatusPanel.svelte';
  import GithubAccountsPanel from '$lib/components/GithubAccountsPanel.svelte';
  import type { EnvironmentStatus } from '$lib/domain/environment';
  import {
    githubAccountKey,
    hasNewGithubAccount,
    type GithubAccount,
    type GithubAccountsStatus,
  } from '$lib/domain/github';
  import { navigationItems } from '$lib/domain/navigation';
  import { environmentErrorMessage, getEnvironmentStatus } from '$lib/ipc/environment';
  import {
    githubCliUpdateRecommended,
    githubErrorMessage,
    launchGithubLogin,
    listGithubAccounts,
    openGithubAccountPage,
    openGithubCliUpdatePage,
    openGithubLoginPage,
    switchGithubAccount,
  } from '$lib/ipc/github';

  let status = $state<EnvironmentStatus | null>(null);
  let isLoading = $state(true);
  let loadError = $state<string | null>(null);
  let githubStatus = $state<GithubAccountsStatus | null>(null);
  let githubLoading = $state(true);
  let githubError = $state<string | null>(null);
  let showGithubUpdateAction = $state(false);
  let switchingAccountKey = $state<string | null>(null);
  let viewingAccountKey = $state<string | null>(null);
  let loginLoading = $state(false);
  let loginInProgress = $state(false);
  let loginBaseline = $state<GithubAccountsStatus | null>(null);
  let loginPollGeneration = 0;
  let updateLoading = $state(false);

  const LOGIN_POLL_INTERVAL_MS = 3_000;
  const LOGIN_POLL_ATTEMPTS = 40;

  function wait(milliseconds: number): Promise<void> {
    return new Promise((resolve) => globalThis.setTimeout(resolve, milliseconds));
  }

  function finishLoginTracking(): void {
    loginPollGeneration += 1;
    loginInProgress = false;
    loginBaseline = null;
  }

  async function refreshEnvironment(): Promise<void> {
    isLoading = true;
    loadError = null;

    try {
      status = await getEnvironmentStatus();
    } catch (error) {
      loadError = environmentErrorMessage(error);
    } finally {
      isLoading = false;
    }
  }

  async function refreshGithubAccounts(): Promise<void> {
    githubLoading = true;
    githubError = null;
    showGithubUpdateAction = false;

    try {
      const refreshedStatus = await listGithubAccounts();
      githubStatus = refreshedStatus;
      if (
        loginInProgress &&
        loginBaseline &&
        hasNewGithubAccount(loginBaseline, refreshedStatus)
      ) {
        finishLoginTracking();
      }
    } catch (error) {
      githubError = githubErrorMessage(error);
      showGithubUpdateAction = githubCliUpdateRecommended(error);
    } finally {
      githubLoading = false;
    }
  }

  async function activateGithubAccount(account: GithubAccount): Promise<void> {
    switchingAccountKey = githubAccountKey(account);
    githubError = null;

    try {
      githubStatus = await switchGithubAccount(account);
    } catch (error) {
      githubError = githubErrorMessage(error);
    } finally {
      switchingAccountKey = null;
    }
  }

  async function viewGithubAccount(account: GithubAccount): Promise<void> {
    viewingAccountKey = githubAccountKey(account);
    githubError = null;

    try {
      await openGithubAccountPage(account);
    } catch (error) {
      githubError = githubErrorMessage(error);
    } finally {
      viewingAccountKey = null;
    }
  }

  async function pollGithubLogin(generation: number): Promise<void> {
    for (let attempt = 0; attempt < LOGIN_POLL_ATTEMPTS; attempt += 1) {
      await wait(LOGIN_POLL_INTERVAL_MS);
      if (generation !== loginPollGeneration || !loginBaseline) {
        return;
      }

      try {
        const refreshedStatus = await listGithubAccounts();
        githubStatus = refreshedStatus;
        if (hasNewGithubAccount(loginBaseline, refreshedStatus)) {
          githubError = null;
          finishLoginTracking();
          return;
        }
      } catch {
        // A transient status failure while the browser flow is pending is retried.
      }
    }

    if (generation === loginPollGeneration) {
      finishLoginTracking();
      githubError =
        'Browser login is still pending. Complete the flow, then refresh GitHub accounts.';
    }
  }

  async function addGithubAccount(): Promise<void> {
    loginLoading = true;
    githubError = null;
    showGithubUpdateAction = false;

    try {
      const baseline = githubStatus;
      githubStatus = await launchGithubLogin('github.com');
      loginBaseline = baseline ?? githubStatus;
      loginInProgress = true;
      loginPollGeneration += 1;
      void pollGithubLogin(loginPollGeneration);

      try {
        await openGithubLoginPage();
      } catch (error) {
        githubError = githubErrorMessage(error);
      }
    } catch (error) {
      githubError = githubErrorMessage(error);
      showGithubUpdateAction = githubCliUpdateRecommended(error);
    } finally {
      loginLoading = false;
    }
  }

  async function openGithubLoginBrowser(): Promise<void> {
    loginLoading = true;

    try {
      await openGithubLoginPage();
    } catch (error) {
      githubError = githubErrorMessage(error);
    } finally {
      loginLoading = false;
    }
  }

  async function openGithubCliUpdate(): Promise<void> {
    updateLoading = true;

    try {
      await openGithubCliUpdatePage();
    } catch (error) {
      githubError = githubErrorMessage(error);
      showGithubUpdateAction = false;
    } finally {
      updateLoading = false;
    }
  }

  onMount(() => {
    void refreshEnvironment();
    void refreshGithubAccounts();
  });
</script>

<svelte:head>
  <title>Overview · Git Identity Manager</title>
</svelte:head>

<main class="min-h-screen bg-stone-950 text-stone-100">
  <div class="grid min-h-screen grid-cols-[220px_1fr]">
    <aside class="border-r border-white/10 bg-stone-950/80 p-4">
      <div class="mb-8">
        <p class="text-xs font-semibold uppercase tracking-[0.24em] text-sky-300">Local-first</p>
        <h1 class="mt-2 text-lg font-semibold">Git Identity Manager</h1>
      </div>

      <nav aria-label="Primary navigation" class="space-y-1">
        {#each navigationItems as item (item.id)}
          <a
            class={`block rounded-lg px-3 py-2 text-sm transition hover:bg-white/5 hover:text-white ${
              item.id === 'overview' ? 'bg-white/10 text-white' : 'text-stone-300'
            }`}
            href={`#${item.id}`}
            aria-current={item.id === 'overview' ? 'page' : undefined}
          >
            {item.label}
          </a>
        {/each}
      </nav>
    </aside>

    <section class="px-10 py-12">
      <div class="max-w-4xl">
        <p class="text-sm font-medium text-sky-300">Overview</p>
        <h2 class="mt-3 text-4xl font-semibold tracking-tight text-white">Identity overview</h2>
        <p class="mt-4 max-w-2xl text-base leading-7 text-stone-300">
          See the GitHub accounts already managed by GitHub CLI and switch the active account
          without exposing credentials to the interface.
        </p>

        <div class="mt-10 space-y-6">
          <GithubAccountsPanel
            status={githubStatus}
            loading={githubLoading}
            error={githubError}
            {switchingAccountKey}
            {viewingAccountKey}
            {loginLoading}
            {loginInProgress}
            showUpdateAction={showGithubUpdateAction}
            {updateLoading}
            onRefresh={refreshGithubAccounts}
            onSwitch={activateGithubAccount}
            onView={viewGithubAccount}
            onLogin={addGithubAccount}
            onOpenLoginPage={openGithubLoginBrowser}
            onUpdate={openGithubCliUpdate}
          />

          <EnvironmentStatusPanel
            {status}
            loading={isLoading}
            error={loadError}
            onRefresh={refreshEnvironment}
          />
        </div>
      </div>
    </section>
  </div>
</main>
