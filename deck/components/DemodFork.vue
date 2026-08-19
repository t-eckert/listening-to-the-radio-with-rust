<script setup>
// The demodulation fork, told like the right side of the CoverageFlow diagram:
// one IQ point forks into the two questions every demodulator asks — how far
// from the origin (magnitude → amplitude modulation) or how fast it's turning
// (phase change → frequency modulation) — and each fans out to its applications.
// Blue is the amplitude family, orange the frequency family, matching the deck.

const PW = 205, PH = 42 // application pill size
const bluePills = [
  { t: 'AM broadcast', x: 660, y: 87 },
  { t: 'ADS-B', x: 880, y: 87 },
  { t: 'garage remotes', x: 660, y: 131 },
  { t: 'weather satellites', x: 880, y: 131 },
]
const orangePills = [
  { t: 'FM broadcast', x: 660, y: 307 },
  { t: 'pagers', x: 880, y: 307 },
  { t: 'ship AIS', x: 660, y: 351 },
  { t: 'digital voice', x: 880, y: 351 },
]
</script>

<template>
  <svg viewBox="0 0 1120 470" class="fork" role="img"
       aria-label="The IQ point forks into two questions: how far from the origin (magnitude, amplitude modulation: AM broadcast, ADS-B, garage remotes, weather satellites) and how fast it is rotating (phase, frequency modulation: FM broadcast, pagers, ship AIS, digital voice).">
    <defs>
      <marker id="fk-blue" viewBox="0 0 10 10" refX="8" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse">
        <path d="M0 0 L10 5 L0 10 z" fill="#60a5fa" />
      </marker>
      <marker id="fk-orange" viewBox="0 0 10 10" refX="8" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse">
        <path d="M0 0 L10 5 L0 10 z" fill="#fb923c" />
      </marker>
    </defs>

    <!-- the fork: one IQ point splitting into two branches -->
    <path d="M160 240 C 226 240, 226 130, 286 130" fill="none" stroke="#60a5fa" stroke-width="3.5" marker-end="url(#fk-blue)" />
    <path d="M160 240 C 226 240, 226 350, 286 350" fill="none" stroke="#fb923c" stroke-width="3.5" marker-end="url(#fk-orange)" />

    <!-- origin -->
    <rect x="20" y="205" width="140" height="70" rx="12" fill="#eef2ff" stroke="#818cf8" stroke-width="2" />
    <text x="90" y="246" class="fk-origin">IQ point</text>

    <!-- ===== amplitude branch (blue) ===== -->
    <rect x="290" y="70" width="330" height="120" rx="14" fill="#eff6ff" stroke="#60a5fa" stroke-width="2" />
    <text x="455" y="98" class="fk-eyebrow" fill="#2563eb">AMPLITUDE MODULATION</text>
    <text x="455" y="130" class="fk-q" fill="#1e3a8a">How far from the origin?</text>
    <text x="455" y="162" class="fk-formula" fill="#1e40af">magnitude = √(I² + Q²)</text>
    <line x1="620" y1="130" x2="656" y2="130" stroke="#60a5fa" stroke-width="2.5" marker-end="url(#fk-blue)" />
    <g v-for="p in bluePills" :key="p.t">
      <rect :x="p.x" :y="p.y" :width="PW" :height="PH" rx="10" fill="#eff6ff" stroke="#93c5fd" stroke-width="1.5" />
      <text :x="p.x + PW / 2" :y="p.y + 27" class="fk-pill" fill="#1e40af">{{ p.t }}</text>
    </g>

    <!-- ===== frequency branch (orange) ===== -->
    <rect x="290" y="290" width="330" height="120" rx="14" fill="#fff7ed" stroke="#fb923c" stroke-width="2" />
    <text x="455" y="318" class="fk-eyebrow" fill="#ea580c">FREQUENCY MODULATION</text>
    <text x="455" y="350" class="fk-q" fill="#9a3412">How fast is it rotating?</text>
    <text x="455" y="382" class="fk-formula" fill="#9a3412">phase = Δφ since last sample</text>
    <line x1="620" y1="350" x2="656" y2="350" stroke="#fb923c" stroke-width="2.5" marker-end="url(#fk-orange)" />
    <g v-for="p in orangePills" :key="p.t">
      <rect :x="p.x" :y="p.y" :width="PW" :height="PH" rx="10" fill="#fff7ed" stroke="#fdba74" stroke-width="1.5" />
      <text :x="p.x + PW / 2" :y="p.y + 27" class="fk-pill" fill="#9a3412">{{ p.t }}</text>
    </g>
  </svg>
</template>

<style scoped>
.fork { width: 100%; height: auto; font-family: inherit; }
.fork text { text-anchor: middle; }
.fork .fk-origin { font-size: 20px; font-weight: 600; fill: #312e81; }
.fork .fk-eyebrow { font-size: 12px; font-weight: 700; letter-spacing: 0.1em; }
.fork .fk-q { font-size: 21px; font-weight: 700; }
.fork .fk-formula { font-size: 15px; font-family: ui-monospace, monospace; }
.fork .fk-pill { font-size: 15px; font-weight: 500; }
</style>
