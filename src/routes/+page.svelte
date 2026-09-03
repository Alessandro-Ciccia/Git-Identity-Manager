<script lang="ts">
  import { onMount } from 'svelte';

  import DirectoryRulesPanel from '$lib/components/DirectoryRulesPanel.svelte';
  import EnvironmentStatusPanel from '$lib/components/EnvironmentStatusPanel.svelte';
  import GithubAccountsPanel from '$lib/components/GithubAccountsPanel.svelte';
  import ProfilesPanel from '$lib/components/ProfilesPanel.svelte';
  import RepositoriesPanel from '$lib/components/RepositoriesPanel.svelte';
  import SettingsPanel from '$lib/components/SettingsPanel.svelte';
  import WelcomeScreen from '$lib/components/WelcomeScreen.svelte';
  import type {
    DirectoryRule,
    DirectoryRuleInput,
    DirectoryRulePreview,
  } from '$lib/domain/directoryRules';
  import type { EnvironmentStatus } from '$lib/domain/environment';
  import {
    githubAccountKey,
    hasNewGithubAccount,
    type GithubAccount,
    type GithubAccountsStatus,
  } from '$lib/domain/github';
  import { navigationItems, type NavigationSection } from '$lib/domain/navigation';
  import type { GitProfile, ProfileInput } from '$lib/domain/profiles';
  import type { RegisteredRepository, RepositoryProfilePreview } from '$lib/domain/repositories';
  import {
    applyDirectoryRule,
    directoryRuleErrorMessage,
    listDirectoryRules,
    previewDirectoryRule,
    previewRemoveDirectoryRule,
    removeDirectoryRule,
  } from '$lib/ipc/directoryRules';
  import { environmentErrorMessage, getEnvironmentStatus } from '$lib/ipc/environment';
  import {
    getPreferences,
    preferenceErrorMessage,
    setThemePreference,
    setWelcomeDismissed,
  } from '$lib/ipc/preferences';
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
  import {
    applyRepositoryProfile,
    assignRepositoryProfile,
    listRepositories,
    previewRepositoryProfile,
    refreshRepository,
    registerRepository,
    removeRepository,
    removeRepositoryProfile,
    repositoryErrorMessage,
    revealRepository,
  } from '$lib/ipc/repositories';
  import { selectRepositoryDirectory } from '$lib/native/directoryPicker';
  import type { ThemePreference } from '$lib/domain/preferences';
  import { theme } from '$lib/state/theme.svelte';

  type ImplementedSection = Extract<
    NavigationSection,
    'overview' | 'profiles' | 'repositories' | 'rules' | 'settings'
  >;

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
  let repositories = $state<RegisteredRepository[]>([]);
  let repositoriesLoading = $state(true);
  let repositoriesError = $state<string | null>(null);
  let repositoriesSuccess = $state<string | null>(null);
  let addingRepository = $state(false);
  let refreshingRepositoryId = $state<string | null>(null);
  let removingRepositoryId = $state<string | null>(null);
  let revealingRepositoryId = $state<string | null>(null);
  let assigningRepositoryId = $state<string | null>(null);
  let previewingRepositoryId = $state<string | null>(null);
  let applyingRepositoryId = $state<string | null>(null);
  let repositoryProfilePreview = $state<RepositoryProfilePreview | null>(null);
  let directoryRules = $state<DirectoryRule[]>([]);
  let directoryRulesLoading = $state(true);
  let directoryRulesError = $state<string | null>(null);
  let directoryRulesSuccess = $state<string | null>(null);
  let directoryRulePending = $state(false);
  let directoryRulePreview = $state<DirectoryRulePreview | null>(null);
  let themePreference = $state<ThemePreference>('system');
  let preferencesLoading = $state(true);
  let preferencesError = $state<string | null>(null);
  let themePending = $state(false);
  let welcomeDismissed = $state(true);
  let welcomeSkipping = $state(false);
  let welcomeSkipError = $state<string | null>(null);

  const initialLoadPending = $derived(
    isLoading ||
      githubLoading ||
      profilesLoading ||
      repositoriesLoading ||
      directoryRulesLoading ||
      preferencesLoading,
  );
  const welcomeApplies = $derived(
    !welcomeDismissed &&
      !initialLoadPending &&
      status !== null &&
      (!status.isReady ||
        (profiles.length === 0 && repositories.length === 0 && directoryRules.length === 0)),
  );
  // The welcome screen stands in for the Overview until setup is done; every other
  // section stays reachable from the sidebar so it never traps navigation.
  const showWelcome = $derived(welcomeApplies && activeSection === 'overview');

  const LOGIN_POLL_INTERVAL_MS = 3_000;
  const LOGIN_POLL_ATTEMPTS = 40;

  function wait(milliseconds: number): Promise<void> {
    return new Promise((resolve) => globalThis.setTimeout(resolve, milliseconds));
  }

  function isImplementedSection(section: NavigationSection): section is ImplementedSection {
    return (
      section === 'overview' ||
      section === 'profiles' ||
      section === 'repositories' ||
      section === 'rules' ||
      section === 'settings'
    );
  }

  function selectSection(section: NavigationSection): void {
    if (isImplementedSection(section)) {
      activeSection = section;
    }
  }

  function onNavKeydown(event: globalThis.KeyboardEvent): void {
    const keys = ['ArrowDown', 'ArrowUp', 'Home', 'End'];
    if (!keys.includes(event.key)) {
      return;
    }
    event.preventDefault();
    const buttons = Array.from(
      event.currentTarget instanceof globalThis.HTMLElement
        ? event.currentTarget.querySelectorAll<globalThis.HTMLButtonElement>(
            'button:not([disabled])',
          )
        : [],
    );
    if (buttons.length === 0) {
      return;
    }
    const currentIndex = buttons.findIndex(
      (button) => button === globalThis.document.activeElement,
    );
    let nextIndex: number;
    if (event.key === 'Home') {
      nextIndex = 0;
    } else if (event.key === 'End') {
      nextIndex = buttons.length - 1;
    } else {
      const delta = event.key === 'ArrowDown' ? 1 : -1;
      const base = currentIndex === -1 ? 0 : currentIndex;
      nextIndex = (base + delta + buttons.length) % buttons.length;
    }
    buttons[nextIndex]?.focus();
  }

  async function refreshPreferences(): Promise<void> {
    preferencesLoading = true;
    preferencesError = null;

    try {
      const preferences = await getPreferences();
      themePreference = preferences.theme;
      welcomeDismissed = preferences.welcomeDismissed;
      theme.init(preferences.theme);
    } catch (error) {
      preferencesError = preferenceErrorMessage(error);
    } finally {
      preferencesLoading = false;
    }
  }

  async function changeTheme(preference: ThemePreference): Promise<void> {
    const previous = themePreference;
    themePreference = preference;
    theme.setPreference(preference);
    themePending = true;
    preferencesError = null;

    try {
      const preferences = await setThemePreference(preference);
      themePreference = preferences.theme;
      theme.setPreference(preferences.theme);
    } catch (error) {
      themePreference = previous;
      theme.setPreference(previous);
      preferencesError = preferenceErrorMessage(error);
    } finally {
      themePending = false;
    }
  }

  async function skipWelcome(): Promise<void> {
    welcomeSkipping = true;
    welcomeSkipError = null;

    try {
      const preferences = await setWelcomeDismissed(true);
      welcomeDismissed = preferences.welcomeDismissed;
    } catch (error) {
      welcomeSkipError = preferenceErrorMessage(error);
    } finally {
      welcomeSkipping = false;
    }
  }

  function startProfileFromWelcome(): void {
    activeSection = 'profiles';
  }

  function startRepositoryFromWelcome(): void {
    activeSection = 'repositories';
    void addRepository();
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

  async function refreshRepositories(): Promise<void> {
    repositoriesLoading = true;
    repositoriesError = null;

    try {
      repositories = await listRepositories();
    } catch (error) {
      repositoriesError = repositoryErrorMessage(error);
    } finally {
      repositoriesLoading = false;
    }
  }

  async function refreshDirectoryRules(): Promise<void> {
    directoryRulesLoading = true;
    directoryRulesError = null;

    try {
      directoryRules = await listDirectoryRules();
    } catch (error) {
      directoryRulesError = directoryRuleErrorMessage(error);
    } finally {
      directoryRulesLoading = false;
    }
  }

  async function chooseRuleDirectory(): Promise<string | null> {
    directoryRulesError = null;
    try {
      return await selectRepositoryDirectory();
    } catch (error) {
      directoryRulesError = directoryRuleErrorMessage(error);
      return null;
    }
  }

  async function previewRule(input: DirectoryRuleInput): Promise<void> {
    directoryRulePending = true;
    directoryRulesError = null;
    directoryRulesSuccess = null;
    try {
      directoryRulePreview = await previewDirectoryRule(input);
    } catch (error) {
      directoryRulesError = directoryRuleErrorMessage(error);
    } finally {
      directoryRulePending = false;
    }
  }

  async function previewRuleRemoval(id: string): Promise<void> {
    directoryRulePending = true;
    directoryRulesError = null;
    directoryRulesSuccess = null;
    try {
      directoryRulePreview = await previewRemoveDirectoryRule(id);
    } catch (error) {
      directoryRulesError = directoryRuleErrorMessage(error);
    } finally {
      directoryRulePending = false;
    }
  }

  function cancelRulePreview(): void {
    if (!directoryRulePending) directoryRulePreview = null;
  }

  async function applyRulePreview(): Promise<void> {
    const preview = directoryRulePreview;
    if (!preview || !preview.canApply) return;
    directoryRulePending = true;
    directoryRulesError = null;
    directoryRulesSuccess = null;
    try {
      if (preview.operation === 'remove' && preview.ruleId) {
        await removeDirectoryRule(preview.ruleId);
        directoryRules = directoryRules.filter((rule) => rule.id !== preview.ruleId);
        directoryRulesSuccess = 'Directory rule removed and Git configuration verified.';
      } else if (preview.profile) {
        const updated = await applyDirectoryRule({
          id: preview.ruleId,
          directory: preview.directory,
          profileId: preview.profile.id,
        });
        directoryRules = [...directoryRules.filter((rule) => rule.id !== updated.id), updated];
        directoryRulesSuccess = 'Directory rule applied and Git configuration verified.';
      }
      directoryRulePreview = null;
      await refreshRepositories();
    } catch (error) {
      const message = directoryRuleErrorMessage(error);
      await refreshDirectoryRules();
      directoryRulesError = message;
    } finally {
      directoryRulePending = false;
    }
  }

  async function addRepository(): Promise<void> {
    addingRepository = true;
    repositoriesError = null;

    try {
      const path = await selectRepositoryDirectory();
      if (!path) {
        return;
      }
      const repository = await registerRepository(path);
      repositories = [
        ...repositories.filter((existing) => existing.id !== repository.id),
        repository,
      ];
    } catch (error) {
      repositoriesError = repositoryErrorMessage(error);
    } finally {
      addingRepository = false;
    }
  }

  async function refreshRegisteredRepository(id: string): Promise<void> {
    refreshingRepositoryId = id;
    repositoriesError = null;

    try {
      const refreshed = await refreshRepository(id);
      repositories = repositories.map((repository) =>
        repository.id === id ? refreshed : repository,
      );
    } catch (error) {
      repositoriesError = repositoryErrorMessage(error);
    } finally {
      refreshingRepositoryId = null;
    }
  }

  async function assignRegisteredRepositoryProfile(
    id: string,
    profileId: string | null,
  ): Promise<void> {
    assigningRepositoryId = id;
    repositoriesError = null;
    repositoriesSuccess = null;

    try {
      const updated = profileId
        ? await assignRepositoryProfile(id, profileId)
        : await removeRepositoryProfile(id);
      repositories = repositories.map((repository) =>
        repository.id === id ? updated : repository,
      );
    } catch (error) {
      repositoriesError = repositoryErrorMessage(error);
    } finally {
      assigningRepositoryId = null;
    }
  }

  async function previewRegisteredRepositoryProfile(id: string): Promise<void> {
    previewingRepositoryId = id;
    repositoriesError = null;
    repositoriesSuccess = null;

    try {
      repositoryProfilePreview = await previewRepositoryProfile(id);
    } catch (error) {
      repositoriesError = repositoryErrorMessage(error);
    } finally {
      previewingRepositoryId = null;
    }
  }

  function cancelRepositoryProfilePreview(): void {
    if (!applyingRepositoryId) {
      repositoryProfilePreview = null;
    }
  }

  async function applyRegisteredRepositoryProfile(): Promise<void> {
    const preview = repositoryProfilePreview;
    if (!preview) {
      return;
    }

    const id = preview.repositoryId;
    applyingRepositoryId = id;
    repositoriesError = null;
    repositoriesSuccess = null;

    try {
      const updated = await applyRepositoryProfile(id);
      repositories = repositories.map((repository) =>
        repository.id === id ? updated : repository,
      );
      repositoriesSuccess = `Verified ${preview.profile.label} in this repository’s local Git configuration.`;
      repositoryProfilePreview = null;
    } catch (error) {
      repositoriesError = repositoryErrorMessage(error);
      try {
        const refreshed = await refreshRepository(id);
        repositories = repositories.map((repository) =>
          repository.id === id ? refreshed : repository,
        );
      } catch {
        // Preserve the authoritative apply error if follow-up inspection also fails.
      }
    } finally {
      applyingRepositoryId = null;
    }
  }

  async function removeRegisteredRepository(id: string): Promise<boolean> {
    removingRepositoryId = id;
    repositoriesError = null;

    try {
      await removeRepository(id);
      repositories = repositories.filter((repository) => repository.id !== id);
      return true;
    } catch (error) {
      repositoriesError = repositoryErrorMessage(error);
      return false;
    } finally {
      removingRepositoryId = null;
    }
  }

  async function revealRegisteredRepository(id: string): Promise<void> {
    revealingRepositoryId = id;
    repositoriesError = null;

    try {
      await revealRepository(id);
    } catch (error) {
      repositoriesError = repositoryErrorMessage(error);
    } finally {
      revealingRepositoryId = null;
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

  const sectionTitles: Record<ImplementedSection, string> = {
    overview: 'Overview',
    profiles: 'Profiles',
    repositories: 'Repositories',
    rules: 'Rules',
    settings: 'Settings',
  };

  onMount(() => {
    void refreshPreferences();
    void refreshEnvironment();
    void refreshGithubAccounts();
    void refreshProfiles();
    void refreshRepositories();
    void refreshDirectoryRules();
  });
</script>

<svelte:head>
  <title>{showWelcome ? 'Welcome' : sectionTitles[activeSection]} · Git Identity Manager</title>
</svelte:head>

<main class="h-screen overflow-hidden bg-canvas text-fg-strong">
  <div class="grid h-full grid-cols-[220px_1fr]">
    <aside class="flex h-full flex-col overflow-hidden border-r border-edge bg-canvas/80 p-4">
      <div class="mb-8">
        <p class="text-xs font-semibold uppercase tracking-[0.24em] text-accent-text">
          Local-first
        </p>
        <h1 class="mt-2 text-lg font-semibold">Git Identity Manager</h1>
      </div>

      <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
      <nav aria-label="Primary navigation" class="space-y-1" onkeydown={onNavKeydown}>
        {#each navigationItems as item (item.id)}
          {@const implemented = isImplementedSection(item.id)}
          <button
            type="button"
            class={`block w-full rounded-lg px-3 py-2 text-left text-sm transition ${
              activeSection === item.id
                ? 'bg-hover-strong text-fg-strong'
                : implemented
                  ? 'text-fg hover:bg-hover hover:text-fg-strong'
                  : 'cursor-not-allowed text-fg-faint'
            }`}
            onclick={() => selectSection(item.id)}
            disabled={!implemented}
            tabindex={activeSection === item.id ? 0 : -1}
            aria-current={activeSection === item.id ? 'page' : undefined}
          >
            {item.label}
          </button>
        {/each}
      </nav>
    </aside>

    <section class="h-full overflow-y-auto px-10 py-12">
      <div class="max-w-4xl">
        {#if showWelcome}
          <WelcomeScreen
            {status}
            envLoading={isLoading}
            hasProfiles={profiles.length > 0}
            hasRepositories={repositories.length > 0}
            skipping={welcomeSkipping}
            skipError={welcomeSkipError}
            onRecheck={refreshEnvironment}
            onCreateProfile={startProfileFromWelcome}
            onAddRepository={startRepositoryFromWelcome}
            onSkip={skipWelcome}
          />
        {:else if activeSection === 'overview'}
          <p class="text-sm font-medium text-accent-text">Overview</p>
          <h2 class="mt-3 text-4xl font-semibold tracking-tight text-fg-strong">
            Identity overview
          </h2>
          <p class="mt-4 max-w-2xl text-base leading-7 text-fg">
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
        {:else if activeSection === 'profiles'}
          <p class="text-sm font-medium text-accent-text">Profiles</p>
          <h2 class="mt-3 text-4xl font-semibold tracking-tight text-fg-strong">
            Reusable identities
          </h2>
          <p class="mt-4 max-w-2xl text-base leading-7 text-fg">
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
        {:else if activeSection === 'repositories'}
          <p class="text-sm font-medium text-accent-text">Repositories</p>
          <h2 class="mt-3 text-4xl font-semibold tracking-tight text-fg-strong">
            Repository identities
          </h2>
          <p class="mt-4 max-w-2xl text-base leading-7 text-fg">
            Inspect each repository’s effective Git identity, configuration source, and remotes
            without changing local or global Git configuration.
          </p>

          <div class="mt-10">
            <RepositoriesPanel
              {repositories}
              {profiles}
              {githubStatus}
              loading={repositoriesLoading}
              error={repositoriesError}
              success={repositoriesSuccess}
              adding={addingRepository}
              refreshingId={refreshingRepositoryId}
              removingId={removingRepositoryId}
              revealingId={revealingRepositoryId}
              assigningId={assigningRepositoryId}
              previewingId={previewingRepositoryId}
              applyingId={applyingRepositoryId}
              preview={repositoryProfilePreview}
              onAdd={addRepository}
              onRefreshAll={refreshRepositories}
              onRefresh={refreshRegisteredRepository}
              onRemove={removeRegisteredRepository}
              onReveal={revealRegisteredRepository}
              onAssignProfile={assignRegisteredRepositoryProfile}
              onPreviewProfile={previewRegisteredRepositoryProfile}
              onCancelPreview={cancelRepositoryProfilePreview}
              onApplyProfile={applyRegisteredRepositoryProfile}
            />
          </div>
        {:else if activeSection === 'rules'}
          <p class="text-sm font-medium text-accent-text">Rules</p>
          <h2 class="mt-3 text-4xl font-semibold tracking-tight text-fg-strong">
            Directory identity rules
          </h2>
          <p class="mt-4 max-w-2xl text-base leading-7 text-fg">
            Automatically resolve a profile for repositories below a directory through safe,
            previewed Git conditional includes.
          </p>

          <div class="mt-10">
            <DirectoryRulesPanel
              rules={directoryRules}
              {profiles}
              loading={directoryRulesLoading}
              error={directoryRulesError}
              success={directoryRulesSuccess}
              pending={directoryRulePending}
              preview={directoryRulePreview}
              onRefresh={refreshDirectoryRules}
              onChooseDirectory={chooseRuleDirectory}
              onPreview={previewRule}
              onPreviewRemove={previewRuleRemoval}
              onCancelPreview={cancelRulePreview}
              onApply={applyRulePreview}
            />
          </div>
        {:else}
          <p class="text-sm font-medium text-accent-text">Settings</p>
          <h2 class="mt-3 text-4xl font-semibold tracking-tight text-fg-strong">
            Application preferences
          </h2>
          <p class="mt-4 max-w-2xl text-base leading-7 text-fg">
            Adjust how Git Identity Manager looks and review where it stores data on this computer.
          </p>

          <div class="mt-10">
            <SettingsPanel
              theme={themePreference}
              resolved={theme.resolved}
              pending={themePending || preferencesLoading}
              error={preferencesError}
              onSetTheme={changeTheme}
            />
          </div>
        {/if}
      </div>
    </section>
  </div>
</main>
