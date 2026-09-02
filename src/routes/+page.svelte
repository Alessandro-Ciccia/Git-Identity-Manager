<script lang="ts">
  import { navigationItems } from '$lib/domain/navigation';

  const pillars = [
    'Narrow Tauri commands; no arbitrary shell execution',
    'Local-only persistence; no GitHub tokens or secrets stored',
    'Git and GitHub CLI integrations isolated behind Rust services',
  ];
</script>

<svelte:head>
  <title>Git Identity Manager</title>
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
          >
            {item.label}
          </a>
        {/each}
      </nav>
    </aside>

    <section class="px-10 py-12">
      <div class="max-w-4xl">
        <p class="text-sm font-medium text-sky-300">Milestone 0 bootstrap</p>
        <h2 class="mt-3 text-4xl font-semibold tracking-tight text-white">
          A safe desktop control plane for Git and GitHub identities.
        </h2>
        <p class="mt-5 max-w-2xl text-base leading-7 text-stone-300">
          The project shell is ready for incremental milestones. Product features begin in
          Milestone 1 with dependency detection from the Rust backend.
        </p>

        <div class="mt-10 grid gap-4 md:grid-cols-3">
          {#each pillars as pillar (pillar)}
            <article class="rounded-2xl border border-white/10 bg-white/[0.03] p-5 shadow-2xl shadow-black/10">
              <p class="text-sm leading-6 text-stone-200">{pillar}</p>
            </article>
          {/each}
        </div>

        <section class="mt-10 rounded-2xl border border-white/10 bg-stone-900/50 p-6" aria-labelledby="next-step">
          <h3 id="next-step" class="text-base font-semibold text-white">Next implementation step</h3>
          <p class="mt-2 text-sm leading-6 text-stone-300">
            Add typed Tauri commands for environment status, detecting <code>git</code> and
            <code>gh</code> without exposing shell access to the frontend.
          </p>
        </section>
      </div>
    </section>
  </div>
</main>
