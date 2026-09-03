<script lang="ts">
  import { onMount } from 'svelte';

  import EnvironmentStatusPanel from '$lib/components/EnvironmentStatusPanel.svelte';
  import GithubAccountsPanel from '$lib/components/GithubAccountsPanel.svelte';
  import ProfilesPanel from '$lib/components/ProfilesPanel.svelte';
  import type { EnvironmentStatus } from '$lib/domain/environment';
  import {
    githubAccountKey,
    hasNewGithubAccount,
    type GithubAccount,
    type GithubAccountsStatus,
  } from '$lib/domain/github';
  import { navigationItems, type NavigationSection } from '$lib/domain/navigation';
  import type { GitProfile, ProfileInput } from '$lib/domain/profiles';
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
  import {
    createProfile,
    deleteProfile,
    listProfiles,
    profileErrorMessage,
    updateProfile,
  } from '$lib/ipc/profiles';

  type ImplementedSection = Extract<NavigationSection, 'overview' | 'profiles'>;

  let activeSection = $state<ImplementedSection>('overview');
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
  let profiles = $state<GitProfile[]>([]);
  let profilesLoading = $state(true);
  let profilesError = $state<string | null>(null);
  let savingProfileId = $state<string | null>(null);
  let deletingProfileId = $state<string | null>(null);

  const LOGIN_POLL_INTERVAL_MS = 3_000;
  const LOGIN_POLL_ATTEMPTS = 40;

  function wait(milliseconds: number): Promise<void> {
    return new Promise((resolve) => globalThis.setTimeout(resolve, milliseconds));
  }

  function isImplementedSection(section: NavigationSection): section is ImplementedSection {
    return section === 'overview' || section === 'profiles';
  }

  function selectSection(section: NavigationSection): void {
    if (isImplementedSection(section)) {
      activeSection = section;
    }
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
      if (loginInProgress && loginBaseline && hasNewGithubAccount(loginBaseline, refreshedStatus)) {
        finishLoginTracking();
      }
    } catch (error) {
      githubError = githubErrorMessage(error);
      showGithubUpdateAction = githubCliUpdateRecommended(error);
    } finally {
      githubLoading = false;
    }
  }

  async function refreshProfiles(): Promise<void> {
    profilesLoading = true;
    profilesError = null;

    try {
      profiles = await listProfiles();
    } catch (error) {
      profilesError = profileErrorMessage(error);
    } finally {
      profilesLoading = false;
    }
  }

  async function addProfile(input: ProfileInput): Promise<boolean> {
    savingProfileId = 'new';
    profilesError = null;

    try {
      const created = await createProfile(input);
      profiles = [...profiles, created];
      return true;
    } catch (error) {
      profilesError = profileErrorMessage(error);
      return false;
    } finally {
      savingProfileId = null;
    }
  }

  async function saveProfile(id: string, input: ProfileInput): Promise<boolean> {
    savingProfileId = id;
    profilesError = null;

    try {
      const updated = await updateProfile(id, input);
      profiles = profiles.map((profile) => (profile.id === id ? updated : profile));
      return true;
    } catch (error) {
      profilesError = profileErrorMessage(error);
      return false;
    } finally {
      savingProfileId = null;
    }
  }

  async function removeProfile(id: string): Promise<boolean> {
    deletingProfileId = id;
    profilesError = null;

    try {
      await deleteProfile(id);
      profiles = profiles.filter((profile) => profile.id !== id);
      return true;
    } catch (error) {
      profilesError = profileErrorMessage(error);
      return false;
    } finally {
      deletingProfileId = null;
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
    void refreshProfiles();
  });
</script>

<svelte:head>
  <title>{activeSection === 'profiles' ? 'Profiles' : 'Overview'} · Git Identity Manager</title>
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
          <button
            type="button"
            class={`block w-full rounded-lg px-3 py-2 text-left text-sm transition ${
              activeSection === item.id
                ? 'bg-white/10 text-white'
                : isImplementedSection(item.id)
                  ? 'text-stone-300 hover:bg-white/5 hover:text-white'
                  : 'cursor-not-allowed text-stone-600'
            }`}
            onclick={() => selectSection(item.id)}
            disabled={!isImplementedSection(item.id)}
            aria-current={activeSection === item.id ? 'page' : undefined}
          >
            {item.label}
          </button>
        {/each}
      </nav>
    </aside>

    <section class="px-10 py-12">
      <div class="max-w-4xl">
        {#if activeSection === 'overview'}
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
        {:else}
          <p class="text-sm font-medium text-sky-300">Profiles</p>
          <h2 class="mt-3 text-4xl font-semibold tracking-tight text-white">Reusable identities</h2>
          <p class="mt-4 max-w-2xl text-base leading-7 text-stone-300">
            Keep Git names and emails consistent, and optionally connect each profile to an account
            already managed by GitHub CLI.
          </p>

          <div class="mt-10">
            <ProfilesPanel
              {profiles}
              accounts={githubStatus?.accounts ?? []}
              loading={profilesLoading}
              error={profilesError}
              {savingProfileId}
              {deletingProfileId}
              onRefresh={refreshProfiles}
              onCreate={addProfile}
              onUpdate={saveProfile}
              onDelete={removeProfile}
            />
          </div>
        {/if}
      </div>
    </section>
  </div>
</main>
