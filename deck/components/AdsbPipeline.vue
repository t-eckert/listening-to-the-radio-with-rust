<script setup>
// The ADS-B pipeline, in the same vertical-map idiom as PipelineMap so the
// third receiver visibly rhymes with the first two.
//
// Two things this has to do that a plain list could not:
//
//   1. Separate the FOUR swappable DSP stages from what happens after them.
//     The old code block listed six arrows, which quietly contradicted the
//     line "four stages, and only the first one is radio". Track and HTTP are
//     now visibly downstream of the numbered group rather than part of it.
//
//   2. Make "only the first one is radio" legible BEFORE it is said. Stage 1
//     is sky; stages 2-4 are indigo. The colour split carries the argument,
//     and click 1 names it.
//
// Hop labels say what flows, in prose, deliberately NOT the type names --
// Candidate / RawFrame / Validated are the payoff of "ADS-B: Four Stages"
// later, and spending them here would flatten that reveal.
</script>

<template>
  <div class="apipe">
    <div class="endpoint">IQ samples &middot; 2.4 MS/s</div>

    <div class="hop">
      <svg class="chev" viewBox="0 0 12 20" aria-hidden="true">
        <line x1="6" y1="0" x2="6" y2="12" />
        <path d="M1.5 10 L6 18 L10.5 10 Z" />
      </svg>
      <span class="carry">magnitude samples</span>
    </div>

    <div class="row">
      <div class="step radio">
        <span class="badge">1</span>
        <span class="name">magnitude</span>
        <span class="sub">sqrt(I&sup2; + Q&sup2;), same as AM</span>
      </div>
      <span class="tag radio-tag" v-click="1">radio</span>
    </div>

    <div class="hop">
      <svg class="chev" viewBox="0 0 12 20" aria-hidden="true">
        <line x1="6" y1="0" x2="6" y2="12" />
        <path d="M1.5 10 L6 18 L10.5 10 Z" />
      </svg>
      <span class="carry">a possible message start</span>
    </div>

    <div class="row">
      <div class="step bits">
        <span class="badge">2</span>
        <span class="name">preamble detect</span>
        <span class="sub">find the 8 &micro;s signature</span>
      </div>
      <span class="tag bits-tag" v-click="1">bit&#8209;twiddling</span>
    </div>

    <div class="hop">
      <svg class="chev" viewBox="0 0 12 20" aria-hidden="true">
        <line x1="6" y1="0" x2="6" y2="12" />
        <path d="M1.5 10 L6 18 L10.5 10 Z" />
      </svg>
      <span class="carry">112 raw bits</span>
    </div>

    <div class="row">
      <div class="step bits">
        <span class="badge">3</span>
        <span class="name">bit slice</span>
        <span class="sub">pulse positions become bits</span>
      </div>
    </div>

    <div class="hop">
      <svg class="chev" viewBox="0 0 12 20" aria-hidden="true">
        <line x1="6" y1="0" x2="6" y2="12" />
        <path d="M1.5 10 L6 18 L10.5 10 Z" />
      </svg>
      <span class="carry">a frame that passed CRC</span>
    </div>

    <div class="row">
      <div class="step bits">
        <span class="badge">4</span>
        <span class="name">CRC-24 validate</span>
        <span class="sub">discard what the air damaged</span>
      </div>
    </div>

    <div class="after">
      <div class="tail">
        <span class="tname">track</span>
        <span class="tsub">pair CPR frames into a position</span>
      </div>
      <div class="tail">
        <span class="tname">HTTP</span>
        <span class="tsub">JSON and an SSE stream</span>
      </div>
    </div>

    <div class="endpoint out">aircraft on the map</div>
  </div>
</template>

<style scoped>
.apipe {
  display: flex;
  flex-direction: column;
  align-items: center;
  font-size: 0.8em;
}

.endpoint {
  width: 15em;
  text-align: center;
  padding: 0.2em 1em;
  border: 1px dashed #64748b;
  border-radius: 0.5em;
  color: #cbd5e1;
  opacity: 0.85;
}
.endpoint.out { border-style: solid; border-color: #94a3b8; }

.hop {
  display: flex;
  align-items: center;
  gap: 0.6em;
  height: 1.15em;
}
.chev { width: 12px; height: 20px; display: block; }
.chev line { stroke: #64748b; stroke-width: 1.5; }
.chev path { fill: #64748b; }
.carry {
  font-size: 0.9em;
  color: #94a3b8;
  font-style: italic;
}

/* the step card, plus room for a tag in the right gutter */
.row {
  display: grid;
  grid-template-columns: 34em 9em;
  align-items: center;
  gap: 0.9em;
}

.step {
  display: grid;
  grid-template-columns: auto 9.5em 1fr;
  align-items: center;
  gap: 0.8em;
  padding: 0.26em 0.9em;
  border-radius: 0.5em;
  border: 1.5px solid;
}
.step.radio { border-color: #38bdf8; background: #e0f2fe; color: #075985; }
.step.bits  { border-color: #818cf8; background: #eef2ff; color: #312e81; }

.badge {
  display: grid;
  place-items: center;
  width: 1.55em;
  height: 1.55em;
  border-radius: 999px;
  color: #fff;
  font-weight: 700;
  font-size: 0.9em;
}
.step.radio .badge { background: #38bdf8; }
.step.bits .badge  { background: #818cf8; }

.name { font-size: 1.15em; font-weight: 700; }
.sub { font-size: 0.92em; white-space: nowrap; }
.step.radio .sub { color: #0369a1; }
.step.bits .sub  { color: #6366f1; }

.tag {
  justify-self: start;
  font-size: 0.82em;
  font-weight: 700;
  letter-spacing: 0.02em;
  padding: 0.12em 0.6em;
  border-radius: 0.35em;
  white-space: nowrap;
}
.radio-tag { background: #38bdf8; color: #082f49; }
.bits-tag  { background: #c7d2fe; color: #3730a3; }

/* everything after the four stages is no longer signal processing */
.after {
  display: flex;
  flex-direction: column;
  gap: 0.1em;
  margin: 0.4em 0 0.3em;
  padding-top: 0.4em;
  border-top: 1px dashed #475569;
  width: 34em;
}
.tail {
  display: grid;
  grid-template-columns: 9.5em 1fr;
  gap: 0.8em;
  padding: 0.02em 0.9em;
  color: #94a3b8;
  font-size: 0.95em;
}
.tname { font-weight: 700; color: #cbd5e1; }
</style>
