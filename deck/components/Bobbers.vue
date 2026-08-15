<script setup>
// The physics metaphor, animated. Replaces what we were going to build in
// Figma; needs no feature flag and plays offline in the browser.
//
// The wave is a travelling sine along the water surface, not concentric rings.
// Rings looked like a bullseye centred on the bobber, and a static export only
// caught one of them; a sine still reads correctly when frozen, which matters
// because the PDF is the backup deck.
//
//   receiver: show the second bobber (the receiving antenna)
//   paused:   freeze everything, for a still frame
defineProps({
  receiver: { type: Boolean, default: false },
  paused: { type: Boolean, default: false },
})

const BASE = 130 // water line
const TX = 140 // transmitter x
const RX = 500 // receiver x
const WAVELENGTH = 96
const AMP = 17

// One long constant-amplitude sine. Decay is applied by a mask that does NOT
// move, so translating the wave slides the crests without dragging the
// envelope along with them.
const wave = (() => {
  const pts = []
  for (let x = -WAVELENGTH; x <= 760; x += 4) {
    const y = BASE - AMP * Math.sin((2 * Math.PI * (x - TX)) / WAVELENGTH)
    pts.push(`${x === -WAVELENGTH ? 'M' : 'L'}${x} ${y.toFixed(2)}`)
  }
  return pts.join(' ')
})()

// Unique per instance: two of these can be in the DOM at once during a slide
// transition, and duplicate mask ids would cross-wire them.
const uid = `bob${Math.floor(performance.now() * 1000) % 1e9}`
</script>

<template>
  <svg
    class="bobbers"
    :class="{ paused }"
    viewBox="0 0 640 200"
    role="img"
    aria-label="A bobber oscillating in water, sending waves along the surface to a second bobber"
  >
    <defs>
      <linearGradient :id="`${uid}-fade`" x1="0" x2="1">
        <stop offset="0" stop-color="white" stop-opacity="0" />
        <stop offset="0.22" stop-color="white" stop-opacity="1" />
        <stop offset="0.78" stop-color="white" stop-opacity="0.45" />
        <stop offset="1" stop-color="white" stop-opacity="0" />
      </linearGradient>
      <mask :id="`${uid}-mask`">
        <rect x="0" y="0" width="640" height="200" :fill="`url(#${uid}-fade)`" />
      </mask>
    </defs>

    <!-- still water, for reference -->
    <line class="water" x1="10" y1="130" x2="630" y2="130" />

    <!-- the wave, travelling left to right -->
    <g :mask="`url(#${uid}-mask)`">
      <path class="wave" :d="wave" />
    </g>

    <!-- transmitter: pushed up and down -->
    <g class="bob tx">
      <line class="stem" :x1="TX" :y1="BASE - 13" :x2="TX" :y2="BASE - 44" />
      <circle :cx="TX" :cy="BASE" r="13" />
    </g>

    <!-- receiver: driven by the arriving wave, so it lags -->
    <g v-if="receiver" class="bob rx">
      <line class="stem" :x1="RX" :y1="BASE - 13" :x2="RX" :y2="BASE - 44" />
      <circle :cx="RX" :cy="BASE" r="13" />
    </g>

    <text class="label" :x="TX" y="186">transmitter</text>
    <text v-if="receiver" class="label" :x="RX" y="186">receiver</text>
  </svg>
</template>

<style scoped>
.bobbers {
  width: 100%;
  max-width: 620px;
  margin: 0 auto;
  display: block;
}

.water {
  stroke: currentColor;
  stroke-width: 1;
  opacity: 0.2;
}

.wave {
  fill: none;
  stroke: currentColor;
  stroke-width: 2.5;
  opacity: 0.5;
  animation: travel 1.8s linear infinite;
}

.bob circle {
  fill: currentColor;
}

.stem {
  stroke: currentColor;
  stroke-width: 2;
  opacity: 0.35;
}

/* Both bobbers ride the same wave. The receiver is three quarters of a period
   behind, because the wave has to travel 360px to reach it and that is 3.75
   wavelengths. */
.tx,
.rx {
  animation: bob 1.8s linear infinite;
}

/* The wave covers the 360px to the receiver in 3.75 wavelengths, so the
   receiver lags by 0.75 of a period. For a sine that is the same shape as
   leading by a quarter period, which is what a negative delay expresses. */
.rx {
  animation-delay: -0.45s;
}

.label {
  fill: currentColor;
  opacity: 0.5;
  font-size: 15px;
  font-family: inherit;
  text-anchor: middle;
}

@keyframes bob {
  0%   { transform: translateY(0); }
  12.5% { transform: translateY(12px); }
  25%  { transform: translateY(17px); }
  37.5% { transform: translateY(12px); }
  50%  { transform: translateY(0); }
  62.5% { transform: translateY(-12px); }
  75%  { transform: translateY(-17px); }
  87.5% { transform: translateY(-12px); }
  100% { transform: translateY(0); }
}

/* One wavelength per period, so crests and bobbers stay coherent. */
@keyframes travel {
  from {
    transform: translateX(0);
  }
  to {
    transform: translateX(96px);
  }
}

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
