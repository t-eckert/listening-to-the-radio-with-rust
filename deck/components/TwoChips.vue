<script setup>
// The RTL-SDR internals as a signal path: antenna in, tuner shifts the chosen
// frequency down to baseband, the ADC samples it to bytes, out over USB to your
// code. Replaces the ASCII box the slide used to draw.
//
// The tuner chip name is a prop because it is the one thing that differs between
// dongles: the RTL-SDR Blog V3 uses an R820T2, the V4 an R828D. The ADC
// (RTL2832U) and the architecture are identical on both.
defineProps({
  tuner: { type: String, default: 'R820T2' },
})

// Geometry in viewBox units. Two chips sit inside the dongle enclosure; the
// antenna feeds the first from outside, USB leaves the last to your code.
const CY = 168 // vertical centre of the signal path
const CHIP_W = 190
const CHIP_H = 78
const TUNER_X = 178
const ADC_X = 486
</script>

<template>
  <svg viewBox="0 0 1000 300" class="two-chips" role="img"
       aria-label="Antenna into the tuner, which shifts to baseband, into the RTL2832U ADC, out over USB to your code">
    <defs>
      <marker id="tc-arrow" viewBox="0 0 10 10" refX="8" refY="5"
              markerWidth="7" markerHeight="7" orient="auto-start-reverse">
        <path d="M0 0 L10 5 L0 10 z" fill="#94a3b8" />
      </marker>
    </defs>

    <!-- dongle enclosure -->
    <rect x="128" y="74" width="612" height="168" rx="16"
          fill="#f8fafc" stroke="#cbd5e1" stroke-width="2" />
    <text x="148" y="100" class="tc-enclosure">RTL-SDR dongle</text>

    <!-- antenna, feeding in from outside -->
    <g stroke="#64748b" stroke-width="2.5" fill="none">
      <line x1="52" y1="196" x2="52" y2="120" />
      <line x1="52" y1="120" x2="38" y2="104" />
      <line x1="52" y1="120" x2="66" y2="104" />
    </g>
    <text x="52" y="216" class="tc-caption">antenna</text>
    <line x1="72" y1="158" x2="176" y2="158" stroke="#94a3b8" stroke-width="2.5"
          marker-end="url(#tc-arrow)" />

    <!-- tuner chip -->
    <rect :x="TUNER_X" :y="CY - CHIP_H / 2" :width="CHIP_W" :height="CHIP_H" rx="12"
          fill="#eef2ff" stroke="#818cf8" stroke-width="2" />
    <text :x="TUNER_X + CHIP_W / 2" y="160" class="tc-title" fill="#312e81">{{ tuner }}</text>
    <text :x="TUNER_X + CHIP_W / 2" y="182" class="tc-role" fill="#6366f1">tuner</text>
    <text :x="TUNER_X + CHIP_W / 2" y="266" class="tc-caption">shift freq → baseband</text>

    <!-- analog hop to the ADC -->
    <line :x1="TUNER_X + CHIP_W" y1="158" :x2="ADC_X - 2" y2="158"
          stroke="#94a3b8" stroke-width="2.5" marker-end="url(#tc-arrow)" />
    <text :x="(TUNER_X + CHIP_W + ADC_X) / 2" y="146" class="tc-wire">analog</text>

    <!-- ADC chip -->
    <rect :x="ADC_X" :y="CY - CHIP_H / 2" :width="CHIP_W" :height="CHIP_H" rx="12"
          fill="#ecfdf5" stroke="#34d399" stroke-width="2" />
    <text :x="ADC_X + CHIP_W / 2" y="160" class="tc-title" fill="#065f46">RTL2832U</text>
    <text :x="ADC_X + CHIP_W / 2" y="182" class="tc-role" fill="#059669">8-bit ADC</text>
    <text :x="ADC_X + CHIP_W / 2" y="266" class="tc-caption">sample → digital bytes</text>

    <!-- USB out to your code -->
    <line :x1="ADC_X + CHIP_W" y1="158" x2="864" y2="158"
          stroke="#94a3b8" stroke-width="2.5" marker-end="url(#tc-arrow)" />
    <text x="792" y="146" class="tc-wire">USB</text>
    <text x="872" y="153" class="tc-code">your</text>
    <text x="872" y="173" class="tc-code">code</text>
  </svg>
</template>

<style scoped>
/* Sizes live in CSS classes, not SVG font-size attributes: the theme stylesheet
   overrides the attributes, so a class is the only reliable way to size text. */
.two-chips {
  width: 100%;
  height: auto;
  font-family: inherit;
}
.two-chips text { text-anchor: middle; }
.two-chips .tc-title { font-size: 20px; font-weight: 600; }
.two-chips .tc-role { font-size: 12px; }
.two-chips .tc-enclosure {
  font-size: 13px;
  font-weight: 600;
  fill: #94a3b8;
  text-anchor: start;
  letter-spacing: 0.04em;
  text-transform: uppercase;
}
.two-chips .tc-caption { font-size: 12px; fill: #64748b; }
.two-chips .tc-wire { font-size: 11px; fill: #94a3b8; letter-spacing: 0.04em; }
.two-chips .tc-code { font-size: 14px; font-weight: 600; fill: #475569; text-anchor: start; }
</style>
