<script setup>
// The ADS-B receiver is not on stage: a Raspberry Pi with a 7 cm antenna sits by
// an upstairs window and is reached from the stage laptop over the network. This
// is the payoff of the antenna rule (right antenna, view of the sky) and it
// collects the earlier "socket" callback — the code doesn't care that the dongle
// is in another room.
const LINK_CY = 176 // both boxes share this vertical centre so the link is level
</script>

<template>
  <svg viewBox="0 0 1000 340" class="remote" role="img"
       aria-label="A Raspberry Pi with a 7 cm antenna upstairs by a window, reached from the stage laptop over the network">
    <defs>
      <marker id="rr-arrow" viewBox="0 0 10 10" refX="8" refY="5"
              markerWidth="7" markerHeight="7" orient="auto-start-reverse">
        <path d="M0 0 L10 5 L0 10 z" fill="#94a3b8" />
      </marker>
    </defs>

    <!-- remote unit, upstairs -->
    <text x="240" y="66" class="rr-where">7th floor, by a window</text>
    <rect x="70" y="80" width="340" height="192" rx="14"
          fill="#eef2ff" stroke="#818cf8" stroke-width="2" />
    <text x="240" y="124" class="rr-line" fill="#3730a3">7 cm antenna</text>
    <text x="240" y="160" class="rr-line" fill="#3730a3">RTL-SDR</text>
    <text x="240" y="200" class="rr-strong" fill="#312e81">Raspberry Pi</text>
    <text x="240" y="238" class="rr-line" fill="#3730a3">running <tspan class="rr-mono">skyward</tspan></text>

    <!-- the network link (two-way) -->
    <line x1="414" :y1="LINK_CY" x2="646" :y2="LINK_CY" stroke="#94a3b8" stroke-width="2.5"
          marker-start="url(#rr-arrow)" marker-end="url(#rr-arrow)" />
    <text x="530" y="164" class="rr-wire">network</text>

    <!-- the stage laptop -->
    <text x="790" y="66" class="rr-where">this stage</text>
    <rect x="650" y="126" width="280" height="100" rx="14"
          fill="#fff7ed" stroke="#fb923c" stroke-width="2" />
    <text x="790" y="164" class="rr-strong" fill="#9a3412">laptop</text>
    <text x="790" y="198" class="rr-line" fill="#b45309">browser</text>
  </svg>
</template>

<style scoped>
/* Sizes in CSS classes: the theme stylesheet overrides SVG font-size attributes. */
.remote { width: 100%; height: auto; font-family: inherit; }
.remote text { text-anchor: middle; }
.remote .rr-line { font-size: 18px; }
.remote .rr-strong { font-size: 20px; font-weight: 600; }
.remote .rr-mono { font-size: 18px; font-weight: 600; font-family: ui-monospace, monospace; }
.remote .rr-where { font-size: 13px; fill: #64748b; letter-spacing: 0.03em; }
.remote .rr-wire { font-size: 12px; fill: #94a3b8; letter-spacing: 0.04em; }
</style>
