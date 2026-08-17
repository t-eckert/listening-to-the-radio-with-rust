<script setup>
// Two sources of IQ bytes converging into one consumer: rtl_sdr over a local
// pipe, rtl_tcp over a network socket, both feeding the SAME "your code" box.
// The convergence is the whole point of the slide — the code can't tell the two
// transports apart — so the two arrows must meet at one box, not run as parallel
// pipelines.
const SRC_X = 214
const SRC_W = 176
const SRC_H = 62
const TOP_CY = 88   // rtl_sdr row
const BOT_CY = 196  // rtl_tcp row
const CODE_X = 668
const CODE_CY = 142 // your code sits between the two sources

// Fan-IN: a curve from each source's right edge to the shared box's left-centre.
const branch = (cy) => {
  const sx = SRC_X + SRC_W
  const ex = CODE_X - 4
  const mx = (sx + ex) / 2
  return `M${sx} ${cy} C ${mx} ${cy}, ${mx} ${CODE_CY}, ${ex} ${CODE_CY}`
}
</script>

<template>
  <svg viewBox="0 0 1000 284" class="transport" role="img"
       aria-label="rtl_sdr over a pipe and rtl_tcp over a socket both converge into one 'your code' box">
    <defs>
      <marker id="tr-arrow" viewBox="0 0 10 10" refX="8" refY="5"
              markerWidth="7" markerHeight="7" orient="auto-start-reverse">
        <path d="M0 0 L10 5 L0 10 z" fill="#94a3b8" />
      </marker>
    </defs>

    <!-- converging connectors -->
    <path :d="branch(TOP_CY)" fill="none" stroke="#94a3b8" stroke-width="2.5" marker-end="url(#tr-arrow)" />
    <path :d="branch(BOT_CY)" fill="none" stroke="#94a3b8" stroke-width="2.5" marker-end="url(#tr-arrow)" />
    <text x="512" y="108" class="tr-wire">pipe</text>
    <text x="512" y="188" class="tr-wire">socket</text>

    <!-- source 1: local pipe -->
    <text x="196" :y="TOP_CY - 6" class="tr-where">this laptop</text>
    <rect :x="SRC_X" :y="TOP_CY - SRC_H / 2" :width="SRC_W" :height="SRC_H" rx="12"
          fill="#eef2ff" stroke="#818cf8" stroke-width="2" />
    <text :x="SRC_X + SRC_W / 2" :y="TOP_CY + 6" class="tr-mono" fill="#312e81">rtl_sdr</text>

    <!-- source 2: network socket -->
    <text x="196" :y="BOT_CY - 6" class="tr-where">somewhere else</text>
    <rect :x="SRC_X" :y="BOT_CY - SRC_H / 2" :width="SRC_W" :height="SRC_H" rx="12"
          fill="#eef2ff" stroke="#818cf8" stroke-width="2" />
    <text :x="SRC_X + SRC_W / 2" :y="BOT_CY + 6" class="tr-mono" fill="#312e81">rtl_tcp</text>

    <!-- the one consumer both feed -->
    <rect :x="CODE_X" :y="CODE_CY - 40" width="212" height="80" rx="14"
          fill="#ecfdf5" stroke="#34d399" stroke-width="2.5" />
    <text :x="CODE_X + 106" :y="CODE_CY + 7" class="tr-code" fill="#065f46">your code</text>
  </svg>
</template>

<style scoped>
/* Sizes in CSS classes: the theme stylesheet overrides SVG font-size attributes. */
.transport { width: 100%; height: auto; font-family: inherit; }
.transport text { text-anchor: middle; }
.transport .tr-mono { font-size: 19px; font-weight: 600; font-family: ui-monospace, monospace; }
.transport .tr-code { font-size: 21px; font-weight: 600; }
.transport .tr-where { font-size: 13px; fill: #64748b; text-anchor: end; }
.transport .tr-wire { font-size: 12px; fill: #94a3b8; letter-spacing: 0.04em; }
</style>
