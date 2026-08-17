<script setup>
// The physics metaphor, animated. Replaces what we were going to build in
// Figma; needs no feature flag and plays offline in the browser.
//
// The wave EMANATES from the transmitter: still water to its left, full swing at
// the source, decaying with distance as the energy spreads out. That look needs
// a spatial amplitude envelope anchored at the source — which a translating path
// can't have (moving the path drags its envelope along). So instead of sliding a
// pre-built sine, we hold the envelope fixed in space and advance only the phase
// each frame. Crests are born at the bobber, travel right, and shrink as they go.
//
// Because both bobbers read their height from the same wave function, the
// receiver's lag and its smaller swing fall out for free: it is exactly as far
// behind as the wave takes to reach it, and as weak as the wave is when it gets
// there. No hand-tuned delay.
//
// A frozen frame still reads correctly (it is a decaying sine off a point
// source), which matters because the exported PDF is the backup deck. When we
// are printing, or the viewer asked for reduced motion, we never start the loop
// and just render the initial frame.
//
//   receiver: show the second bobber (the receiving antenna)
//   paused:   freeze everything, for a still frame
import { ref, computed, onMounted, onBeforeUnmount } from 'vue'

const props = defineProps({
  receiver: { type: Boolean, default: false },
  paused: { type: Boolean, default: false },
})

const BASE = 130 // water line
const TX = 140 // transmitter x (the source)
const RX = 500 // receiver x
const WAVELENGTH = 96
const AMP = 17 // swing at the source
const SPREAD = 320 // amplitude ~ 1/(1 + d/SPREAD): full at the source, weaker away
const XMAX = 730 // wave drawn from the source out to here
const PERIOD = 1.8 // seconds per oscillation
const K = (2 * Math.PI) / WAVELENGTH

// Amplitude as a function of position: zero left of the source, full at it,
// falling off with distance, and eased to zero over the last stretch so the
// train fades out instead of ending on a hard vertical edge.
function envelope(x) {
  if (x < TX) return 0
  const d = x - TX
  const spread = 1 / (1 + d / SPREAD)
  const edge = Math.min(1, (XMAX - x) / 70)
  return AMP * spread * Math.max(0, edge)
}

// Phase grows with time; a larger phase slides the pattern in +x, i.e. away from
// the source. The envelope does not move, so the motion reads as radiation.
function waveY(x, phase) {
  return BASE - envelope(x) * Math.sin(K * (x - TX) - phase)
}

const phase = ref(0)

const wavePath = computed(() => {
  const pts = []
  for (let x = TX; x <= XMAX; x += 3) {
    pts.push(`${x === TX ? 'M' : 'L'}${x} ${waveY(x, phase.value).toFixed(2)}`)
  }
  return pts.join(' ')
})

// Both bobbers sit exactly on the wave.
const txY = computed(() => waveY(TX, phase.value))
const rxY = computed(() => waveY(RX, phase.value))

let raf = 0
let last = 0
function tick(now) {
  if (last) phase.value += (2 * Math.PI * (now - last)) / (PERIOD * 1000)
  last = now
  raf = requestAnimationFrame(tick)
}

const still =
  props.paused ||
  (typeof matchMedia !== 'undefined' && matchMedia('(prefers-reduced-motion: reduce)').matches) ||
  (typeof document !== 'undefined' && !!document.querySelector('.print-mode'))

onMounted(() => {
  if (!still) raf = requestAnimationFrame(tick)
})
onBeforeUnmount(() => cancelAnimationFrame(raf))
</script>

<template>
  <svg
    class="bobbers"
    viewBox="0 0 640 200"
    role="img"
    aria-label="A bobber oscillating in water, radiating waves along the surface to a second bobber"
  >
    <!-- still water, for reference (calm to the left of the source) -->
    <line class="water" x1="10" y1="130" x2="630" y2="130" />

    <!-- the wave, radiating from the transmitter -->
    <path class="wave" :d="wavePath" />

    <!-- transmitter: the source, driven up and down -->
    <g class="bob tx" :transform="`translate(0 ${(txY - BASE).toFixed(2)})`">
      <line class="stem" :x1="TX" :y1="BASE - 13" :x2="TX" :y2="BASE - 44" />
      <circle :cx="TX" :cy="BASE" r="13" />
    </g>

    <!-- receiver: driven by the arriving wave, so it lags and swings less -->
    <g v-if="receiver" class="bob rx" :transform="`translate(0 ${(rxY - BASE).toFixed(2)})`">
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
  stroke-linecap: round;
  opacity: 0.55;
}

.bob circle {
  fill: currentColor;
}

.stem {
  stroke: currentColor;
  stroke-width: 2;
  opacity: 0.35;
}

.label {
  fill: currentColor;
  opacity: 0.5;
  font-size: 15px;
  font-family: inherit;
  text-anchor: middle;
}
</style>
