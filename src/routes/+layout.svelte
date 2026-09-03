<script lang="ts">
  import '../app.css';

  import SplashScreen from '$lib/components/SplashScreen.svelte';

  let { children } = $props();

  let showSplash = $state(true);
</script>

{#if showSplash}
  <SplashScreen ondone={() => (showSplash = false)} />
{/if}

{@render children()}
