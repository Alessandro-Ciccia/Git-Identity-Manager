<script lang="ts">
  type Props = {
    /** Total time the splash stays before it is removed, in ms. */
    duration?: number;
    ondone: () => void;
  };

  let { duration = 2000, ondone }: Props = $props();

  const FADE_MS = 320;
  let leaving = $state(false);

  $effect(() => {
    const startFade = globalThis.setTimeout(
      () => {
        leaving = true;
      },
      Math.max(0, duration - FADE_MS),
    );
    const finish = globalThis.setTimeout(ondone, duration);
    return () => {
      globalThis.clearTimeout(startFade);
      globalThis.clearTimeout(finish);
    };
  });
</script>

<div class="splash" class:splash--leaving={leaving} aria-hidden="true" data-testid="splash-screen">
  <div class="splash__stage">
    <span class="splash__halo"></span>
    <img class="splash__icon" src="/splash-icon.png" alt="" width="132" height="132" />
  </div>
</div>

<style>
  .splash {
    position: fixed;
    inset: 0;
    z-index: 100;
    display: grid;
    place-items: center;
    background:
      radial-gradient(
        circle at 50% 45%,
        rgba(144, 80, 248, 0.2) 0%,
        rgba(64, 40, 176, 0.07) 40%,
        transparent 70%
      ),
      var(--canvas);
    opacity: 1;
    transition: opacity 320ms ease;
  }

  .splash--leaving {
    opacity: 0;
    pointer-events: none;
  }

  .splash__stage {
    position: relative;
    display: grid;
    place-items: center;
  }

  .splash__halo {
    position: absolute;
    width: 300px;
    height: 300px;
    border-radius: 9999px;
    background: radial-gradient(
      circle,
      rgba(144, 80, 248, 0.5) 0%,
      rgba(144, 80, 248, 0.24) 36%,
      rgba(64, 40, 176, 0.1) 56%,
      transparent 72%
    );
    filter: blur(10px);
    animation:
      splash-halo-in 720ms cubic-bezier(0.22, 1, 0.36, 1) both,
      splash-halo-pulse 1.9s ease-in-out 720ms infinite;
  }

  .splash__icon {
    position: relative;
    display: block;
    filter: drop-shadow(0 0 20px rgba(144, 80, 248, 0.55))
      drop-shadow(0 0 54px rgba(144, 80, 248, 0.32));
    animation: splash-pop 760ms cubic-bezier(0.22, 1, 0.36, 1) both;
  }

  @keyframes splash-pop {
    0% {
      transform: scale(0.55);
      opacity: 0;
    }
    55% {
      opacity: 1;
    }
    100% {
      transform: scale(1);
      opacity: 1;
    }
  }

  @keyframes splash-halo-in {
    0% {
      transform: scale(0.35);
      opacity: 0;
    }
    100% {
      transform: scale(1);
      opacity: 1;
    }
  }

  @keyframes splash-halo-pulse {
    0%,
    100% {
      transform: scale(0.92);
      opacity: 0.68;
    }
    50% {
      transform: scale(1.12);
      opacity: 1;
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .splash__halo,
    .splash__icon {
      animation: none;
    }
  }
</style>
