<script setup>
// The three things we build in this talk, previewed up front as a promise before
// the first demo. Same palette as CoverageFlow's fan-out (FM orange, AM/aviation
// blue, ADS-B green) so the colours carry meaning across the deck. Cards are
// self-contained light chips with dark ink, legible in either theme.
const apps = [
  {
    title: 'FM Radio',
    tag: 'music',
    body: 'Broadcast FM, straight to your speakers.',
    fill: '#fff7ed', stroke: '#fb923c', ink: '#9a3412',
  },
  {
    title: 'AM / Aviation',
    tag: 'voice',
    body: 'Pilots and towers on the aviation band.',
    fill: '#eff6ff', stroke: '#60a5fa', ink: '#1e40af',
  },
  {
    title: 'Aircraft Tracking',
    tag: 'ADS-B · 1090 MHz',
    body: 'Positions of the planes flying overhead.',
    fill: '#ecfdf5', stroke: '#34d399', ink: '#065f46',
  },
]
</script>

<template>
  <div class="three-apps">
    <div v-for="app in apps" :key="app.title" class="app-card"
         :style="{ background: app.fill, borderColor: app.stroke, color: app.ink }">
      <div class="app-title">{{ app.title }}</div>
      <div class="app-tag" :style="{ borderColor: app.stroke }">{{ app.tag }}</div>
      <div class="app-body">{{ app.body }}</div>
    </div>
  </div>
</template>

<style scoped>
.three-apps {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 1.5rem;
}
.app-card {
  border: 2px solid;
  border-radius: 16px;
  padding: 1.75rem 1.5rem;
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}
.app-title {
  font-size: 1.5rem;
  font-weight: 700;
  line-height: 1.1;
}
.app-tag {
  align-self: flex-start;
  border: 1.5px solid;
  border-radius: 999px;
  padding: 0.1rem 0.7rem;
  font-size: 0.8rem;
  font-weight: 600;
  opacity: 0.85;
}
.app-body {
  font-size: 1.05rem;
  line-height: 1.4;
  opacity: 0.9;
}
</style>
