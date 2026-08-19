<script setup>
// In-deck version of the `iq-demo` TUI (demos/iq-demo). Left: the signal in the
// time domain, a sine whose height is A*sin(phase). Right: the same sample as a
// point on the complex plane — I = A*cos(phase), Q = A*sin(phase) — rotating on a
// circle of radius A. The whole point: rotation SPEED is frequency, DISTANCE from
// the origin is amplitude. Both white dots are the same instant of the signal.
//
// Controls are buttons + keys that don't collide with Slidev's arrow-key slide
// nav: [ ] change frequency, - = change amplitude.
import { ref, computed, onMounted, onUnmounted } from 'vue'

const HISTORY = 5.0 // seconds shown on the oscilloscope
const FREQ_MIN = 0.1, FREQ_MAX = 2.0, FREQ_STEP = 0.1
const AMP_MIN = 0.1, AMP_MAX = 1.0, AMP_STEP = 0.1

const frequency = ref(0.4)
const amplitude = ref(0.8)
const phase = ref(0) // accumulated, so frequency changes stay continuous
const sweeping = ref(false) // hands-off mode: drive both from a slow clock

// --- geometry (viewBox units) ---
const pxT = (t) => 32 + (t / HISTORY) * 480       // time -> x on the left plot
const pyV = (v) => 228 - v * 130                  // value -> y on the left plot
const RCX = 750, RCY = 228, S = 127               // complex-plane centre + scale
const cxI = (i) => RCX + i * S
const cyQ = (q) => RCY - q * S

const omega = computed(() => 2 * Math.PI * frequency.value)

const wavePoints = computed(() => {
  const A = amplitude.value, w = omega.value, ph = phase.value
  let s = ''
  for (let i = 0; i <= 240; i++) {
    const t = (i / 240) * HISTORY
    const y = A * Math.sin(ph - w * (HISTORY - t))
    s += `${pxT(t).toFixed(1)},${pyV(y).toFixed(1)} `
  }
  return s.trim()
})

const iq = computed(() => {
  const A = amplitude.value, ph = phase.value
  return { x: cxI(A * Math.cos(ph)), y: cyQ(A * Math.sin(ph)) }
})
const waveDotY = computed(() => pyV(amplitude.value * Math.sin(phase.value)))

const arc = (span, steps) => {
  const A = amplitude.value, ph = phase.value
  let s = ''
  for (let k = 0; k <= steps; k++) {
    const a = ph - (k / steps) * span
    s += `${cxI(A * Math.cos(a)).toFixed(1)},${cyQ(A * Math.sin(a)).toFixed(1)} `
  }
  return s.trim()
}
const trail = computed(() => arc(1.7 * Math.PI, 44)) // faint tail, most of a turn
const trailHead = computed(() => arc(0.5 * Math.PI, 14)) // brighter near the dot

// --- animation ---
let raf, last, sweepT = 0
function tick(now) {
  const t = now / 1000
  if (last === undefined) last = t
  const dt = t - last
  last = t
  if (sweeping.value) {
    // Two slow cosines on different periods so freq and amp drift out of phase,
    // covering the whole range together without a fixed pattern.
    sweepT += dt
    const mix = (period) => 0.5 - 0.5 * Math.cos((2 * Math.PI * sweepT) / period)
    frequency.value = FREQ_MIN + (FREQ_MAX - FREQ_MIN) * mix(9)
    amplitude.value = AMP_MIN + (AMP_MAX - AMP_MIN) * mix(5.5)
  }
  phase.value += 2 * Math.PI * frequency.value * dt
  raf = requestAnimationFrame(tick)
}

// --- controls ---
const round1 = (x) => Math.round(x * 10) / 10
// Any manual adjustment takes back control from the sweep.
function adjFreq(d) { sweeping.value = false; frequency.value = round1(Math.min(FREQ_MAX, Math.max(FREQ_MIN, frequency.value + d))) }
function adjAmp(d) { sweeping.value = false; amplitude.value = round1(Math.min(AMP_MAX, Math.max(AMP_MIN, amplitude.value + d))) }

function onKey(e) {
  const map = { '[': () => adjFreq(-FREQ_STEP), ']': () => adjFreq(FREQ_STEP),
                '-': () => adjAmp(-AMP_STEP), '=': () => adjAmp(AMP_STEP), '+': () => adjAmp(AMP_STEP) }
  if (map[e.key]) { e.preventDefault(); e.stopImmediatePropagation(); map[e.key]() }
}

onMounted(() => { window.addEventListener('keydown', onKey, true); raf = requestAnimationFrame(tick) })
onUnmounted(() => { window.removeEventListener('keydown', onKey, true); cancelAnimationFrame(raf) })
</script>

