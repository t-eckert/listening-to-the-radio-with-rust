<script setup>
// What the tuner does, shown instead of said: the antenna sees a whole spectrum
// of signals at once (top). The tuner selects one station and slides the entire
// spectrum down so that station sits at 0 Hz — baseband — where the ADC's narrow
// sampling window catches it (bottom). Everything else slides away with it, which
// is the point: it's a shift, not a filter.
//
// Positions are schematic, not to scale. Colours: the chosen station is orange
// (the deck's FM colour); everything else is muted slate.

const A_BASE = 190 // baseline y of the top (RF) panel
const B_BASE = 430 // baseline y of the bottom (baseband) panel
const X0 = 70
const X1 = 930
const CENTER = 500 // x of 0 Hz in the bottom panel
const SHIFT = CENTER - 250 // the chosen station sits at x=250 up top

// A smooth spectral bump filled down to the baseline.
function bump(cx, w, h, base) {
  const top = base - h
  return `M${cx - w} ${base} C ${cx - w * 0.45} ${base} ${cx - w * 0.45} ${top} ${cx} ${top}`
       + ` C ${cx + w * 0.45} ${top} ${cx + w * 0.45} ${base} ${cx + w} ${base} Z`
}

// Top panel: the crowded RF spectrum. One station (sel) is the one we want.
const rf = [
  { cx: 120, w: 24, h: 32 },
  { cx: 208, w: 22, h: 42 },
  { cx: 250, w: 30, h: 98, sel: true },
  { cx: 294, w: 22, h: 40 },
  { cx: 380, w: 24, h: 46 },
  { cx: 560, w: 28, h: 60 },
  { cx: 690, w: 28, h: 54 },
  { cx: 862, w: 36, h: 68 },
]
const chosen = rf.find((s) => s.sel)

// Bottom panel: the same spectrum shifted by SHIFT. The chosen station lands on
// 0; a few neighbours become faint ghosts that have slid outside the ADC window.
const ghosts = [
  { cx: 120 + SHIFT, w: 24, h: 32 },
  { cx: 380 + SHIFT, w: 24, h: 46 },
  { cx: 560 + SHIFT, w: 28, h: 60 },
]
const WIN = 66 // half-width of the ADC sampling window around 0
</script>

<template>
  <svg viewBox="0 0 1000 476" class="tuner-shift" role="img"
       aria-label="The antenna receives a full spectrum of signals; the tuner shifts the chosen station down to zero hertz, baseband, where the ADC sampling window captures it.">
    <defs>
      <marker id="ts-axis" viewBox="0 0 10 10" refX="8" refY="5"
              markerWidth="6" markerHeight="6" orient="auto-start-reverse">
        <path d="M0 0 L10 5 L0 10 z" fill="#64748b" />
      </marker>
    </defs>

    <!-- ================= TOP: what the antenna receives ================= -->
    <text :x="X0" y="42" class="ts-kicker">ANTENNA</text>

    <!-- non-selected signals -->
    <path v-for="(s, i) in rf.filter(x => !x.sel)" :key="'rf' + i"
          :d="bump(s.cx, s.w, s.h, A_BASE)" class="ts-signal" />
    <!-- the chosen station -->
    <path :d="bump(chosen.cx, chosen.w, chosen.h, A_BASE)" class="ts-chosen" />

    <!-- selection window around the chosen station -->
    <rect :x="chosen.cx - 30" :y="A_BASE - chosen.h - 20" width="60"
          :height="chosen.h + 26" rx="8" class="ts-select" />
    <text :x="chosen.cx" :y="A_BASE - chosen.h - 30" class="ts-tag">your station</text>

    <!-- RF baseline -->
    <line :x1="X0" :y1="A_BASE" :x2="X1" :y2="A_BASE" class="ts-base"
          marker-end="url(#ts-axis)" />
    <text :x="X1" :y="A_BASE + 22" class="ts-axis" text-anchor="end">frequency</text>

    <!-- ================= the shift ================= -->
    <text :x="CENTER" :y="B_BASE - 174" class="ts-move-label">R820T2 + LO</text>
    <text :x="CENTER" :y="B_BASE - 155" class="ts-move-label ts-move-sub">shifts to 0</text>

    <!-- ================= BOTTOM: what the ADC samples ================= -->
    <text :x="X0" :y="B_BASE - 150" class="ts-kicker">BASEBAND</text>

    <!-- ADC sampling window -->
    <rect :x="CENTER - WIN" :y="B_BASE - 130" :width="WIN * 2" height="130" rx="8"
          class="ts-window" />
    <text :x="CENTER + WIN + 14" :y="B_BASE - 74" class="ts-tag ts-win-tag">sampled</text>

    <!-- signals that slid out of the window -->
    <path v-for="(g, i) in ghosts" :key="'g' + i"
          :d="bump(g.cx, g.w, g.h, B_BASE)" class="ts-ghost" />
    <!-- the chosen station, now centred on 0 -->
    <path :d="bump(CENTER, chosen.w, chosen.h, B_BASE)" class="ts-chosen" />

    <!-- baseband baseline with 0 marked -->
    <line :x1="X0" :y1="B_BASE" :x2="X1" :y2="B_BASE" class="ts-base"
          marker-end="url(#ts-axis)" />
    <line :x1="CENTER" :y1="B_BASE - 4" :x2="CENTER" :y2="B_BASE + 8" class="ts-tick" />
    <text :x="CENTER" :y="B_BASE + 30" class="ts-zero">0</text>
    <text :x="X1" :y="B_BASE + 22" class="ts-axis" text-anchor="end">frequency</text>
  </svg>
</template>

<style scoped>
.tuner-shift { width: 100%; height: auto; font-family: inherit; }

.tuner-shift .ts-signal { fill: #64748b; opacity: 0.55; }
.tuner-shift .ts-ghost  { fill: #64748b; opacity: 0.22; }
.tuner-shift .ts-chosen { fill: #fb923c; }

.tuner-shift .ts-base { stroke: #64748b; stroke-width: 2; }
.tuner-shift .ts-tick { stroke: #cbd5e1; stroke-width: 2; }

.tuner-shift .ts-select {
  fill: none;
  stroke: #fdba74;
  stroke-width: 2;
  stroke-dasharray: 5 5;
}
.tuner-shift .ts-window {
  fill: #fb923c;
  fill-opacity: 0.1;
  stroke: #fb923c;
  stroke-opacity: 0.5;
  stroke-width: 1.5;
}

.tuner-shift .ts-kicker {
  fill: #94a3b8;
  font-size: 15px;
  font-weight: 700;
  letter-spacing: 0.12em;
}
.tuner-shift .ts-tag {
  fill: #fdba74;
  font-size: 14px;
  font-weight: 600;
  text-anchor: middle;
}
.tuner-shift .ts-win-tag { fill: #fdba74; opacity: 0.9; text-anchor: start; }
.tuner-shift .ts-move-label {
  fill: #fb923c;
  font-size: 14px;
  font-weight: 600;
  text-anchor: middle;
}
.tuner-shift .ts-move-sub { font-size: 13px; font-weight: 500; opacity: 0.85; }
.tuner-shift .ts-axis { fill: #94a3b8; font-size: 13px; }
.tuner-shift .ts-zero {
  fill: #e2e8f0;
  font-size: 16px;
  font-weight: 700;
  text-anchor: middle;
}
</style>
