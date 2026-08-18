<script setup>
// The talk's roadmap as the signal path itself, left to right: one chain from
// the air to a demodulator, which then fans out into the three applications the
// same pipeline can feed. It replaces the old four-noun list ("Physics,
// Instrumentation, Interpretation, Application") with the thing those nouns
// describe — a signal becoming, in turn, music, a voice, and an aircraft.
//
// The diagram is schematic, not to scale. Boxes are self-contained light chips
// with dark text, so they stay legible whether the slide renders light or dark.

// The single signal path. Each node is one section of the talk.
const chain = [
  { title: 'Antenna', sub: 'waves → current' },
  { title: 'RTL-SDR', sub: 'digitize spectrum' },
  { title: 'IQ Signals', sub: 'raw samples' },
  { title: 'Demodulation', sub: 'extract the signal' },
]

// Where the one path leads: three demos, each its own colour.
const apps = [
  { title: 'FM Radio', sub: 'music', fill: '#fff7ed', stroke: '#fb923c', ink: '#9a3412' },
  { title: 'AM / Aviation', sub: 'voice', fill: '#eff6ff', stroke: '#60a5fa', ink: '#1e40af' },
  { title: 'Aircraft Tracking', sub: 'ADS-B · 1090 MHz', fill: '#ecfdf5', stroke: '#34d399', ink: '#065f46' },
]

// Geometry (all in viewBox units). The chain is a row of equal boxes at one
// height; the apps are a column to their right, and curved connectors branch
// from the last chain node to each app.
const NW = 168 // chain node width
const NH = 72  // chain node height
const GAP = 46 // arrow gap between chain nodes
const X0 = 8   // first chain node x
const CY = 214 // chain vertical centre

const AW = 196 // app node width
const AH = 64  // app node height
const AX = 930 // app column x — kept well clear of the chain so the branch
               // connectors have room to curve (see viewBox width below)
const APP_CY = [96, 214, 332] // app vertical centres

const chainX = (i) => X0 + i * (NW + GAP)
const lastRight = chainX(chain.length - 1) + NW // right edge of Demodulation

// The hardware/software boundary. Antenna and RTL-SDR are physical; the IQ
// samples and everything after them are just data in your code. The RTL-SDR is
// the last hardware node, so the line sits in the gap after it.
const divX = (chainX(1) + NW + chainX(2)) / 2

// Straight arrows between consecutive chain nodes.
const arrows = chain.slice(1).map((_, i) => ({
  x1: chainX(i) + NW + 6,
  x2: chainX(i + 1) - 6,
}))

// Curved branch connectors from the demod node out to each app.
const branches = APP_CY.map((cy) => {
  const sx = lastRight + 4
  const ex = AX - 6
  const mx = (sx + ex) / 2
  return `M${sx} ${CY} C ${mx} ${CY}, ${mx} ${cy}, ${ex} ${cy}`
})
</script>

<template>
  <svg viewBox="0 0 1140 384" class="coverage-flow" role="img"
       aria-label="Signal path from antenna through the RTL-SDR, IQ signals and demodulation, fanning out into FM, AM aviation, and aircraft tracking">
    <defs>
      <marker id="cf-arrow" viewBox="0 0 10 10" refX="8" refY="5"
              markerWidth="7" markerHeight="7" orient="auto-start-reverse">
        <path d="M0 0 L10 5 L0 10 z" fill="#94a3b8" />
      </marker>
    </defs>

    <!-- Hardware / software boundary: physical front end to the left, code to
         the right. Drawn first so the flow arrow crosses over it. -->
    <g class="cf-divider">
      <line :x1="divX" :y1="24" :x2="divX" :y2="360"
            stroke="#64748b" stroke-width="2" stroke-dasharray="6 7" />
      <text :x="divX - 14" :y="42" text-anchor="end" class="cf-zone">Hardware</text>
      <text :x="divX + 14" :y="42" text-anchor="start" class="cf-zone">Software</text>
    </g>

    <!-- Straight arrows along the chain -->
    <line v-for="(a, i) in arrows" :key="'a' + i"
          :x1="a.x1" :y1="CY" :x2="a.x2" :y2="CY"
          stroke="#94a3b8" stroke-width="2.5" marker-end="url(#cf-arrow)" />

    <!-- Branch connectors into the applications -->
    <path v-for="(d, i) in branches" :key="'b' + i" :d="d"
          fill="none" stroke="#94a3b8" stroke-width="2.5" marker-end="url(#cf-arrow)" />

    <!-- Chain nodes -->
    <g v-for="(n, i) in chain" :key="'n' + i">
      <rect :x="chainX(i)" :y="CY - NH / 2" :width="NW" :height="NH" rx="12"
            fill="#eef2ff" stroke="#818cf8" stroke-width="2" />
      <text :x="chainX(i) + NW / 2" :y="CY - 4" text-anchor="middle"
            class="cf-title" fill="#312e81">{{ n.title }}</text>
      <text :x="chainX(i) + NW / 2" :y="CY + 18" text-anchor="middle"
            class="cf-sub" fill="#6366f1">{{ n.sub }}</text>
    </g>

    <!-- Application nodes -->
    <g v-for="(app, i) in apps" :key="'app' + i">
      <rect :x="AX" :y="APP_CY[i] - AH / 2" :width="AW" :height="AH" rx="12"
            :fill="app.fill" :stroke="app.stroke" stroke-width="2" />
      <text :x="AX + AW / 2" :y="APP_CY[i] - 3" text-anchor="middle"
            class="cf-title" :fill="app.ink">{{ app.title }}</text>
      <text :x="AX + AW / 2" :y="APP_CY[i] + 17" text-anchor="middle"
            class="cf-sub" :fill="app.ink" opacity="0.75">{{ app.sub }}</text>
    </g>
  </svg>
</template>

<style scoped>
.coverage-flow {
  width: 100%;
  height: auto;
  font-family: inherit;
}
/* Sizing lives in CSS, not SVG font-size attributes: the theme's stylesheet
   overrides presentation attributes, so a class (which wins the cascade) is the
   only reliable way to size SVG text here. */
.coverage-flow .cf-title {
  font-size: 18px;
  font-weight: 600;
}
.coverage-flow .cf-sub {
  font-size: 12px;
  font-weight: 400;
}
.coverage-flow .cf-zone {
  font-size: 14px;
  font-weight: 700;
  letter-spacing: 2px;
  text-transform: uppercase;
  fill: #94a3b8;
}
</style>
