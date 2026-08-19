<script setup>
// Step 1 is a LOW-PASS filter at baseband. The tuner (previous slide) already slid
// our station (97.7) down to 0 Hz, so it sits at the CENTRE of the sampled band.
// The filter keeps the middle — the passband — and rolls off both shoulders, so
// the neighbouring stations are cut. Here: bright station at centre, a shaded
// passband + filter curve over it, faded neighbours on either side.
//
// The humps and the filter curve are drawn, not measured.
defineProps({
  channel: { type: String, default: '97.7' },
  span: { type: String, default: '960 kHz sampled, your station at the centre' },
  width: { type: String, default: '200 kHz kept' },
})

// Stations across the baseband span. The centre one (0 Hz) is ours and survives;
// the rest sit outside the passband and get removed.
const stations = [
  { x: 130, w: 120, h: 40 },
  { x: 300, w: 150, h: 58 },
  { x: 500, w: 180, h: 92, keep: true }, // ours, at 0 Hz
  { x: 700, w: 150, h: 64 },
  { x: 870, w: 120, h: 46 },
]

// A flat-topped hump with sloped shoulders, drawn from the baseline.
function hump(s, base = 150) {
  const l = s.x - s.w / 2, r = s.x + s.w / 2, top = base - s.h
  return `M${l - 14} ${base} L${l} ${top} L${r} ${top} L${r + 14} ${base} Z`
}

// Low-pass magnitude response: flat over the centre passband, curved skirts down
// to the noise floor. The stroke is the response; the same path closed is the
// shaded "kept" region.
const filterTop = 'M300 150 C348 150,360 46,384 46 L616 46 C640 46,652 150,700 150'
const filterFill = filterTop + ' Z'
</script>

<template>
  <svg class="spectrum" viewBox="0 0 1000 196" role="img" :aria-label="span">
    <text class="span" x="0" y="14">{{ span }}</text>

    <!-- noise floor -->
    <path class="noise" d="M0 150 L60 146 L110 152 L180 145 L240 151 L300 144 L360 150 L420 146 L500 151 L560 145 L620 150 L690 146 L740 152 L800 145 L870 151 L930 146 L1000 150 Z" />
    <line class="base" x1="0" y1="150" x2="1000" y2="150" />

    <!-- 0 Hz (DC) axis -->
    <line class="dc" x1="500" y1="52" x2="500" y2="150" />

    <!-- the low-pass passband and its response curve -->
    <path class="passband" :d="filterFill" />
    <path class="filter" :d="filterTop" />

    <!-- stations: ours bright and kept, the rest faded (cut by the filter) -->
    <path v-for="(s, i) in stations" :key="i" class="station" :class="{ ours: s.keep }" :d="hump(s)" />

    <!-- label our station -->
    <text class="tag" x="500" y="36">{{ channel }}</text>

    <!-- kept-width callout under the passband -->
    <line class="tick" x1="384" y1="162" x2="616" y2="162" />
    <text class="callout" x="500" y="182">{{ width }}</text>
  </svg>
</template>

<style scoped>
.spectrum {
  width: 100%;
  max-width: 620px;
  margin: 0 auto;
  display: block;
}

.noise { fill: currentColor; opacity: 0.1; }
.base { stroke: currentColor; stroke-width: 1; opacity: 0.3; }

.dc { stroke: currentColor; stroke-width: 1; opacity: 0.25; stroke-dasharray: 2 4; }

.passband { fill: #818cf8; opacity: 0.16; }
.filter { fill: none; stroke: #a5b4fc; stroke-width: 2.5; }

.station { fill: currentColor; opacity: 0.16; }
.station.ours { opacity: 0.92; }

.tag, .callout, .span { fill: currentColor; font-family: inherit; text-anchor: middle; }
.tag { font-size: 22px; font-weight: 600; }
.callout { font-size: 16px; opacity: 0.65; }
.span { font-size: 15px; opacity: 0.45; text-anchor: start; }

.tick { stroke: currentColor; stroke-width: 1.5; opacity: 0.5; }
</style>
