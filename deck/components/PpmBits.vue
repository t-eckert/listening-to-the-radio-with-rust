<script setup>
// How a 1090 MHz Mode S frame turns time into bits. The signal is on-off keyed:
// the carrier is either on (a pulse) or off. There is no amplitude or phase to
// read — only WHERE in each slot the pulse sits. That is pulse-position
// modulation, and it makes timing the whole game.
//
// The waveform is drawn, not captured. The pulse positions are exact (they are
// the ones the decoder in demos/adsb-decoder/src/demod.rs looks for); the clean
// square edges are a schematic — a real magnitude trace is noisy and rounded.
//
//   preamble: pulses at 0, 1, 3.5, 4.5 µs, each 0.5 µs wide (a fixed pattern)
//   data:     each 1 µs bit is two half-slots; pulse in the first half = 1,
//             pulse in the second half = 0

// Geometry: x maps microseconds to pixels, so the picture is to scale in time.
const LEFT = 18
const PPU = 52 // pixels per microsecond
const SPAN_US = 12 // 8 µs preamble + 4 data bits shown
const BASE = 104 // carrier off (low)
const TOP = 54 // carrier on  (high)

const X = (us) => LEFT + us * PPU

// The four data bits we illustrate: 1 0 1 0.
const DATA = [1, 0, 1, 0]

// Preamble pulse intervals, then one pulse per data bit placed in the half-slot
// its value selects. Data starts at 8 µs; bit i occupies [8+i, 8+i+1].
const pulses = [
  [0, 0.5],
  [1, 1.5],
  [3.5, 4],
  [4.5, 5],
]
DATA.forEach((b, i) => {
  const start = 8 + i + (b === 1 ? 0 : 0.5)
  pulses.push([start, start + 0.5])
})

// One stepped path for the whole on-off-keyed waveform: rise at each pulse
// start, fall at each pulse end, flat at the carrier-off level between them.
const wave = (() => {
  let d = `M${X(0)} ${BASE}`
  for (const [s, e] of pulses) {
    d += ` L${X(s)} ${BASE} L${X(s)} ${TOP} L${X(e)} ${TOP} L${X(e)} ${BASE}`
  }
  return `${d} L${X(SPAN_US)} ${BASE}`
})()

// The two sample points inside each data bit: the middle of each half-slot
// (0.25 µs and 0.75 µs in). The decoder compares these two magnitudes and the
// higher one wins — so one dot sits on the pulse, the other in the silence.
const probes = DATA.map((b, i) => {
  const s = 8 + i
  return {
    early: { x: X(s + 0.25), on: b === 1 },
    late: { x: X(s + 0.75), on: b === 0 },
    value: b,
    center: X(s + 0.5),
  }
})

// Integer-µs slot boundaries across the data region, for the faint grid.
const slotLines = [8, 9, 10, 11, 12]
// Dotted line down the middle of each data bit, splitting its two halves.
const midLines = [8.5, 9.5, 10.5, 11.5]
</script>

<template>
  <svg class="ppm" viewBox="0 0 660 176" role="img"
    aria-label="A Mode S waveform: an 8 microsecond preamble followed by four data bits, each one microsecond wide and split into two half-slots. A pulse in the first half is a one, a pulse in the second half is a zero.">
    <text class="cap" :x="X(0)" y="18">on–off keyed — carrier on = pulse, carrier off = silence</text>

    <!-- data-region grid: slot edges and the half-slot divider -->
    <line v-for="u in slotLines" :key="`s${u}`" class="slot" :x1="X(u)" y1="30" :x2="X(u)" y2="112" />
    <line v-for="u in midLines" :key="`m${u}`" class="mid" :x1="X(u)" y1="54" :x2="X(u)" y2="104" />

    <!-- the on-off-keyed magnitude -->
    <path class="wave" :d="wave" />

    <!-- the two sample points per data bit; the higher one decides the bit -->
    <template v-for="(p, i) in probes" :key="`p${i}`">
      <circle class="probe" :class="{ win: p.early.on }" :cx="p.early.x" :cy="p.early.on ? TOP : BASE" r="4" />
      <circle class="probe" :class="{ win: p.late.on }" :cx="p.late.x" :cy="p.late.on ? TOP : BASE" r="4" />
      <text class="bit" :x="p.center" y="130">{{ p.value }}</text>
    </template>

    <!-- region labels -->
    <line class="brace" :x1="X(0)" y1="118" :x2="X(8)" y2="118" />
    <text class="reg" :x="X(4)" y="150">8 µs preamble — a fixed pattern</text>

    <!-- one-bit width callout under the first data bit -->
    <line class="brace" :x1="X(8)" y1="118" :x2="X(9)" y2="118" />
    <text class="tick" :x="X(8.5)" y="150">1 µs</text>
  </svg>
</template>

<style scoped>
.ppm {
  width: 100%;
  max-width: 660px;
  margin: 0 auto;
  display: block;
}

.wave {
  fill: none;
  stroke: currentColor;
  stroke-width: 2.5;
  opacity: 0.9;
  stroke-linejoin: round;
}

.slot {
  stroke: currentColor;
  stroke-width: 1;
  opacity: 0.12;
}

.mid {
  stroke: currentColor;
  stroke-width: 1;
  stroke-dasharray: 2 3;
  opacity: 0.3;
}

.probe {
  fill: currentColor;
  opacity: 0.18;
}

.probe.win {
  opacity: 0.95;
}

.brace {
  stroke: currentColor;
  stroke-width: 1.5;
  opacity: 0.4;
}

.cap,
.reg,
.tick,
.bit {
  fill: currentColor;
  font-family: inherit;
}

.cap {
  font-size: 14px;
  opacity: 0.5;
  text-anchor: start;
}

.bit {
  font-size: 22px;
  font-weight: 600;
  text-anchor: middle;
}

.reg {
  font-size: 14px;
  opacity: 0.6;
  text-anchor: middle;
}

.tick {
  font-size: 13px;
  opacity: 0.55;
  text-anchor: middle;
}
</style>