<template>
  <div class="iq-demo">
    <svg viewBox="0 0 1000 420" class="iq-svg" role="img"
         aria-label="A sine wave in the time domain beside the same sample rotating as a point on the complex plane.">
      <!-- panel titles -->
      <text x="20" y="34" class="iq-h iq-amber">Signal (time domain)</text>
      <text x="560" y="34" class="iq-h iq-cyan">IQ point (complex plane)</text>

      <!-- ===== left: oscilloscope ===== -->
      <rect x="16" y="50" width="512" height="356" rx="10" class="iq-box" />
      <line x1="32" y1="228" x2="512" y2="228" class="iq-axis" />
      <text x="44" y="88" class="iq-tick">+A</text>
      <text x="44" y="372" class="iq-tick">−A</text>
      <polyline :points="wavePoints" class="iq-wave" />
      <circle :cx="512" :cy="waveDotY" r="6" class="iq-dot" />

      <!-- ===== right: complex plane ===== -->
      <rect x="560" y="50" width="380" height="356" rx="10" class="iq-box" />
      <line x1="574" y1="228" x2="926" y2="228" class="iq-axis" />
      <line x1="750" y1="64" x2="750" y2="392" class="iq-axis" />
      <text x="918" y="220" class="iq-tick">I</text>
      <text x="760" y="80" class="iq-tick">Q</text>
      <circle :cx="RCX" :cy="RCY" :r="amplitude * S" class="iq-orbit" />
      <polyline :points="trail" class="iq-trail" />
      <polyline :points="trailHead" class="iq-trail-head" />
      <line :x1="RCX" :y1="RCY" :x2="iq.x" :y2="iq.y" class="iq-phasor" />
      <circle :cx="iq.x" :cy="iq.y" r="6" class="iq-dot" />
    </svg>

    <div class="iq-ctrl">
      <div class="grp">
        <span class="lbl iq-amber">frequency</span>
        <button @click="adjFreq(-FREQ_STEP)" aria-label="lower frequency">−</button>
        <span class="val">{{ frequency.toFixed(1) }} Hz</span>
        <button @click="adjFreq(FREQ_STEP)" aria-label="raise frequency">+</button>
      </div>
      <div class="grp">
        <span class="lbl iq-cyan">amplitude</span>
        <button @click="adjAmp(-AMP_STEP)" aria-label="lower amplitude">−</button>
        <span class="val">{{ amplitude.toFixed(1) }}</span>
        <button @click="adjAmp(AMP_STEP)" aria-label="raise amplitude">+</button>
      </div>
      <button class="sweep" :class="{ active: sweeping }" @click="sweeping = !sweeping"
              :aria-pressed="sweeping">
        {{ sweeping ? '■ stop sweep' : '▸ auto-sweep' }}
      </button>
      <div class="hint"><kbd>[</kbd> <kbd>]</kbd> frequency &nbsp;·&nbsp; <kbd>−</kbd> <kbd>=</kbd> amplitude</div>
    </div>
  </div>
</template>

<style scoped>
.iq-demo { display: flex; flex-direction: column; align-items: center; gap: 1.25rem; }
.iq-svg { width: 100%; height: auto; font-family: inherit; }

.iq-amber { fill: #fbbf24; color: #fbbf24; }
.iq-cyan { fill: #22d3ee; color: #22d3ee; }

.iq-h { font-size: 17px; font-weight: 600; letter-spacing: 0.02em; }
.iq-box { fill: #0f172a; fill-opacity: 0.4; stroke: #334155; stroke-width: 1.5; }
.iq-axis { stroke: #334155; stroke-width: 1.5; }
.iq-tick { fill: #64748b; font-size: 14px; }

.iq-wave { fill: none; stroke: #fbbf24; stroke-width: 2.5; stroke-linejoin: round; }
.iq-orbit { fill: none; stroke: #334155; stroke-width: 1.5; stroke-dasharray: 4 5; }
.iq-trail { fill: none; stroke: #0e7490; stroke-width: 2.5; opacity: 0.6; stroke-linecap: round; }
.iq-trail-head { fill: none; stroke: #22d3ee; stroke-width: 2.5; opacity: 0.9; stroke-linecap: round; }
.iq-phasor { stroke: #22d3ee; stroke-width: 2.5; }
.iq-dot { fill: #f8fafc; stroke: #0f172a; stroke-width: 1; }

.iq-ctrl {
  display: flex;
  align-items: center;
  gap: 1.75rem;
  font-size: 1.05rem;
}
.iq-ctrl .grp { display: flex; align-items: center; gap: 0.6rem; }
.iq-ctrl .lbl { font-weight: 600; }
.iq-ctrl .val { min-width: 4.2rem; text-align: center; color: #e2e8f0; font-variant-numeric: tabular-nums; }
.iq-ctrl button {
  width: 2rem; height: 2rem;
  border: 1px solid #475569;
  border-radius: 8px;
  background: #1e293b;
  color: #e2e8f0;
  font-size: 1.1rem;
  line-height: 1;
  cursor: pointer;
}
.iq-ctrl button:hover { background: #334155; }
.iq-ctrl button.sweep {
  width: auto;
  height: 2rem;
  padding: 0 0.9rem;
  font-size: 0.95rem;
  color: #cbd5e1;
  white-space: nowrap;
}
.iq-ctrl button.sweep.active {
  border-color: #22d3ee;
  background: rgba(34, 211, 238, 0.15);
  color: #67e8f9;
}
.iq-ctrl .hint { color: #64748b; font-size: 0.85rem; white-space: nowrap; }
.iq-ctrl kbd {
  border: 1px solid #475569;
  border-radius: 5px;
  padding: 0 0.35rem;
  background: #1e293b;
  color: #cbd5e1;
  font-size: 0.8rem;
}
</style>
