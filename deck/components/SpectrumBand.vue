<script setup>
// "The antenna hears every station at once." A slice of spectrum with several
// stations in it and exactly one boxed: the one we're tuning to in software.
//
// The humps are drawn, not measured. The shape is honest about FM broadcast
// (wide, flat-topped channels on a raised noise floor); the exact peaks are not
// a real capture.
defineProps({
  channel: { type: String, default: '97.7' },
  span: { type: String, default: '960 kHz of spectrum, all at once' },
  width: { type: String, default: '200 kHz, one station' },
})

// x positions of the stations across a 0..1000 viewBox; index 2 is the one we want
const stations = [
  { x: 120, w: 84, h: 52 },
  { x: 268, w: 76, h: 34 },
  { x: 430, w: 96, h: 96 }, // ← ours
  { x: 606, w: 80, h: 44 },
  { x: 762, w: 88, h: 66 },
]
const target = stations[2]

// A flat-topped hump with sloped shoulders, drawn from the baseline.
function hump(s, base = 150) {
  const l = s.x - s.w / 2
  const r = s.x + s.w / 2
  const top = base - s.h
  return `M${l - 14} ${base} L${l} ${top} L${r} ${top} L${r + 14} ${base} Z`
}
</script>

<template>
  <svg class="spectrum" viewBox="0 0 1000 186" role="img" :aria-label="span">
    <!-- caption for the whole picture, so it can't be mistaken for a label on
         the boxed channel -->
    <text class="span" x="0" y="14">{{ span }}</text>
    <!-- noise floor -->
    <path
      class="noise"
      d="M0 150 L60 146 L110 152 L180 145 L240 151 L300 144 L360 150 L420 146 L500 151 L560 145 L620 150 L690 146 L740 152 L800 145 L870 151 L930 146 L1000 150 L1000 150 L0 150 Z"
    />
    <line class="base" x1="0" y1="150" x2="1000" y2="150" />

    <!-- every station the antenna is picking up -->
    <path v-for="(s, i) in stations" :key="i" class="station" :class="{ ours: i === 2 }" :d="hump(s)" />

    <!-- the one we keep -->
    <rect
      class="box"
      :x="target.x - target.w / 2 - 22"
      :y="150 - target.h - 20"
      :width="target.w + 44"
      :height="target.h + 20"
      rx="4"
    />
    <text class="tag" :x="target.x" :y="150 - target.h - 30">{{ channel }}</text>

    <!-- width callout under the boxed channel -->
    <line class="tick" :x1="target.x - target.w / 2 - 22" y1="162" :x2="target.x + target.w / 2 + 22" y2="162" />
    <text class="callout" :x="target.x" y="181">{{ width }}</text>
  </svg>
</template>

<style scoped>
.spectrum {
  width: 100%;
  max-width: 560px;
  margin: 0 auto;
  display: block;
}

.noise {
  fill: currentColor;
  opacity: 0.1;
}

.base {
  stroke: currentColor;
  stroke-width: 1;
  opacity: 0.3;
}

.station {
  fill: currentColor;
  opacity: 0.22;
}

.station.ours {
  opacity: 0.9;
}

.box {
  fill: none;
  stroke: currentColor;
  stroke-width: 2;
  stroke-dasharray: 5 4;
  opacity: 0.85;
}

.tag,
.callout,
.span {
  fill: currentColor;
  font-family: inherit;
  text-anchor: middle;
}

.tag {
  font-size: 22px;
  font-weight: 600;
}

.callout {
  font-size: 16px;
  opacity: 0.65;
}

.span {
  font-size: 15px;
  opacity: 0.45;
  text-anchor: start;
}

.tick {
  stroke: currentColor;
  stroke-width: 1.5;
  opacity: 0.5;
}
</style>
