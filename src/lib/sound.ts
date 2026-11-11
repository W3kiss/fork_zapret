// Subtle UI sounds synthesised with the Web Audio API (no audio files needed).

let ctx: AudioContext | null = null;

function audio(): AudioContext | null {
  try {
    ctx ??= new AudioContext();
    if (ctx.state === 'suspended') void ctx.resume();
    return ctx;
  } catch {
    return null;
  }
}

function tone(freq: number, start: number, dur: number, gain = 0.08, type: OscillatorType = 'sine') {
  const a = audio();
  if (!a) return;
  const t0 = a.currentTime + start;
  const osc = a.createOscillator();
  const g = a.createGain();
  osc.type = type;
  osc.frequency.setValueAtTime(freq, t0);
  g.gain.setValueAtTime(0, t0);
  g.gain.linearRampToValueAtTime(gain, t0 + 0.015);
  g.gain.exponentialRampToValueAtTime(0.0001, t0 + dur);
  osc.connect(g).connect(a.destination);
  osc.start(t0);
  osc.stop(t0 + dur + 0.05);
}

export const sounds = {
  connect() {
    tone(523.25, 0, 0.35);
    tone(659.25, 0.09, 0.35);
    tone(783.99, 0.18, 0.5);
  },
  disconnect() {
    tone(659.25, 0, 0.3, 0.06);
    tone(440, 0.1, 0.45, 0.06);
  },
  found() {
    [783.99, 987.77, 1174.66, 1567.98].forEach((f, i) => tone(f, i * 0.07, 0.4, 0.05, 'triangle'));
  },
  error() {
    tone(220, 0, 0.25, 0.06, 'triangle');
    tone(185, 0.12, 0.3, 0.06, 'triangle');
  },
  tick() {
    tone(1400, 0, 0.06, 0.02, 'sine');
  }
};
