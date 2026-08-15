<script setup>
// The physics metaphor, animated. Replaces what we were going to build in
// Figma; this needs no feature flag and plays offline in the browser.
//
//   receiver: show the second bobber (the receiving antenna)
//   paused:   freeze everything, for a still frame or the PDF export
defineProps({
  receiver: { type: Boolean, default: false },
  paused: { type: Boolean, default: false },
})
</script>

<template>
  <svg
    class="bobbers"
    :class="{ paused }"
    viewBox="0 0 640 220"
    role="img"
    aria-label="A bobber oscillating in water, radiating waves outward to a second bobber"
  >
    <!-- still water -->
    <line class="water" x1="20" y1="110" x2="620" y2="110" />

    <!-- waves radiating outward from the transmitter -->
    <g class="rings">
      <circle v-for="n in 4" :key="n" cx="140" cy="110" :style="{ animationDelay: `${(n - 1) * 0.9}s` }" />
    </g>

    <!-- transmitter: pushed up and down -->
    <g class="bob tx">
      <line class="stem" x1="140" y1="110" x2="140" y2="74" />
      <circle cx="140" cy="66" r="13" />
    </g>

    <!-- receiver: driven by the arriving wave, so it lags -->
    <g v-if="receiver" class="bob rx">
      <line class="stem" x1="500" y1="110" x2="500" y2="74" />
      <circle cx="500" cy="66" r="13" />
    </g>

    <text class="label" x="140" y="176">transmitter</text>
    <text v-if="receiver" class="label" x="500" y="176">receiver</text>
  </svg>
</template>

<style scoped>
.bobbers {
  width: 100%;
  max-width: 640px;
  margin: 0 auto;
  display: block;
  overflow: visible;
}

.water {
  stroke: currentColor;
  stroke-width: 1;
  opacity: 0.25;
}

.rings circle {
  fill: none;
  stroke: currentColor;
  stroke-width: 2;
  animation: ring 3.6s linear infinite;
}

.bob circle {
  fill: currentColor;
}

.stem {
  stroke: currentColor;
  stroke-width: 2;
  opacity: 0.4;
}

/* Both bobbers ride the same wave; the receiver is a quarter period behind,
   because the wave has to travel to reach it. */
.tx {
  animation: bob 1.8s ease-in-out infinite;
}
.rx {
  animation: bob 1.8s ease-in-out infinite;
  animation-delay: -0.45s;
}

.label {
  fill: currentColor;
  opacity: 0.55;
  font-size: 15px;
  font-family: inherit;
  text-anchor: middle;
}

@keyframes bob {
  0%,
  100% {
    transform: translateY(-16px);
  }
  50% {
    transform: translateY(16px);
  }
}

@keyframes ring {
  from {
    r: 8;
    opacity: 0.5;
  }
  to {
    r: 250;
    opacity: 0;
  }
}

/* Frozen for stills, PDF export, and anyone who asked for less motion. */
.paused *,
:global(.print-mode) .bobbers * {
  animation-play-state: paused !important;
}

@media (prefers-reduced-motion: reduce) {
  .bobbers * {
    animation-play-state: paused !important;
  }
}
</style>
