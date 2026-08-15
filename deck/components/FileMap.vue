<script setup>
// A minimap of a source file: bands sized by real line count, so the picture
// argues the point instead of decorating it. The three `step` bands are the
// radio; everything else is scaffolding.
//
// Line counts are from demos/fm-single/src/main.rs and should be re-measured
// if that file changes shape.
defineProps({
  file: { type: String, required: true },
  bands: { type: Array, required: true }, // [{ label, note, lines, step? }]
})

const total = (bands) => bands.reduce((n, b) => n + b.lines, 0)
</script>

<template>
  <div class="filemap-wrap">
    <div class="filename">{{ file }}</div>

    <div
      class="filemap"
      :style="{ gridTemplateRows: bands.map((b) => `${b.lines}fr`).join(' ') }"
    >
      <template v-for="(b, i) in bands" :key="i">
        <div class="bar" :class="{ step: b.step }" />
        <div class="row" :class="{ step: b.step }">
          <span class="label">{{ b.label }}</span>
          <span class="note">{{ b.note }}</span>
          <span class="lines">{{ b.lines }}</span>
        </div>
      </template>
    </div>

    <div class="total">{{ total(bands) }} lines, and the radio is the highlighted part</div>
  </div>
</template>

<style scoped>
.filemap-wrap {
  font-size: 0.9em;
}

.filename {
  font-family: var(--slidev-code-font-family, monospace);
  opacity: 0.55;
  margin-bottom: 0.6em;
}

.filemap {
  display: grid;
  grid-template-columns: 0.75em 1fr;
  column-gap: 1em;
  height: 15.5em;
}

.bar {
  border-radius: 2px;
  background: color-mix(in srgb, currentColor 12%, transparent);
  margin-bottom: 2px;
}

.bar.step {
  background: currentColor;
  opacity: 0.85;
}

.row {
  display: grid;
  grid-template-columns: 12em 1fr 3em;
  align-items: center;
  gap: 0.8em;
  opacity: 0.45;
  border-bottom: 1px solid color-mix(in srgb, currentColor 10%, transparent);
}

.row.step {
  opacity: 1;
}

.label {
  font-family: var(--slidev-code-font-family, monospace);
  white-space: nowrap;
}

.note {
  font-size: 0.88em;
  opacity: 0.7;
}

.lines {
  font-family: var(--slidev-code-font-family, monospace);
  font-size: 0.85em;
  text-align: right;
  font-variant-numeric: tabular-nums;
  opacity: 0.6;
}

.total {
  margin-top: 0.8em;
  opacity: 0.55;
  font-size: 0.9em;
}
</style>
