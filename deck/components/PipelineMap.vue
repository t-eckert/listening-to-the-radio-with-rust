<script setup>
// A sample-rate pipeline shown as a MAP, revealed in beats so it can be walked:
//   base    — the endpoints (in / out)
//   click 1 — the step cards (a table of contents for the slides that follow)
//   click 2 — the sample rate on each hop
//   click 3 — the whole-number divisions (the design punchline)
//
// Defaults describe the FM receiver. Pass `steps`/`rates` to reuse it — the AM
// receiver is the same map with two middle names changed. The final downsample
// (÷5) folds onto the last step, matching the code (Step 1's low-pass reused on
// the audio, with the last step running at the audio rate).
defineProps({
  input: { type: String, default: 'IQ samples' },
  output: { type: String, default: 'speakers' },
  steps: {
    type: Array,
    default: () => [
      { n: 1, name: 'Filter', sub: 'pick one station out of the noise', divide: 4 },
      { n: 2, name: 'Demodulate', sub: 'turn rotation into sound' },
      { n: 3, name: 'De-emphasis', sub: 'downsample, then undo the treble boost', divide: 5 },
    ],
  },
  rates: { type: Array, default: () => ['960 kHz', '240 kHz', '240 kHz', '48 kHz'] },
})
</script>

<template>
  <div class="pmap">
    <div class="endpoint">{{ input }}</div>

    <template v-for="(s, i) in steps" :key="s.n">
      <!-- hop into this step: arrow (with the steps) + rate (one beat later) -->
      <div class="hop">
        <svg class="chev" v-click="1" viewBox="0 0 12 30" aria-hidden="true">
          <line x1="6" y1="0" x2="6" y2="20" />
          <path d="M1.5 18 L6 28 L10.5 18 Z" />
        </svg>
        <span class="rate" v-click="2">{{ rates[i] }}</span>
      </div>

      <div class="step" v-click="1">
        <span class="badge">{{ s.n }}</span>
        <span class="name">{{ s.name }}</span>
        <span class="sub">{{ s.sub }}</span>
        <span v-if="s.divide" class="divide" v-click="3">÷{{ s.divide }}</span>
      </div>
    </template>

    <!-- final hop to the output -->
    <div class="hop">
      <svg class="chev" v-click="1" viewBox="0 0 12 30" aria-hidden="true">
        <line x1="6" y1="0" x2="6" y2="20" />
        <path d="M1.5 18 L6 28 L10.5 18 Z" />
      </svg>
      <span class="rate" v-click="2">{{ rates[steps.length] }}</span>
    </div>

    <div class="endpoint">{{ output }}</div>
  </div>
</template>

<style scoped>
.pmap { display: flex; flex-direction: column; align-items: center; }

.endpoint {
  width: 15em;
  text-align: center;
  padding: 0.4em 1em;
  border: 1px dashed #64748b;
  border-radius: 0.5em;
  color: #cbd5e1;
  opacity: 0.85;
}

.hop {
  display: flex;
  align-items: center;
  gap: 0.7em;
  height: 2.1em;
}
.chev { width: 12px; height: 30px; display: block; }
.chev line { stroke: #64748b; stroke-width: 1.5; }
.chev path { fill: #64748b; }
.rate {
  font-family: var(--slidev-code-font-family, monospace);
  font-size: 1.05em;
  color: #94a3b8;
  font-variant-numeric: tabular-nums;
}

.step {
  display: grid;
  grid-template-columns: auto 8.5em 1fr auto;
  align-items: center;
  gap: 0.9em;
  width: 40em;
  padding: 0.6em 1.1em;
  border: 1.5px solid #818cf8;
  border-radius: 0.6em;
  background: #eef2ff;
  color: #312e81;
}
.badge {
  display: grid;
  place-items: center;
  width: 1.7em;
  height: 1.7em;
  border-radius: 999px;
  background: #818cf8;
  color: #fff;
  font-weight: 700;
  font-size: 0.95em;
}
.name { font-size: 1.25em; font-weight: 700; }
.sub { color: #6366f1; font-size: 0.95em; white-space: nowrap; }
.divide {
  font-family: var(--slidev-code-font-family, monospace);
  font-weight: 700;
  padding: 0.1em 0.5em;
  border-radius: 0.35em;
  background: #c7d2fe;
  color: #3730a3;
}
</style>
