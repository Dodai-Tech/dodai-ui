// The page's script: sends actions to the server, swaps in the page it answers with, keeps the
// scroll where it was, follows the scroll in the sidebar and moves what drags (knobs, gizmos).
(() => {
  'use strict';
  const SLACK = 60; // px below the page's top a section still counts as reached
  let busy = false;
  let queued = null; // the newest action that came while one was in flight
  let pinned = null; // the section a sidebar pick holds the highlight on, until the user scrolls
  let current = null; // the section the sidebar highlights
  let redraw = 0;

  const $ = (s) => document.querySelector(s);
  const keep = ['.dodai-page', '.dodai-side__nav'];

  // Run `fn` and put the scroll positions and the typing focus back afterwards.
  function preserving(fn, only = keep) {
    const tops = only.map((s) => ($(s) ? $(s).scrollTop : 0));
    const scrolls = only === keep ? [...document.querySelectorAll('.dodai-scroll')].map((e) => e.scrollTop) : [];
    const f = document.activeElement;
    const id = f && f.dataset ? f.dataset.id : null;
    const sel = id && f.selectionStart != null ? [f.selectionStart, f.selectionEnd] : null;
    fn();
    only.forEach((s, i) => { if ($(s)) $(s).scrollTop = tops[i]; });
    document.querySelectorAll('.dodai-scroll').forEach((e, i) => { if (scrolls[i] != null) e.scrollTop = scrolls[i]; });
    if (id) {
      const n = document.querySelector('[data-id="' + id + '"]');
      if (n) { n.focus(); if (sel) n.setSelectionRange(sel[0], sel[1]); }
    }
  }

  function swapApp(html) {
    preserving(() => {
      const t = document.createElement('template');
      t.innerHTML = html;
      const next = t.content.firstElementChild;
      $('#app').replaceWith(next);
      document.documentElement.dataset.theme = next.dataset.theme;
    });
    after();
  }

  function swapSide(html) {
    preserving(() => {
      const t = document.createElement('template');
      t.innerHTML = html;
      $('#side').replaceWith(t.content.firstElementChild);
    }, ['.dodai-side__nav']);
  }

  // What follows a new page: a picked section is scrolled to, the clock runs again.
  function after() {
    const app = $('#app');
    const jump = app.dataset.jump;
    if (jump) {
      pinned = jump;
      current = jump;
      const el = document.getElementById('sec-' + jump);
      // A hidden tab runs no animations: jump at once there.
      if (el) el.scrollIntoView({ behavior: document.visibilityState === 'visible' ? 'smooth' : 'auto', block: 'start' });
    } else {
      pinned = null;
      current = activeSub();
    }
    clearTimeout(redraw);
    if (app.dataset.redraw) redraw = setTimeout(refresh, Math.max(100, +app.dataset.redraw));
  }

  function activeSub() {
    const on = document.querySelector('[data-sub].dodai-kit--selected, [data-sub][aria-selected=true]');
    return on ? on.dataset.sub : null;
  }

  async function refresh() {
    if (busy) { redraw = setTimeout(refresh, 200); return; }
    const r = await fetch('/view');
    swapApp(await r.text());
  }

  async function send(action, part) {
    if (busy) { queued = [action, part]; return; }
    busy = true;
    try {
      const r = await fetch('/act' + (part ? '?part=' + part : ''), { method: 'POST', body: action });
      const html = await r.text();
      if (part === 'side') swapSide(html); else swapApp(html);
    } finally {
      busy = false;
      if (queued) { const q = queued; queued = null; send(q[0], q[1]); }
    }
  }

  function value(el) {
    if (el.type === 'checkbox') return el.checked ? 'true' : 'false';
    return el.value;
  }

  function fire(type, e) {
    const el = e.target.closest('[data-on~="' + type + '"]');
    if (!el || !el.dataset.a) return;
    // A press inside a popup panel belongs to the panel, not to the element wrapping it.
    const stop = e.target.closest('[data-stop]');
    if (stop && el.contains(stop) && el !== stop) return;
    if (type === 'click' && el.tagName === 'A') e.preventDefault();
    send(el.dataset.a.replace('{v}', () => value(el)));
  }

  for (const type of ['click', 'change', 'input']) {
    document.addEventListener(type, (e) => {
      // Sliders and text send while they move; the rest on change.
      if (type === 'change' && e.target.closest('[data-on="input"]')) return;
      fire(type, e);
    });
  }

  // --- the scroll-spy: the same rule as the other toolkits (dodai_core::spy::pick) ---
  function spy() {
    const page = $('.dodai-page');
    if (!page) return;
    const top = page.getBoundingClientRect().top;
    const tops = [...document.querySelectorAll('[data-sec]')].map((e) => [e.dataset.sec, e.getBoundingClientRect().top - top]);
    if (!tops.length) return;
    const range = page.scrollHeight - page.clientHeight;
    const atEnd = range > 1 && page.scrollTop >= range - 1;
    let pick;
    if (pinned && tops.some((t) => t[0] === pinned)) pick = pinned;
    else if (atEnd) pick = tops[tops.length - 1][0];
    else {
      pick = tops[0][0];
      for (const t of tops) if (t[1] <= SLACK) pick = t[0];
    }
    if (pick !== current) { current = pick; send('V|' + pick, 'side'); }
  }
  let spying = 0;
  document.addEventListener('scroll', () => {
    if (spying) return;
    // A timer, not a frame: a page in a background tab gets no frames.
    spying = setTimeout(() => { spying = 0; spy(); }, 40);
  }, true);
  for (const type of ['wheel', 'touchmove', 'keydown']) {
    document.addEventListener(type, () => { pinned = null; }, { capture: true, passive: true });
  }

  // --- keys ---
  document.addEventListener('keydown', (e) => {
    if (/^(INPUT|TEXTAREA|SELECT)$/.test(e.target.tagName) || e.metaKey || e.ctrlKey || e.altKey) return;
    if (e.key === 't' || e.key === 'T') send('CT|' + (e.key === 't'));
  });

  // --- knobs: drag up or right to raise ---
  let knob = null;
  document.addEventListener('pointerdown', (e) => {
    const el = e.target.closest('[data-knob]');
    if (el && !el.getAttribute('aria-disabled').startsWith('true')) {
      const [key, min, max] = el.dataset.knob.split('|');
      knob = { el, key, min, max, v0: el.dataset.v, x: e.clientX, y: e.clientY };
      el.setPointerCapture(e.pointerId);
    }
    const g = e.target.closest('[data-gizmo]');
    if (g) {
      const [yk, pk, size] = g.dataset.gizmo.split('|');
      const r = g.getBoundingClientRect();
      gizmo = { el: g, yk, pk, size, yaw: g.dataset.yaw, pitch: g.dataset.pitch, x: e.clientX, y: e.clientY, ox: e.clientX - r.left, oy: e.clientY - r.top, drag: false };
      g.setPointerCapture(e.pointerId);
    }
  });
  let gizmo = null;
  document.addEventListener('pointermove', (e) => {
    if (knob) send(['KN', knob.key, knob.min, knob.max, knob.v0, e.clientX - knob.x, e.clientY - knob.y].join('|'));
    if (gizmo) {
      const dx = e.clientX - gizmo.x, dy = e.clientY - gizmo.y;
      if (Math.hypot(dx, dy) > 3) gizmo.drag = true;
      if (gizmo.drag) {
        // The page was redrawn under the pointer: read the camera it shows now.
        const now = document.querySelector('[data-gizmo^="' + gizmo.yk + '|"]');
        const yaw = now ? now.dataset.yaw : gizmo.yaw, pitch = now ? now.dataset.pitch : gizmo.pitch;
        send(['GO', gizmo.yk, gizmo.pk, yaw, pitch, dx, dy].join('|'));
        gizmo.x = e.clientX; gizmo.y = e.clientY;
      }
    }
  });
  document.addEventListener('pointerup', () => {
    if (gizmo && !gizmo.drag) send(['GC', gizmo.yk, gizmo.pk, gizmo.size, gizmo.yaw, gizmo.pitch, gizmo.ox, gizmo.oy].join('|'));
    knob = null; gizmo = null;
  });
  document.addEventListener('wheel', (e) => {
    const el = e.target.closest('[data-knob]');
    if (!el) return;
    e.preventDefault();
    const [key, min, max] = el.dataset.knob.split('|');
    send(['KN', key, min, max, el.dataset.v, 0, Math.sign(e.deltaY) * 8].join('|'));
  }, { passive: false });

  // --- shaders that move: ask for a new picture a few times a second ---
  setInterval(() => {
    for (const img of document.querySelectorAll('img[data-shader]')) {
      const r = img.getBoundingClientRect();
      if (r.bottom < 0 || r.top > innerHeight) continue;
      const next = new Image();
      next.onload = () => { img.src = next.src; };
      next.src = '/shader?' + img.dataset.q + '&t=' + (Date.now() % 600000);
    }
  }, 120);

  after();
})();
