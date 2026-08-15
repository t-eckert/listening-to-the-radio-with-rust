<script setup>
// Vertical flow diagram for a sample-rate chain.
//
// Pass a flat alternating array. Items with `rate`/`label` render as boxes;
// items with `op` render as the connector between them. `divide` puts the
// division factor on the connector, which is the punchline of the FM slide:
// every division is a whole number.
//
//   :steps="[
//     { rate: '960 kHz', label: 'IQ samples' },
//     { op: 'low-pass, keep every 4th sample', divide: 4 },
//     { rate: '240 kHz', label: 'intermediate' },
//     { op: 'FM demodulate' },
//     { label: 'speakers' },
//   ]"
defineProps({
  steps: { type: Array, required: true },
})
</script>

<template>
  <div class="pipeline">
    <template v-for="(s, i) in steps" :key="i">
      <!-- a rate box -->
      <div v-if="!s.op" class="stage" :class="{ terminal: !s.rate }">
        <span class="rate">{{ s.rate }}</span>
        <span class="label">{{ s.label }}</span>
      </div>

      <!-- the connector between two boxes -->
      <div v-else class="op">
        <div class="stem">
          <svg viewBox="0 0 12 34" aria-hidden="true">
            <line x1="6" y1="0" x2="6" y2="24" />
            <path d="M1.5 22 L6 32 L10.5 22 Z" />
          </svg>
        </div>
        <div class="op-text">
          <span v-if="s.divide" class="divide">÷{{ s.divide }}</span>
          <span>{{ s.op }}</span>
        </div>
      </div>
    </template>
  </div>
</template>

<style scoped>
.pipeline {
  display: flex;
  flex-direction: column;
  align-items: center;
  font-size: 0.92em;
}

.stage {
  display: grid;
  grid-template-columns: 6.5em 1fr;
  align-items: baseline;
  gap: 1.1em;
  width: 22em;
  padding: 0.5em 1.1em;
  border: 1px solid currentColor;
  border-radius: 0.4em;
  background: color-mix(in srgb, currentColor 4%, transparent);
}

/* "speakers" has no rate; it is the end of the chain, not a stage. It keeps the
   same width and column grid as the others so the arrow lands inside it and the
   word lines up with the other labels. */
.stage.terminal {
  border-style: dashed;
  opacity: 0.7;
}

.rate {
  font-family: var(--slidev-code-font-family, monospace);
  font-size: 1.35em;
  font-variant-numeric: tabular-nums;
  text-align: right;
  white-space: nowrap;
}

.label {
  opacity: 0.6;
  font-size: 0.95em;
}

.op {
  display: flex;
  align-items: center;
  gap: 0.9em;
  width: 22em;
  padding-left: 2.6em;
}

.stem svg {
  width: 12px;
  height: 34px;
  display: block;
}

.stem line {
  stroke: currentColor;
  stroke-width: 1.5;
  opacity: 0.5;
}

.stem path {
  fill: currentColor;
  opacity: 0.5;
}

.op-text {
  display: flex;
  align-items: baseline;
  gap: 0.6em;
  opacity: 0.7;
  font-size: 0.9em;
}

/* The whole-number divisions are the design. Make them the thing you see. */
.divide {
  font-family: var(--slidev-code-font-family, monospace);
  font-weight: 600;
  opacity: 1;
  padding: 0.05em 0.45em;
  border-radius: 0.3em;
  background: color-mix(in srgb, currentColor 14%, transparent);
}
</style>
