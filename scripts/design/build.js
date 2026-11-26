// Run by `nu scripts/design.nu build` through `openpencil eval` (Figma Plugin API), after the
// token files are imported as variables and `CATALOG` (dist/design/components.json) is prepended.
// Everything here reads variables or the catalog; no value is written by hand. Every design
// system (one variable collection, one mode per theme) gets its own pages, bound to its own
// variables, so switching a mode in the editor recolours that system's components.

const firstPage = figma.currentPage;
const vars = figma.variables.getLocalVariables();
const collections = figma.variables.getLocalVariableCollections();
let collection;
let inBase;
const v = (name) => {
  const found = inBase.find((x) => x.name === name);
  if (!found) throw new Error(`no variable ${name}`);
  return found;
};
const use = (sys) => {
  collection = collections.find((c) => c.name === sys);
  inBase = vars.filter((x) => x.variableCollectionId === collection.id);
};

await figma.loadFontAsync({ family: 'Inter', style: 'Regular' });

function bindFill(node, name) {
  const paint = figma.variables.setBoundVariableForPaint({ type: 'SOLID', color: { r: 0, g: 0, b: 0 } }, 'color', v(name));
  node.fills = [paint];
}
function bindStroke(node, name) {
  const paint = figma.variables.setBoundVariableForPaint({ type: 'SOLID', color: { r: 0, g: 0, b: 0 } }, 'color', v(name));
  node.strokes = [paint];
}
function text(content, size, fillVar) {
  const t = figma.createText();
  t.characters = content;
  t.fontSize = size;
  bindFill(t, fillVar);
  return t;
}

function buildSystem(sys, label) {
  use(sys);
  // Every catalog component must have a frame (or a Behaviour row) in the file: the build fails
  // when one has not, so the spec cannot fall behind the components.
  const covered = new Set();
  const cover = (name) => {
    for (const t of name.split(/[^A-Za-z0-9]+/)) if (t) covered.add(t.toLowerCase());
    covered.add(name.replace(/[^A-Za-z0-9]/g, '').toLowerCase());
  };
  const packName = { cyberpunk: 'stroked', material: 'material', dodai: 'material', neobrutalism: 'bold' }[sys];
  const paintIcon = (n, lined) => {
    if (n.type === 'FRAME' || n.type === 'GROUP') n.fills = [];
    else {
      if (lined) n.fills = [];
      else bindFill(n, 'color/text');
      if (n.strokes && n.strokes.length) bindStroke(n, 'color/text');
    }
    if (n.children) for (const c of n.children) paintIcon(c, lined);
  };
  // The icons are the design's own pack (dodai-icons), painted with a colour variable, so the
  // spec shows what every toolkit draws instead of a font glyph.
  const missingIcons = new Set();
  const ico = (id, size = 16, fill = 'color/text') => {
    const found = ICONS[packName].find((x) => x.id === id);
    if (!found) { missingIcons.add(id); { const e = figma.createEllipse(); e.resize(size, size); bindFill(e, fill); return e; } }
    const n = figma.createNodeFromSvg(found.svg);
    const lined = found.svg.includes('fill="none"');
    paintIcon(n, lined);
    if (lined) { const tint = (m) => { if (m.strokes && m.strokes.length) bindStroke(m, fill); if (m.children) m.children.forEach(tint); }; tint(n); }
    else { const tint = (m) => { if (m.fills && m.fills.length && m.type !== 'FRAME' && m.type !== 'GROUP') bindFill(m, fill); if (m.children) m.children.forEach(tint); }; tint(n); }
    n.rescale(size / n.width);
    return n;
  };
  // --- Foundations: colour roles as swatches bound to the variables.
  const foundations = figma.createPage();
  foundations.name = `${label} / Foundations`;
  figma.currentPage = foundations;
  const roles = inBase.filter((x) => x.name.startsWith('color/') && x.resolvedType === 'COLOR');
  const sheet = figma.createFrame();
  sheet.name = 'Colour roles';
  sheet.layoutMode = 'HORIZONTAL';
  sheet.layoutWrap = 'WRAP';
  sheet.itemSpacing = 12;
  sheet.counterAxisSpacing = 12;
  sheet.paddingLeft = sheet.paddingRight = sheet.paddingTop = sheet.paddingBottom = 24;
  sheet.resize(1200, 100);
  sheet.layoutSizingVertical = 'HUG';
  bindFill(sheet, 'color/background');
  for (const r of roles) {
    const cell = figma.createFrame();
    cell.name = r.name;
    cell.layoutMode = 'VERTICAL';
    cell.itemSpacing = 4;
    cell.fills = [];
    const chip = figma.createRectangle();
    chip.resize(110, 40);
    bindFill(chip, r.name);
    bindStroke(chip, 'color/border');
    cell.appendChild(chip);
    cell.appendChild(text(r.name.replace('color/', ''), 11, 'color/text'));
    sheet.appendChild(cell);
  }
  foundations.appendChild(sheet);

  // --- Button: one component per variant and state, bound to the button/* tokens.
  const buttons = figma.createPage();
  buttons.name = `${label} / Button`;
  figma.currentPage = buttons;
  const parts = new Map(); // "variant-state" -> true
  for (const x of inBase) {
    const m = /^button\/(.+)-(bg|fg|border)$/.exec(x.name);
    if (m) parts.set(m[1], true);
  }
  const states = ['idle', 'hovered', 'pressed', 'focused', 'disabled'];
  const variantNames = [...new Set([...parts.keys()].map((k) => k.replace(/-(idle|hovered|pressed|focused|disabled)$/, '')))];
  const comps = [];
  variantNames.forEach((variant, row) => {
    states.forEach((state, col) => {
      if (!parts.has(`${variant}-${state}`)) return;
      const c = figma.createComponent();
      c.name = `Variant=${variant}, State=${state}`;
      c.layoutMode = 'HORIZONTAL';
      c.layoutSizingHorizontal = 'HUG';
      c.layoutSizingVertical = 'HUG';
      c.primaryAxisAlignItems = 'CENTER';
      c.counterAxisAlignItems = 'CENTER';
      c.paddingLeft = c.paddingRight = 20;
      c.paddingTop = c.paddingBottom = 12;
      c.cornerRadius = 12;
      c.strokeWeight = 1;
      bindFill(c, `button/${variant}-${state}-bg`);
      bindStroke(c, `button/${variant}-${state}-border`);
      c.appendChild(text('Button', 15, `button/${variant}-${state}-fg`));
      c.primaryAxisSizingMode = 'AUTO';
      c.counterAxisSizingMode = 'AUTO';
      buttons.appendChild(c);
      c.x = col * 150;
      c.y = row * 70;
      comps.push(c);
    });
  });
  const set = figma.combineAsVariants(comps, buttons);
  set.name = 'Button';

  // --- Surfaces: one component (set) per Kit family, bound to the kit/* tokens (the data gpui draws from).
  const bind = (node, field, name) => openpencil.bindVariable(node, field, v(name));
  const surfaces = figma.createPage();
  surfaces.name = `${label} / Surfaces`;
  figma.currentPage = surfaces;
  const kits = new Map(); // base -> [variant, kit name]
  for (const x of inBase) {
    const m = /^kit\/(.+)-bg$/.exec(x.name);
    if (!m) continue;
    const parts = m[1].split('-');
    const base = parts[0];
    const rest = parts.slice(1).join('-') || 'default';
    if (!kits.has(base)) kits.set(base, []);
    kits.get(base).push([rest, m[1]]);
  }
  const ROUND = new Set(['avatar', 'dot', 'count', 'step']);
  let surfaceY = 0;
  const surfaceSets = [];
  for (const [base, list] of kits) {
    const made = list.map(([variant, kit], col) => {
      const c = figma.createComponent();
      c.name = list.length > 1 ? `Variant=${variant}` : base;
      c.layoutMode = 'HORIZONTAL';
      c.primaryAxisAlignItems = 'CENTER';
      c.counterAxisAlignItems = 'CENTER';
      c.strokeWeight = 1;
      bindFill(c, `kit/${kit}-bg`);
      bindStroke(c, `kit/${kit}-border`);
      bind(c, 'paddingLeft', `kit/${kit}-pad-x`);
      bind(c, 'paddingRight', `kit/${kit}-pad-x`);
      bind(c, 'paddingTop', `kit/${kit}-pad-y`);
      bind(c, 'paddingBottom', `kit/${kit}-pad-y`);
      if (ROUND.has(base)) c.cornerRadius = 999;
      else bind(c, 'cornerRadius', 'shape/radius');
      c.appendChild(text(base[0].toUpperCase() + base.slice(1) + (variant === 'default' ? '' : ` ${variant}`), 13, `kit/${kit}-fg`));
      c.primaryAxisSizingMode = 'AUTO';
      c.counterAxisSizingMode = 'AUTO';
      surfaces.appendChild(c);
      c.x = col * 200;
      c.y = surfaceY;
      return c;
    });
    surfaceY += 70;
    if (made.length > 1) {
      const set = figma.combineAsVariants(made, surfaces);
      set.name = base[0].toUpperCase() + base.slice(1);
      set.y = surfaceY - 70;
      surfaceSets.push(set);
    }
  }

  // --- Controls: sized from the size/* tokens, coloured from the roles (the gpui control geometry).
  const controls = figma.createPage();
  controls.name = `${label} / Controls`;
  figma.currentPage = controls;
  const mk = (name, w, h, x, y) => {
    const c = figma.createComponent();
    c.name = name;
    c.resize(w, h);
    c.x = x;
    c.y = y;
    c.fills = [];
    controls.appendChild(c);
    return c;
  };
  const num = (name) => v(name).valuesByMode[collection.defaultModeId];
  const sz = (n) => num(`size/${n}`);
  const parts2 = [];
  const row = (name, y, build) => {
    const states = ['off', 'on'];
    const comps = states.map((st, i) => {
      const c = mk(`State=${st}`, 200, 48, i * 220, y);
      build(c, st === 'on');
      return c;
    });
    const set = figma.combineAsVariants(comps, controls);
    set.name = name;
    set.y = y;
    parts2.push(name);
  };
  row('Switch', 0, (c, on) => {
    const w = sz('switch-track-w'), h = sz('switch-track-h');
    const track = figma.createRectangle();
    track.resize(w, h);
    track.cornerRadius = h / 2;
    bindFill(track, on ? 'color/primary' : 'color/surface-container-highest');
    bindStroke(track, 'color/outline');
    const d = on ? sz('switch-handle-on') : sz('switch-handle-off');
    const handle = figma.createEllipse();
    handle.resize(d, d);
    bindFill(handle, on ? 'color/on-primary' : 'color/outline');
    c.appendChild(track);
    c.appendChild(handle);
    handle.x = on ? w - d - (h - d) / 2 : (h - d) / 2;
    handle.y = (h - d) / 2;
  });
  row('Checkbox', 70, (c, on) => {
    const b = figma.createRectangle();
    const n = sz('selection-box');
    b.resize(n, n);
    bind(b, 'cornerRadius', 'shape/radius');
    bindFill(b, on ? 'color/primary' : 'color/surface');
    bindStroke(b, 'color/outline');
    c.appendChild(b);
    if (on) {
      const t = ico('check', n - 4, 'color/on-primary');
      c.appendChild(t);
      t.x = 2;
      t.y = 2;
    }
  });
  row('Radio', 140, (c, on) => {
    const n = sz('selection-box');
    const o = figma.createEllipse();
    o.resize(n, n);
    bindFill(o, 'color/surface');
    bindStroke(o, on ? 'color/primary' : 'color/outline');
    o.strokeWeight = 2;
    c.appendChild(o);
    if (on) {
      const d = figma.createEllipse();
      d.resize(n / 2, n / 2);
      bindFill(d, 'color/primary');
      c.appendChild(d);
      d.x = n / 4;
      d.y = n / 4;
    }
  });
  row('Slider', 210, (c, on) => {
    const th = sz('slider-track'), kn = sz('slider-thumb');
    const track = figma.createRectangle();
    track.resize(160, th);
    track.cornerRadius = th / 2;
    bindFill(track, 'color/surface-container-highest');
    const fill = figma.createRectangle();
    fill.resize(on ? 110 : 40, th);
    fill.cornerRadius = th / 2;
    bindFill(fill, 'color/primary');
    const knob = figma.createEllipse();
    knob.resize(kn, kn);
    bindFill(knob, 'color/primary');
    for (const n of [track, fill, knob]) c.appendChild(n);
    track.y = fill.y = (kn - th) / 2;
    knob.x = (on ? 110 : 40) - kn / 2;
  });
  row('Progress', 280, (c, on) => {
    const th = sz('bar-thickness');
    const track = figma.createRectangle();
    track.resize(160, th);
    track.cornerRadius = th / 2;
    bindFill(track, 'color/surface-container-highest');
    const fill = figma.createRectangle();
    fill.resize(on ? 120 : 50, th);
    fill.cornerRadius = th / 2;
    bindFill(fill, 'color/primary');
    c.appendChild(track);
    c.appendChild(fill);
  });
  row('Input', 350, (c, on) => {
    c.resize(200, num('shape/control-height'));
    c.layoutMode = 'HORIZONTAL';
    c.counterAxisAlignItems = 'CENTER';
    c.paddingLeft = c.paddingRight = 12;
    bind(c, 'cornerRadius', 'shape/radius');
    bindFill(c, 'color/surface');
    bindStroke(c, on ? 'color/primary' : 'color/outline');
    c.strokeWeight = on ? 2 : 1;
    c.appendChild(text(on ? 'Typing' : 'Placeholder', 15, on ? 'color/text' : 'color/text-muted'));
  });
  row('Tab', 420, (c, on) => {
    c.layoutMode = 'VERTICAL';
    c.counterAxisAlignItems = 'CENTER';
    c.resize(120, 48);
    c.paddingTop = 12;
    c.itemSpacing = 8;
    c.appendChild(text('Tab', 15, on ? 'color/primary' : 'color/text-muted'));
    const ind = figma.createRectangle();
    ind.resize(48, 3);
    bindFill(ind, on ? 'color/primary' : 'color/border');
    c.appendChild(ind);
  });

  // --- Feedback and data: Badge, Alert, Chip, the same recipes as gpui (`badge`, `alert`, `chip`
  // in dodai-core `component.rs`): solid tone fill with a contrast text, tint at 12% with a tone
  // outline, selection fill with a primary outline.
  const feedback = figma.createPage();
  feedback.name = `${label} / Feedback`;
  figma.currentPage = feedback;
  const ground = figma.createRectangle();
  ground.name = 'Ground';
  ground.resize(640, 1020);
  ground.x = -60;
  ground.y = -40;
  bindFill(ground, 'color/background');
  feedback.appendChild(ground);
  const lum = (c) => 0.2126 * c.r + 0.7152 * c.g + 0.0722 * c.b;
  const toneRoles = [['info', 'Info'], ['success', 'Success'], ['warning', 'Warning'], ['danger', 'Danger']];
  const onTone = (role) => {
    const val = v(`color/${role}`).valuesByMode[collection.defaultModeId];
    return lum(val) > 0.5 ? 'color/background' : 'color/text';
  };
  const tint = (node, role) => {
    bindFill(node, `color/${role}`);
    node.fills = node.fills.map((p) => ({ ...p, opacity: 0.12 }));
  };
  const fmk = (name, w, h, x, y) => {
    const c = figma.createComponent();
    c.name = name;
    c.resize(w, h);
    c.x = x;
    c.y = y;
    c.fills = [];
    feedback.appendChild(c);
    return c;
  };
  const badges = toneRoles.map(([role, title], i) => {
    const c = fmk(`Tone=${title}`, 40, sz('badge-min'), i * 80, 0);
    c.layoutMode = 'HORIZONTAL';
    c.counterAxisAlignItems = 'CENTER';
    c.primaryAxisAlignItems = 'CENTER';
    c.paddingLeft = c.paddingRight = sz('chip-pad');
    c.cornerRadius = sz('badge-min') / 2;
    bindFill(c, `color/${role}`);
    c.appendChild(text('12', 12, onTone(role)));
    return c;
  });
  const badgeSet = figma.combineAsVariants(badges, feedback);
  badgeSet.name = 'Badge';
  const alerts = toneRoles.map(([role, title], i) => {
    const c = fmk(`Tone=${title}`, 320, 56, 0, 80 + i * 80);
    c.layoutMode = 'HORIZONTAL';
    c.itemSpacing = 12;
    c.paddingLeft = c.paddingRight = 16;
    c.paddingTop = c.paddingBottom = 10;
    c.counterAxisAlignItems = 'CENTER';
    bind(c, 'cornerRadius', 'shape/radius');
    tint(c, role);
    bindStroke(c, `color/${role}`);
    c.appendChild(ico(['info', 'check', 'warning', 'close'][i], 18, `color/${role}`));
    const col = figma.createFrame();
    col.layoutMode = 'VERTICAL';
    col.fills = [];
    col.appendChild(text(`${title} title`, 15, 'color/text'));
    col.appendChild(text('Message body', 13, 'color/text-muted'));
    c.appendChild(col);
    return c;
  });
  const alertSet = figma.combineAsVariants(alerts, feedback);
  alertSet.name = 'Alert';
  alertSet.y = 80;
  const chips = [false, true].map((on, i) => {
    const c = fmk(`Selected=${on ? 'true' : 'false'}`, 96, sz('chip-height'), i * 120, 0);
    c.layoutMode = 'HORIZONTAL';
    c.counterAxisAlignItems = 'CENTER';
    c.primaryAxisAlignItems = 'CENTER';
    c.paddingLeft = c.paddingRight = sz('chip-pad');
    bind(c, 'cornerRadius', 'shape/radius');
    if (on) bindFill(c, 'color/selection');
    bindStroke(c, on ? 'color/primary' : 'color/border');
    c.appendChild(text('Filter', 14, on ? 'color/text' : 'color/text-muted'));
    return c;
  });
  const chipSet = figma.combineAsVariants(chips, feedback);
  chipSet.name = 'Chip';
  chipSet.y = 420;
  parts2.push('Badge', 'Alert', 'Chip');

  // Card, Avatar, Snackbar, Dialog: gpui `card`, `snackbar`, `dialog` recipes and the Avatar tile.
  const cards = ['Elevated', 'Filled', 'Outlined'].map((variant, i) => {
    const c = fmk(`Variant=${variant}`, 160, 96, i * 180, 0);
    c.layoutMode = 'VERTICAL';
    c.itemSpacing = 4;
    c.paddingLeft = c.paddingRight = c.paddingTop = c.paddingBottom = 16;
    bind(c, 'cornerRadius', 'shape/radius');
    if (variant === 'Filled') bindFill(c, 'color/surface-raised');
    else if (variant === 'Elevated') bindFill(c, 'color/background');
    if (variant !== 'Filled') bindStroke(c, 'color/border');
    c.appendChild(text(`${variant} card`, 15, 'color/text'));
    c.appendChild(text('Supporting text', 13, 'color/text-muted'));
    return c;
  });
  const cardSet = figma.combineAsVariants(cards, feedback);
  cardSet.name = 'Card';
  cardSet.y = 500;
  const avatars = [false, true].map((status, i) => {
    const d = 36;
    const c = fmk(`Status=${status ? 'on' : 'off'}`, d, d, i * 60, 0);
    const tile = figma.createEllipse();
    tile.resize(d, d);
    bindFill(tile, 'color/surface-raised');
    bindStroke(tile, 'color/border');
    c.appendChild(tile);
    const ini = text('AB', 13, 'color/text');
    c.appendChild(ini);
    ini.x = (d - ini.width) / 2;
    ini.y = (d - ini.height) / 2;
    if (status) {
      const s = sz('status-dot');
      const dot = figma.createEllipse();
      dot.resize(s, s);
      bindFill(dot, 'color/success');
      bindStroke(dot, 'color/background');
      c.appendChild(dot);
      dot.x = d - s;
      dot.y = d - s;
    }
    return c;
  });
  const avatarSet = figma.combineAsVariants(avatars, feedback);
  avatarSet.name = 'Avatar';
  avatarSet.y = 630;
  const snack = fmk('Snackbar', 320, 48, 0, 700);
  snack.layoutMode = 'HORIZONTAL';
  snack.counterAxisAlignItems = 'CENTER';
  snack.itemSpacing = 16;
  snack.paddingLeft = snack.paddingRight = 16;
  bind(snack, 'cornerRadius', 'shape/radius');
  bindFill(snack, 'color/surface-raised');
  const snackText = text('Message sent', 14, 'color/text');
  snack.appendChild(snackText);
  snack.appendChild(text('UNDO', 13, 'color/primary'));
  const dlg = fmk('Dialog', sz('dialog-width'), 140, 0, 780);
  dlg.layoutMode = 'VERTICAL';
  dlg.itemSpacing = 12;
  dlg.paddingLeft = dlg.paddingRight = dlg.paddingTop = dlg.paddingBottom = 24;
  bind(dlg, 'cornerRadius', 'shape/radius');
  bindFill(dlg, 'color/surface');
  bindStroke(dlg, 'color/border-strong');
  dlg.appendChild(text('Dialog title', 20, 'color/text'));
  dlg.appendChild(text('Dialog body text explaining the choice.', 14, 'color/text-muted'));
  const acts = figma.createFrame();
  acts.layoutMode = 'HORIZONTAL';
  acts.itemSpacing = 12;
  acts.fills = [];
  acts.appendChild(text('Cancel', 14, 'color/text-muted'));
  acts.appendChild(text('Confirm', 14, 'color/primary'));
  dlg.appendChild(acts);
  parts2.push('Card', 'Avatar', 'Snackbar', 'Dialog');

  // --- Components: the rest of the showcase's inputs, navigation, data display and overlays, each
  // built from the same role/size/shape variables as the gpui widgets (frames, not variants).
  const kit = figma.createPage();
  kit.name = `${label} / Components`;
  figma.currentPage = kit;
  const bg0 = figma.createRectangle();
  bg0.name = 'Ground';
  bg0.resize(1320, 2400);
  bg0.x = -60;
  bg0.y = -60;
  bindFill(bg0, 'color/background');
  kit.appendChild(bg0);
  let kx = 0, ky = 0, krow = 0, kcount = 0;
  const put = (name, node) => {
    cover(name);
    node.name = name;
    kit.appendChild(node);
    if (kx > 0 && kx + node.width > 1200) { kx = 0; ky += krow + 56; krow = 0; }
    node.x = kx;
    node.y = ky + 18;
    const cap = text(name, 11, 'color/text-muted');
    kit.appendChild(cap);
    cap.x = kx;
    cap.y = ky;
    kx += Math.max(node.width, cap.width) + 40;
    krow = Math.max(krow, node.height + 18);
    kcount += 1;
    return node;
  };
  // A frame: dir H/V, hugging its children, with optional fill/stroke/radius/padding/gap.
  const box = (o = {}, ...kids) => {
    const f = figma.createFrame();
    f.layoutMode = o.dir === 'V' ? 'VERTICAL' : 'HORIZONTAL';
    f.primaryAxisSizingMode = 'AUTO';
    f.counterAxisSizingMode = 'AUTO';
    f.counterAxisAlignItems = o.align || 'CENTER';
    if (o.main) f.primaryAxisAlignItems = o.main;
    f.itemSpacing = o.gap ?? 8;
    const [px = 0, py = px] = o.pad || [];
    f.paddingLeft = f.paddingRight = px;
    f.paddingTop = f.paddingBottom = py;
    f.fills = [];
    if (o.fill) bindFill(f, o.fill);
    if (o.stroke) { bindStroke(f, o.stroke); f.strokeWeight = o.sw || 1; }
    if (o.round === true) bind(f, 'cornerRadius', 'shape/radius');
    else if (typeof o.round === 'number') f.cornerRadius = o.round;
    if (o.w) { f.resize(o.w, f.height); f.counterAxisSizingMode = o.dir === 'V' ? 'FIXED' : 'AUTO'; if (o.dir !== 'V') f.primaryAxisSizingMode = 'FIXED'; }
    if (o.h) { f.resize(f.width, o.h); if (o.dir === 'V') f.primaryAxisSizingMode = 'FIXED'; else f.counterAxisSizingMode = 'FIXED'; }
    for (const k of kids) f.appendChild(k);
    return f;
  };
  const T = (s, size = 14, fill = 'color/text') => text(s, size, fill);
  const dot = (d, fill, stroke) => { const e = figma.createEllipse(); e.resize(d, d); bindFill(e, fill); if (stroke) bindStroke(e, stroke); return e; };
  const bar = (w, h, fill, r = h / 2) => { const e = figma.createRectangle(); e.resize(w, h); e.cornerRadius = r; bindFill(e, fill); return e; };
  const field = (s, w = 200, muted = true, o = {}) => box({ w, h: num('shape/control-height'), pad: [12, 0], fill: 'color/surface', stroke: 'color/outline', round: true, gap: 8, ...o }, T(s, 14, muted ? 'color/text-muted' : 'color/text'));
  const ch = (s = 'v') => ico(s === '^' ? 'chevron_up' : 'chevron_down', 16, 'color/text-muted');
  const ctrl = num('shape/control-height');
  // Inter has no ▾ ⌕ ⋮ ☰ ⌂ ≡, so the icons the spec needs are drawn or spelled with glyphs it has.
  const ring = (d = 14, fill = 'color/text-muted') => { const e = figma.createEllipse(); e.resize(d, d); e.fills = []; bindStroke(e, fill); e.strokeWeight = 2; return e; };
  const burger = (fill = 'color/text') => ico('menu', 20, fill);

  // Inputs
  put('TextInput', field('Placeholder'));
  put('Select', box({ w: 200, h: ctrl, pad: [12, 0], fill: 'color/surface', stroke: 'color/outline', round: true, main: 'SPACE_BETWEEN' }, T('Choose…', 14, 'color/text-muted'), ch()));
  put('Combobox', box({ w: 200, h: ctrl, pad: [12, 0], fill: 'color/surface', stroke: 'color/primary', sw: 2, round: true, main: 'SPACE_BETWEEN' }, T('Typ', 14), ico('chevron_down', 16, 'color/text-muted')));
  put('NumberStepper', box({ gap: 0, fill: 'color/surface', stroke: 'color/outline', round: true }, box({ w: ctrl, h: ctrl, main: 'CENTER' }, ico('minus', 18)), box({ w: 56, h: ctrl, main: 'CENTER' }, T('3')), box({ w: ctrl, h: ctrl, main: 'CENTER' }, ico('plus', 18))));
  put('OtpInput', box({ gap: 8 }, ...[0, 1, 2, 3].map((i) => box({ w: 40, h: 48, main: 'CENTER', fill: 'color/surface', stroke: i === 2 ? 'color/primary' : 'color/outline', sw: i === 2 ? 2 : 1, round: true }, T(i < 2 ? String(i + 4) : '', 18)))));
  put('Rating', box({ gap: 4 }, ...[1, 1, 1, 0, 0].map((on) => ico('star', 24, on ? 'color/warning' : 'color/outline-variant'))));
  put('SearchBar', box({ w: 280, h: ctrl + 8, pad: [16, 0], fill: 'color/surface-container-high', round: 999, gap: 12 }, ico('search', 18, 'color/text-muted'), T('Search', 14, 'color/text-muted')));
  put('DateInput', field('dd / mm / yyyy', 180));
  put('TimeInput', field('hh : mm', 120));
  put('TagField', box({ w: 260, pad: [8, 8], fill: 'color/surface', stroke: 'color/outline', round: true, gap: 6 }, ...['alpha', 'beta'].map((s) => box({ pad: [8, 2], fill: 'color/selection', stroke: 'color/primary', round: true, gap: 4 }, T(s, 12), ico('close', 14, 'color/text-muted'))), T('add…', 13, 'color/text-muted')));
  put('RangeSlider', (() => { const th = sz('slider-track'), kn = sz('slider-thumb'); const g = box({ w: 200, h: kn, gap: 0 }); g.layoutMode = 'NONE'; const t = bar(200, th, 'color/surface-container-highest'); const f = bar(100, th, 'color/primary'); g.appendChild(t); g.appendChild(f); f.x = 40; const a = dot(kn, 'color/primary'); const b = dot(kn, 'color/primary'); g.appendChild(a); g.appendChild(b); a.x = 40 - kn / 2; b.x = 140 - kn / 2; t.y = f.y = (kn - th) / 2; return g; })());
  put('FileDropZone', box({ dir: 'V', w: 280, h: 110, main: 'CENTER', fill: 'color/surface', stroke: 'color/outline-variant', round: true, gap: 6 }, ico('upload', 24, 'color/text-muted'), T('Drop files here', 14, 'color/text-muted')));
  put('FormField', box({ dir: 'V', align: 'MIN', gap: 4 }, T('Label', 12, 'color/text-muted'), field('Value', 220, false), T('Helper text', 12, 'color/text-muted')));
  put('ColorPicker', box({ gap: 8 }, ...['primary', 'secondary', 'tertiary', 'info', 'success', 'warning', 'danger'].map((r, i) => dot(28, `color/${r}`, i === 0 ? 'color/text' : null))));

  // Actions
  put('IconButton', box({ w: sz('touch-target'), h: sz('touch-target'), main: 'CENTER', round: 999, stroke: 'color/outline' }, ico('heart', 18)));
  put('Fab', box({ w: 56, h: 56, main: 'CENTER', fill: 'color/primary-container', round: 16 }, ico('plus', 26, 'color/on-primary-container')));
  put('ExtendedFab', box({ h: 56, pad: [20, 0], fill: 'color/primary-container', round: 16, gap: 8 }, ico('plus', 22, 'color/on-primary-container'), T('Create', 15, 'color/on-primary-container')));
  put('SplitButton', box({ gap: 1 }, box({ h: ctrl, pad: [16, 0], fill: 'color/primary', round: 8 }, T('Save', 15, 'color/on-primary')), box({ h: ctrl, pad: [10, 0], fill: 'color/primary', round: 8 }, ico('chevron_down', 14, 'color/on-primary'))));
  put('Segmented', box({ gap: 0, stroke: 'color/outline', round: 999 }, ...['Day', 'Week', 'Month'].map((s, i) => box({ h: sz('segmented-height'), pad: [16, 0], fill: i === 1 ? 'color/secondary-container' : null }, T(s, 14, i === 1 ? 'color/on-secondary-container' : 'color/text')))));
  put('Toolbar', box({ pad: [8, 6], fill: 'color/surface-container', round: true, gap: 4 }, ...['format_bold', 'format_italic', 'format_underlined', 'list_bullets'].map((s) => box({ w: 32, h: 32, main: 'CENTER', round: 6 }, ico(s, 18)))));

  // Navigation
  put('TopAppBar', box({ w: 360, h: 64, pad: [16, 0], fill: 'color/surface', main: 'SPACE_BETWEEN' }, box({ gap: 16 }, burger(), T('Title', 20)), ico('dots', 16)));
  put('BottomAppBar', box({ w: 360, h: 72, pad: [16, 0], fill: 'color/surface-container', main: 'SPACE_BETWEEN' }, box({ gap: 24 }, ico('home', 18), ico('search', 18), ico('heart', 18)), box({ w: 48, h: 48, main: 'CENTER', fill: 'color/primary-container', round: 14 }, ico('plus', 22, 'color/on-primary-container'))));
  put('NavigationBar', box({ w: 360, h: 80, fill: 'color/surface-container', main: 'SPACE_BETWEEN', pad: [8, 12] }, ...['Home', 'Search', 'Saved'].map((s, i) => box({ dir: 'V', gap: 4 }, box({ w: 56, h: 28, main: 'CENTER', round: 999, fill: i === 0 ? 'color/secondary-container' : null }, ico(['home', 'search', 'heart'][i], 18, i === 0 ? 'color/on-secondary-container' : 'color/text')), T(s, 12, i === 0 ? 'color/text' : 'color/text-muted')))));
  put('NavigationRail', box({ dir: 'V', w: 80, pad: [8, 16], gap: 16, fill: 'color/surface' }, ...['home', 'search', 'heart'].map((g, i) => box({ dir: 'V', gap: 4 }, box({ w: 56, h: 28, main: 'CENTER', round: 999, fill: i === 0 ? 'color/secondary-container' : null }, ico(['home', 'search', 'heart'][i], 18, i === 0 ? 'color/on-secondary-container' : 'color/text')), T(['Home', 'Find', 'Saved'][i], 12, i === 0 ? 'color/text' : 'color/text-muted')))));
  put('NavigationDrawer', box({ dir: 'V', align: 'MIN', w: 240, pad: [12, 16], gap: 4, fill: 'color/surface', stroke: 'color/border', round: true }, ...['Inbox', 'Sent', 'Drafts'].map((s, i) => box({ w: 216, h: 48, pad: [16, 0], round: 999, fill: i === 0 ? 'color/secondary-container' : null }, T(s, 14, i === 0 ? 'color/on-secondary-container' : 'color/text')))));
  put('Breadcrumb', box({ gap: 8 }, T('Home', 14, 'color/link'), T('/', 14, 'color/text-muted'), T('Library', 14, 'color/link'), T('/', 14, 'color/text-muted'), T('Data', 14)));
  put('Pagination', box({ gap: 4 }, ...['‹', '1', '2', '3', '›'].map((s, i) => box({ w: 36, h: 36, main: 'CENTER', round: 8, fill: i === 2 ? 'color/primary' : null, stroke: i === 2 ? null : 'color/border' }, s === '‹' ? ico('chevron_left', 18) : s === '›' ? ico('chevron_right', 18) : T(s, 14, i === 2 ? 'color/on-primary' : 'color/text')))));
  put('Stepper', box({ gap: 8 }, ...[1, 2, 3].flatMap((n, i) => [box({ w: 28, h: 28, main: 'CENTER', round: 999, fill: n <= 2 ? 'color/primary' : null, stroke: n <= 2 ? null : 'color/outline' }, T(String(n), 13, n <= 2 ? 'color/on-primary' : 'color/text-muted')), ...(i < 2 ? [bar(32, 2, n < 2 ? 'color/primary' : 'color/outline-variant')] : [])])));
  put('TreeView', box({ dir: 'V', align: 'MIN', gap: 2, w: 200 }, T('v src', 14), box({ pad: [16, 0] }, T('lib.rs', 14, 'color/text-muted')), box({ pad: [16, 0], fill: 'color/selection', round: 6 }, T('main.rs', 14)), T('> docs', 14)));
  put('Menu', box({ dir: 'V', align: 'MIN', w: 200, pad: [0, 8], fill: 'color/surface-container', stroke: 'color/border', round: true, gap: 0 }, ...['Cut', 'Copy', 'Paste'].map((s, i) => box({ w: 200, h: 40, pad: [16, 0], fill: i === 1 ? 'color/selection' : null }, T(s, 14)))));
  put('DropdownMenu', box({ dir: 'V', align: 'MIN', gap: 4 }, box({ w: 140, h: ctrl, pad: [12, 0], fill: 'color/surface', stroke: 'color/outline', round: true, main: 'SPACE_BETWEEN' }, T('Actions', 14), ch()), box({ dir: 'V', align: 'MIN', w: 180, pad: [0, 6], fill: 'color/surface-container', stroke: 'color/border', round: true, gap: 0 }, ...['Edit', 'Share'].map((s) => box({ w: 180, h: 36, pad: [12, 0] }, T(s, 14))))));
  put('Tabs', box({ gap: 0 }, ...['One', 'Two', 'Three'].map((s, i) => box({ dir: 'V', pad: [20, 12], gap: 8 }, T(s, 15, i === 0 ? 'color/primary' : 'color/text-muted'), bar(48, 3, i === 0 ? 'color/primary' : 'color/border')))));
  put('Accordion', box({ dir: 'V', align: 'MIN', w: 280, stroke: 'color/border', round: true, gap: 0 }, box({ w: 280, h: 44, pad: [16, 0], main: 'SPACE_BETWEEN' }, T('Section', 15), ch('^')), box({ w: 280, pad: [16, 8] }, T('Expanded content', 13, 'color/text-muted'))));

  // Data display and feedback
  put('Tooltip / DodaiTooltip', box({ pad: [10, 6], fill: 'color/overlay', round: 6 }, T('Tooltip', 12)));
  put('RichTooltip', box({ dir: 'V', align: 'MIN', pad: [14, 10], fill: 'color/overlay', round: 10, gap: 4 }, T('Title', 14), T('Supporting text for the control.', 12, 'color/text-muted'), T('Learn more', 12, 'color/link')));
  put('Popover', box({ dir: 'V', align: 'MIN', pad: [16, 12], fill: 'color/surface-container', stroke: 'color/border', round: true, gap: 4 }, T('Popover', 15), T('Anchored content', 13, 'color/text-muted')));
  put('Toast', box({ pad: [16, 12], fill: 'color/surface-container-high', stroke: 'color/success', round: true, gap: 12 }, ico('check', 18, 'color/success'), T('Saved', 14)));
  put('Banner', box({ w: 360, h: 48, pad: [16, 0], fill: 'color/info', gap: 12 }, ico('info', 18, 'color/background'), T('Update available', 14, 'color/background')));
  put('Spinner', (() => { const e = figma.createEllipse(); e.resize(32, 32); e.fills = []; bindStroke(e, 'color/primary'); e.strokeWeight = 3; e.arcData = { startingAngle: 0, endingAngle: 4.7, innerRadius: 0.8 }; return e; })());
  put('CircularProgress', (() => { const g = box({ w: 40, h: 40, main: 'CENTER' }); g.layoutMode = 'NONE'; const e = figma.createEllipse(); e.resize(40, 40); e.fills = []; bindStroke(e, 'color/surface-container-highest'); e.strokeWeight = 4; g.appendChild(e); const a = figma.createEllipse(); a.resize(40, 40); a.fills = []; bindStroke(a, 'color/primary'); a.strokeWeight = 4; a.arcData = { startingAngle: -1.57, endingAngle: 2.9, innerRadius: 0.88 }; g.appendChild(a); const t = T('70', 12); g.appendChild(t); t.x = 20 - t.width / 2; t.y = 20 - t.height / 2; return g; })());
  put('IndeterminateBar', box({ w: 200, h: sz('bar-thickness'), round: 999, fill: 'color/surface-container-highest' }, bar(70, sz('bar-thickness'), 'color/primary')));
  put('Skeleton', box({ dir: 'V', align: 'MIN', gap: 8 }, bar(200, 14, 'color/surface-container-highest', 4), bar(140, 14, 'color/surface-container-highest', 4), bar(200, 64, 'color/surface-container-highest', 8)));
  put('Divider', bar(200, sz('hairline'), 'color/border', 0));
  put('Kbd', box({ pad: [8, 3], fill: 'color/surface-container-high', stroke: 'color/border-strong', round: 4 }, T('⌘ K', 12)));
  put('List', box({ dir: 'V', align: 'MIN', w: 280, gap: 0 }, ...['One', 'Two'].map((s) => box({ w: 280, h: 56, pad: [16, 0], gap: 16, stroke: 'color/border' }, dot(32, 'color/primary-container'), box({ dir: 'V', align: 'MIN', gap: 0 }, T(s, 15), T('Supporting text', 12, 'color/text-muted'))))));
  put('Table', box({ dir: 'V', align: 'MIN', w: 320, stroke: 'color/border', round: true, gap: 0 }, box({ w: 320, h: 36, pad: [12, 0], fill: 'color/surface-container', main: 'SPACE_BETWEEN' }, T('Name', 12, 'color/text-muted'), T('Qty', 12, 'color/text-muted')), ...['Alpha|3', 'Beta|9'].map((r) => { const [a, b] = r.split('|'); return box({ w: 320, h: 40, pad: [12, 0], main: 'SPACE_BETWEEN' }, T(a, 14), T(b, 14)); })));
  put('Timeline', box({ dir: 'V', align: 'MIN', gap: 0 }, ...['Created', 'Shipped', 'Delivered'].map((s, i) => box({ gap: 12, align: 'MIN' }, box({ dir: 'V', gap: 0, align: 'CENTER' }, dot(12, i < 2 ? 'color/primary' : 'color/outline'), ...(i < 2 ? [bar(2, 24, 'color/outline-variant', 0)] : [])), T(s, 14, i < 2 ? 'color/text' : 'color/text-muted')))));
  put('Clock', (() => { const g = box({ w: 96, h: 96 }); g.layoutMode = 'NONE'; const f = dot(96, 'color/surface'); bindStroke(f, 'color/outline'); g.appendChild(f); const h1 = bar(3, 28, 'color/text', 2); const h2 = bar(2, 38, 'color/primary', 1); g.appendChild(h1); g.appendChild(h2); h1.x = 47; h1.y = 20; h2.x = 47; h2.y = 10; return g; })());
  put('Stopwatch', box({ gap: 12, pad: [16, 10], fill: 'color/surface-container', round: true }, T('00:42.7', 22), ico('play', 16, 'color/primary')));
  put('CodeBlock', box({ dir: 'V', align: 'MIN', pad: [16, 12], fill: 'color/surface-container-lowest', stroke: 'color/border', round: true, gap: 2 }, T('fn main() {', 13, 'color/text'), T('    run();', 13, 'color/text-muted'), T('}', 13, 'color/text')));
  put('Stat (CountUp)', box({ dir: 'V', align: 'MIN', gap: 0 }, T('12,480', 32), T('Active users', 12, 'color/text-muted')));
  put('SideSheet', box({ dir: 'V', align: 'MIN', w: 240, h: 200, pad: [20, 16], fill: 'color/surface', stroke: 'color/border-strong', round: true, gap: 8 }, T('Side sheet', 18), T('Detail content', 13, 'color/text-muted')));
  put('BottomSheet', box({ dir: 'V', w: 320, h: 140, pad: [16, 12], fill: 'color/surface', stroke: 'color/border-strong', round: true, gap: 12 }, bar(sz('sheet-handle-w'), sz('sheet-handle-h'), 'color/outline-variant'), T('Bottom sheet', 16)));
  put('Carousel', box({ gap: 8 }, bar(120, 140, 'color/primary-container', 16), bar(80, 140, 'color/secondary-container', 16), bar(40, 140, 'color/tertiary-container', 16)));
  put('Image', box({ w: 160, h: 100, main: 'CENTER', fill: 'color/surface-container-high', round: true }, ico('image', 32, 'color/text-muted')));
  put('Markdown', box({ dir: 'V', align: 'MIN', gap: 4 }, T('Heading', 20), T('Body text with a link.', 14, 'color/text-muted'), T('• list item', 14)));
  put('SnapStack', box({ dir: 'V', gap: 6 }, bar(200, 56, 'color/primary-container', 12), bar(200, 56, 'color/secondary-container', 12)));
  // Batch two: pickers, tables, palettes, overlays, shells, selectors, effect layers, typography.
  const cal = (sel) => box({ dir: 'V', align: 'MIN', gap: 6, pad: [12, 12], fill: 'color/surface-container', stroke: 'color/border', round: true },
    box({ w: 224, main: 'SPACE_BETWEEN' }, ico('chevron_left', 16, 'color/text-muted'), T('March 2026', 14), ico('chevron_right', 16, 'color/text-muted')),
    box({ gap: 0 }, ...['M', 'T', 'W', 'T', 'F', 'S', 'S'].map((d) => box({ w: 32, h: 20, main: 'CENTER' }, T(d, 11, 'color/text-muted')))),
    ...[0, 1, 2].map((r) => box({ gap: 0 }, ...[1, 2, 3, 4, 5, 6, 7].map((c) => { const n = r * 7 + c; return box({ w: 32, h: 32, main: 'CENTER', round: 999, fill: n === sel ? 'color/primary' : null, stroke: n === 9 ? 'color/primary' : null }, T(String(n), 12, n === sel ? 'color/on-primary' : 'color/text')); }))));
  put('DatePicker', cal(12));
  put('DateTimePicker', box({ dir: 'V', gap: 8 }, cal(12), box({ gap: 8 }, field('10 : 30', 100, false), box({ gap: 0, stroke: 'color/outline', round: 8 }, box({ w: 44, h: ctrl, main: 'CENTER', fill: 'color/secondary-container' }, T('AM', 13, 'color/on-secondary-container')), box({ w: 44, h: ctrl, main: 'CENTER' }, T('PM', 13))))));
  put('DateTimeRangePicker', box({ gap: 8 }, field('12 Mar 10:00', 150, false), ico('arrow_right', 16, 'color/text-muted'), field('14 Mar 18:30', 150, false)));
  put('TimePicker', box({ dir: 'V', gap: 8, pad: [16, 16], fill: 'color/surface-container', stroke: 'color/border', round: true }, box({ gap: 6 }, box({ w: 72, h: 64, main: 'CENTER', fill: 'color/primary-container', round: 10 }, T('10', 32, 'color/on-primary-container')), T(':', 32), box({ w: 72, h: 64, main: 'CENTER', fill: 'color/surface-container-highest', round: 10 }, T('30', 32)))));
  put('DataTable', box({ dir: 'V', align: 'MIN', w: 420, stroke: 'color/border', round: true, gap: 0 }, box({ w: 420, h: 40, pad: [12, 0], fill: 'color/surface-container', gap: 12 }, bar(14, 14, 'color/outline', 3), T('Name ↑', 12, 'color/text-muted'), T('Status', 12, 'color/text-muted'), T('Qty', 12, 'color/text-muted')), ...['Alpha|Active|3', 'Beta|Paused|9', 'Gamma|Active|4'].map((r, i) => { const [a, b, c] = r.split('|'); return box({ w: 420, h: 44, pad: [12, 0], gap: 12, fill: i === 1 ? 'color/selection' : null }, bar(14, 14, i === 1 ? 'color/primary' : 'color/outline', 3), T(a, 14), T(b, 14, 'color/text-muted'), T(c, 14)); })));
  put('Command', box({ dir: 'V', align: 'MIN', w: 360, fill: 'color/surface-container', stroke: 'color/border', round: true, gap: 0 }, box({ w: 360, h: 48, pad: [16, 0], gap: 12, stroke: 'color/border' }, ico('search', 18, 'color/text-muted'), T('Type a command…', 14, 'color/text-muted')), ...['Open file|⌘ O', 'Save|⌘ S', 'Settings|⌘ ,'].map((r, i) => { const [a, b] = r.split('|'); return box({ w: 360, h: 40, pad: [16, 0], main: 'SPACE_BETWEEN', fill: i === 0 ? 'color/selection' : null }, T(a, 14), T(b, 12, 'color/text-muted')); })));
  put('CommandPalette', box({ dir: 'V', align: 'MIN', w: 420, fill: 'color/surface-container-high', stroke: 'color/border-strong', round: true, gap: 0 }, box({ w: 420, h: 52, pad: [16, 0], gap: 12 }, ico('search', 20, 'color/text-muted'), T('Search commands', 16, 'color/text-muted')), box({ w: 420, pad: [16, 8], fill: 'color/surface-container' }, T('Recent', 11, 'color/text-muted')), ...['Toggle theme', 'Open charts', 'Show shaders'].map((s, i) => box({ w: 420, h: 40, pad: [16, 0], fill: i === 1 ? 'color/selection' : null }, T(s, 14)))));
  put('ContextMenu', box({ dir: 'V', align: 'MIN', w: 180, pad: [0, 6], fill: 'color/surface-container-high', stroke: 'color/border', round: true, gap: 0 }, ...['Open', 'Rename', 'Delete'].map((s, i) => box({ w: 180, h: 36, pad: [12, 0], fill: i === 1 ? 'color/selection' : null }, T(s, 14, i === 2 ? 'color/danger' : 'color/text')))));
  put('HoverCard', box({ gap: 12, pad: [16, 14], fill: 'color/surface-container-high', stroke: 'color/border', round: true }, dot(40, 'color/primary-container'), box({ dir: 'V', align: 'MIN', gap: 2 }, T('@dodai', 15), T('Design system · 4 designs', 12, 'color/text-muted'))));
  put('FullScreenDialog', box({ dir: 'V', align: 'MIN', w: 260, h: 180, fill: 'color/surface', stroke: 'color/border-strong', round: true, gap: 0 }, box({ w: 260, h: 52, pad: [16, 0], main: 'SPACE_BETWEEN' }, ico('close', 16), T('Save', 14, 'color/primary')), box({ pad: [16, 8] }, T('Full-screen content', 14, 'color/text-muted'))));
  put('ModalDrawer', box({ dir: 'V', align: 'MIN', w: 240, h: 200, pad: [16, 16], fill: 'color/surface-container-low', stroke: 'color/border', round: true, gap: 8 }, T('Drawer', 18), ...['Inbox', 'Starred', 'Sent'].map((s) => box({ w: 208, h: 40, pad: [12, 0] }, T(s, 14, 'color/text-muted')))));
  put('ResponsiveShell', box({ gap: 0, stroke: 'color/border', round: true }, box({ dir: 'V', w: 64, h: 180, pad: [8, 16], gap: 16, fill: 'color/surface-container' }, ...[0, 1, 2].map((i) => bar(32, 8, i === 0 ? 'color/primary' : 'color/outline-variant'))), box({ dir: 'V', align: 'MIN', w: 240, h: 180, pad: [16, 12], gap: 8 }, bar(120, 14, 'color/surface-container-highest', 4), bar(200, 60, 'color/surface-container', 8), bar(200, 40, 'color/surface-container', 8))));
  put('SearchView', box({ dir: 'V', align: 'MIN', w: 320, fill: 'color/surface-container-high', round: 24, gap: 0 }, box({ w: 320, h: 52, pad: [16, 0], gap: 12 }, ico('search', 18, 'color/text-muted'), T('Search', 15, 'color/text-muted')), ...['Recent one', 'Recent two'].map((s) => box({ w: 320, h: 44, pad: [20, 0], gap: 12 }, ico('history', 16, 'color/text-muted'), T(s, 14)))));
  put('SelectOption', box({ dir: 'V', align: 'MIN', w: 200, pad: [0, 6], fill: 'color/surface-container', stroke: 'color/border', round: true }, ...['Alpha', 'Beta', 'Gamma'].map((s, i) => box({ w: 200, h: 36, pad: [12, 0], main: 'SPACE_BETWEEN', fill: i === 1 ? 'color/selection' : null }, T(s, 14), ...(i === 1 ? [ico('check', 15, 'color/primary')] : [])))));
  put('ThemeSelector', box({ gap: 8 }, ...['primary', 'secondary', 'tertiary', 'info'].map((r, i) => box({ w: 56, h: 56, main: 'CENTER', fill: 'color/surface', stroke: i === 0 ? 'color/primary' : 'color/border', sw: i === 0 ? 2 : 1, round: true }, dot(28, `color/${r}`)))));
  put('ThemeSwitcherBar', box({ pad: [12, 8], fill: 'color/surface-container', stroke: 'color/border', round: 999, gap: 12 }, T('Dark', 13), box({ w: 44, h: 24, pad: [2, 0], fill: 'color/primary', round: 999, main: 'MAX' }, dot(20, 'color/on-primary')), T('Light', 13, 'color/text-muted')));
  put('DesignSystemSelector', box({ gap: 0, stroke: 'color/outline', round: 999 }, ...['Cyberpunk', 'Material', 'Neo-Brutalism', 'Dodai'].map((s, i) => box({ h: sz('segmented-height'), pad: [14, 0], fill: i === 3 ? 'color/secondary-container' : null }, T(s, 13, i === 3 ? 'color/on-secondary-container' : 'color/text')))));
  put('Knob', (() => { const g = box({ w: 72, h: 72 }); g.layoutMode = 'NONE'; const t = ring(64, 'color/surface-container-highest'); t.strokeWeight = 6; g.appendChild(t); t.x = 4; t.y = 4; const a = ring(64, 'color/primary'); a.strokeWeight = 6; a.arcData = { startingAngle: 2.2, endingAngle: 4.9, innerRadius: 0.9 }; g.appendChild(a); a.x = 4; a.y = 4; const v = T('64', 16); g.appendChild(v); v.x = 26; v.y = 26; return g; })());
  put('Timer', box({ gap: 12, pad: [16, 10], fill: 'color/surface-container', round: true }, T('05:00', 22), ico('stop', 16, 'color/danger')));
  put('TapTempo', box({ dir: 'V', gap: 4, w: 96, h: 96, main: 'CENTER', fill: 'color/primary-container', round: 999 }, T('120', 26, 'color/on-primary-container'), T('BPM', 11, 'color/on-primary-container')));
  put('Activity feed', box({ dir: 'V', align: 'MIN', gap: 0, w: 300 }, ...['New release|2m', 'Build passed|1h'].map((r) => { const [a, b] = r.split('|'); return box({ w: 300, h: 52, gap: 12, stroke: 'color/border' }, dot(10, 'color/primary'), T(a, 14), T(b, 12, 'color/text-muted')); })));
  put('Panel', box({ dir: 'V', align: 'MIN', w: 240, h: 120, pad: [16, 14], fill: 'color/surface', stroke: 'color/border', round: true, gap: 6 }, T('Panel', 16), T('Container surface', 13, 'color/text-muted')));
  put('SplitPane', box({ gap: 0, stroke: 'color/border', round: true }, box({ w: 140, h: 110, pad: [12, 12], fill: 'color/surface' }, T('Left', 14)), bar(6, 110, 'color/outline-variant', 0), box({ w: 180, h: 110, pad: [12, 12], fill: 'color/surface' }, T('Right', 14))));
  put('ScrollArea', box({ gap: 4, h: 110 }, box({ dir: 'V', align: 'MIN', w: 200, h: 110, pad: [12, 8], gap: 6, fill: 'color/surface', stroke: 'color/border', round: true }, T('Row one', 14), T('Row two', 14), T('Row three', 14)), bar(6, 50, 'color/outline')));
  put('ScrollSpy', box({ dir: 'V', align: 'MIN', gap: 4 }, ...['Intro', 'Usage', 'API'].map((s, i) => box({ gap: 8 }, bar(2, 18, i === 1 ? 'color/primary' : 'color/outline-variant', 1), T(s, 14, i === 1 ? 'color/text' : 'color/text-muted')))));
  put('IconBadge', (() => { const g = box({ w: 40, h: 40 }); g.layoutMode = 'NONE'; const heart = ico('heart', 22); g.appendChild(heart); heart.x = 10; heart.y = 8; const b = box({ w: 16, h: sz('badge-min'), main: 'CENTER', fill: 'color/danger', round: 999 }, T('3', 10, 'color/on-error')); g.appendChild(b); b.x = 24; b.y = 0; return g; })());
  put('SelectionTrack/Marker', box({ gap: 0 }, bar(120, 4, 'color/primary', 2), dot(16, 'color/primary'), bar(60, 4, 'color/outline-variant', 2)));
  put('Shimmer', box({ w: 200, h: 24, round: 6, fill: 'color/surface-container-highest' }, bar(60, 24, 'color/surface-container-high', 6)));
  put('LoadingIndicator', box({ gap: 6 }, ...[0.4, 0.7, 1, 0.7, 0.4].map((a) => { const d = dot(12, 'color/primary'); d.opacity = a; return d; })));
  // Effect layers (the full-surface effects the showcase applies over an app).
  put('FrostLayer', (() => { const g = box({ w: 220, h: 110, round: true }); g.layoutMode = 'NONE'; g.clipsContent = true; const a = bar(220, 110, 'color/primary-container', 0); g.appendChild(a); const d1 = dot(60, 'color/secondary'); g.appendChild(d1); d1.x = 30; d1.y = 30; const pane = figma.createRectangle(); pane.resize(140, 70); pane.cornerRadius = 12; bindFill(pane, 'color/surface'); pane.fills = pane.fills.map((p) => ({ ...p, opacity: 0.5 })); pane.effects = [{ type: 'BACKGROUND_BLUR', radius: 16, visible: true }]; g.appendChild(pane); pane.x = 40; pane.y = 20; return g; })());
  put('GlowLayer', (() => { const g = box({ w: 160, h: 80, main: 'CENTER', fill: 'color/surface', stroke: 'color/primary', round: true }, T('Glow', 16, 'color/primary')); g.effects = [{ type: 'DROP_SHADOW', color: { r: 0.2, g: 0.8, b: 1, a: 0.7 }, offset: { x: 0, y: 0 }, radius: 18, spread: 0, visible: true, blendMode: 'NORMAL' }]; return g; })());
  put('Scanlines', (() => { const g = box({ w: 160, h: 80, fill: 'color/surface', round: true }); g.layoutMode = 'NONE'; g.clipsContent = true; for (let y = 0; y < 80; y += 4) { const l = bar(160, 1, 'color/text', 0); l.opacity = 0.18; g.appendChild(l); l.y = y; } const t = T('Scan', 18); g.appendChild(t); t.x = 16; t.y = 28; return g; })());
  put('FrostedScene', (() => { const g = box({ w: 220, h: 110, round: true }); g.layoutMode = 'NONE'; g.clipsContent = true; const a = bar(220, 110, 'color/tertiary-container', 0); g.appendChild(a); const b = bar(220, 40, 'color/primary-container', 0); g.appendChild(b); b.y = 70; const pane = figma.createRectangle(); pane.resize(120, 60); pane.cornerRadius = 12; bindFill(pane, 'color/surface'); pane.fills = pane.fills.map((p) => ({ ...p, opacity: 0.45 })); pane.effects = [{ type: 'BACKGROUND_BLUR', radius: 20, visible: true }]; g.appendChild(pane); pane.x = 50; pane.y = 25; return g; })());
  // Typography.
  put('Heading / Text / Label', box({ dir: 'V', align: 'MIN', gap: 4 }, T('Heading', 28), T('Body text', 16), T('Label', 12, 'color/text-muted'), T('Mono 0x2A', 13, 'color/text-muted')));
  put('Icon', box({ gap: 12 }, ico('heart', 26), ico('star', 26, 'color/warning'), ico('check', 26, 'color/success')));
  put('TreeItem / TreeRow', box({ dir: 'V', align: 'MIN', gap: 0, w: 220 }, box({ w: 220, h: 32, pad: [8, 0], gap: 6 }, ico('chevron_down', 14, 'color/text-muted'), T('Folder', 14)), box({ w: 220, h: 32, pad: [28, 0], fill: 'color/selection', round: 6 }, T('file.txt', 14))));
  put('NavigationItem', box({ w: 200, h: 48, pad: [16, 0], gap: 12, fill: 'color/secondary-container', round: 999 }, ico('home', 16, 'color/on-secondary-container'), T('Home', 14, 'color/on-secondary-container')));
  put('MenuItem', box({ w: 200, h: 40, pad: [16, 0], main: 'SPACE_BETWEEN', fill: 'color/selection' }, T('Copy', 14), T('⌘ C', 12, 'color/text-muted')));
  put('TimelineEvent', box({ gap: 12, align: 'MIN' }, dot(12, 'color/primary'), box({ dir: 'V', align: 'MIN', gap: 0 }, T('Shipped', 14), T('2 days ago', 12, 'color/text-muted'))));
  put('SelectionMarker', box({ gap: 8 }, box({ w: 22, h: 22, main: 'CENTER', fill: 'color/primary', round: 4 }, ico('check', 16, 'color/on-primary')), box({ w: 22, h: 22, stroke: 'color/outline', round: 4 })));
  put('Pagination (compact)', box({ gap: 12 }, ico('chevron_left', 18), T('3 / 12', 14), ico('chevron_right', 18)));
  put('Mono / Muted', box({ dir: 'V', align: 'MIN', gap: 2 }, T('fn main()', 13), T('muted caption', 12, 'color/text-muted')));
  // States of the interactive parts: default, hover (state layer), focus (ring), error, disabled.
  const layer = (n) => { const o = figma.createRectangle(); o.resize(n.width, n.height); bindFill(o, 'color/text'); o.opacity = 0.08; n.layoutMode === 'NONE' || n.appendChild(o); o.layoutPositioning = 'ABSOLUTE'; o.x = 0; o.y = 0; return n; };
  const dim = (n) => { n.opacity = 0.38; return n; };
  const stateRow = (name, mk) => put(`${name} states`, box({ dir: 'V', align: 'MIN', gap: 10 }, ...['Default', 'Hover', 'Focus', 'Disabled'].map((st) => box({ gap: 12 }, box({ w: 64 }, T(st, 11, 'color/text-muted')), mk(st)))));
  stateRow('Field', (st) => { const f = field(st === 'Focus' ? 'Typing' : 'Placeholder', 200, st !== 'Focus', { stroke: st === 'Focus' ? 'color/primary' : 'color/outline', sw: st === 'Focus' ? 2 : 1 }); if (st === 'Hover') layer(f); return st === 'Disabled' ? dim(f) : f; });
  stateRow('Field (error)', (st) => { const f = field(st === 'Disabled' ? 'Disabled' : 'Invalid value', 200, false, { stroke: st === 'Disabled' ? 'color/outline' : 'color/danger', sw: st === 'Focus' ? 2 : 1 }); if (st === 'Hover') layer(f); return st === 'Disabled' ? dim(f) : f; });
  stateRow('Chip', (st) => { const c = box({ h: sz('chip-height'), pad: [sz('chip-pad'), 0], stroke: st === 'Focus' ? 'color/primary' : 'color/border', sw: st === 'Focus' ? 2 : 1, round: true, fill: st === 'Hover' ? 'color/surface-container-high' : null }, T('Filter', 14, 'color/text-muted')); return st === 'Disabled' ? dim(c) : c; });
  stateRow('List item', (st) => { const c = box({ w: 240, h: 48, pad: [16, 0], round: 8, stroke: st === 'Focus' ? 'color/primary' : null, sw: 2, fill: st === 'Hover' ? 'color/surface-container-high' : null }, T('List item', 14)); return st === 'Disabled' ? dim(c) : c; });
  stateRow('Switch', (st) => { const on = st === 'Focus'; const w = sz('switch-track-w'), h = sz('switch-track-h'); const g = box({ w, h }); g.layoutMode = 'NONE'; const t = bar(w, h, on ? 'color/primary' : 'color/surface-container-highest'); bindStroke(t, 'color/outline'); g.appendChild(t); const d = on ? sz('switch-handle-on') : sz('switch-handle-off'); const k = dot(st === 'Hover' ? d + 4 : d, on ? 'color/on-primary' : 'color/outline'); g.appendChild(k); k.x = on ? w - d - (h - d) / 2 : (h - d) / 2; k.y = (h - k.height) / 2; return st === 'Disabled' ? dim(g) : g; });
  stateRow('Checkbox', (st) => { const n = sz('selection-box'); const g = box({ w: n, h: n, main: 'CENTER', stroke: st === 'Focus' ? 'color/primary' : 'color/outline', sw: st === 'Focus' ? 2 : 1, round: 4, fill: st === 'Hover' ? 'color/surface-container-high' : null }); return st === 'Disabled' ? dim(g) : g; });
  stateRow('Icon button', (st) => { const c = box({ w: sz('touch-target'), h: sz('touch-target'), main: 'CENTER', round: 999, stroke: st === 'Focus' ? 'color/primary' : null, sw: 2, fill: st === 'Hover' ? 'color/surface-container-high' : null }, ico('heart', 18)); return st === 'Disabled' ? dim(c) : c; });
  // Components added with the gizmo, dialogs and trees of the toolkits.
  put('AxisGizmo', (() => {
    const S = 130, c = S / 2;
    const g = box({ w: S, h: S });
    g.layoutMode = 'NONE';
    const disc = dot(S, 'color/surface');
    bindStroke(disc, 'color/border-strong');
    disc.opacity = 0.9;
    g.appendChild(disc);
    const stem = (x, y, fill) => { const l = figma.createVector(); l.vectorPaths = [{ windingRule: 'NONE', data: `M ${c} ${c} L ${x} ${y}` }]; l.fills = []; bindStroke(l, fill); l.strokeWeight = 2; g.appendChild(l); };
    const ball = (x, y, fill, label, solid) => {
      const d = 22;
      stem(x, y, fill);
      const b = dot(d, fill);
      if (!solid) { b.fills = []; bindStroke(b, fill); b.strokeWeight = 2; }
      g.appendChild(b); b.x = x - d / 2; b.y = y - d / 2;
      if (label) { const t = T(label, 11, solid ? 'color/background' : fill); g.appendChild(t); t.x = x - t.width / 2; t.y = y - t.height / 2; }
    };
    // Negative ends first so the near ones paint over them.
    ball(c - 34, c - 12, 'color/danger', '', false);
    ball(c, c + 36, 'color/success', '', false);
    ball(c - 16, c - 22, 'color/info', '', false);
    ball(c + 34, c + 12, 'color/danger', 'X', true);
    ball(c, c - 36, 'color/success', 'Y', true);
    ball(c + 16, c + 22, 'color/info', 'Z', true);
    const btn = (cx, cy, d, id) => {
      const b = dot(d, 'color/surface-raised');
      bindStroke(b, 'color/border');
      g.appendChild(b); b.x = cx - d / 2; b.y = cy - d / 2;
      const i = ico(id, Math.round(d * 0.62), 'color/text');
      g.appendChild(i); i.x = cx - i.width / 2; i.y = cy - i.height / 2;
    };
    const r = (S / 2) * Math.SQRT1_2;
    btn(c - r, c - r, 30, 'home');
    btn(c - r, c + r, 26, 'minus');
    btn(c + r, c + r, 26, 'plus');
    return g;
  })());
  put('DialogCard', box({ dir: 'V', align: 'MIN', w: 300, pad: [20, 16], gap: 12, fill: 'color/surface-container-high', stroke: 'color/border-strong', round: true }, T('Dialog title', 18), T('Message body that explains the choice.', 14, 'color/text-muted'), box({ gap: 16, main: 'MAX', w: 260 }, T('Cancel', 14, 'color/text-muted'), T('OK', 14, 'color/primary'))));
  put('FabMenu', box({ dir: 'V', align: 'MAX', gap: 8 }, ...['Photo', 'File'].map((s) => box({ h: 36, pad: [14, 0], fill: 'color/secondary-container', round: 999, gap: 8 }, T(s, 13, 'color/on-secondary-container'))), box({ w: 56, h: 56, main: 'CENTER', fill: 'color/primary-container', round: 16 }, ico('close', 24, 'color/on-primary-container'))));
  put('InputBox', box({ w: 260, h: ctrl, pad: [12, 0], fill: 'color/surface', stroke: 'color/outline', round: true, gap: 8 }, T('$', 14, 'color/text-muted'), T('120.00', 14), bar(8, 1, 'color/surface', 0), T('USD', 12, 'color/text-muted')));
  put('ListItem', box({ w: 280, h: 56, pad: [16, 0], gap: 16, fill: 'color/selection', round: 8 }, ico('folder', 22, 'color/primary'), box({ dir: 'V', align: 'MIN', gap: 0 }, T('Selected row', 15), T('Supporting text', 12, 'color/text-muted'))));
  put('ProgressBar', box({ w: 220, h: sz('bar-thickness'), round: 999, fill: 'color/surface-container-highest' }, bar(140, sz('bar-thickness'), 'color/primary')));
  put('Tag', box({ gap: 8 }, ...[['info', 'Info'], ['success', 'Done'], ['warning', 'Wait'], ['danger', 'Error']].map(([r, s]) => box({ pad: [num('space/sm'), num('space/xs') / 2], fill: `color/${r}`, round: 999 }, T(s, num('font/size/xs'), 'color/background')))));
  put('TreeNavigation', box({ dir: 'V', align: 'MIN', gap: 0, w: 220 }, box({ w: 220, h: 32, pad: [8, 0], gap: 6 }, ico('chevron_down', 14, 'color/text-muted'), T('Designs', 14)), box({ w: 220, h: 32, pad: [28, 0], gap: 6, fill: 'color/selection', round: 6 }, T('Dodai', 14, 'color/primary')), box({ w: 220, h: 32, pad: [28, 0], gap: 6 }, T('Material', 14)), box({ w: 220, h: 32, pad: [8, 0], gap: 6 }, ico('chevron_right', 14, 'color/text-muted'), T('Settings', 14))));
  put('Column', box({ w: 260, h: 40, pad: [12, 0], fill: 'color/surface-container', stroke: 'color/border', gap: 6, main: 'SPACE_BETWEEN' }, box({ gap: 4 }, T('Name', 12, 'color/text-muted'), ico('chevron_up', 12, 'color/text-muted')), bar(2, 18, 'color/outline-variant', 1)));
  bg0.resize(1320, ky + krow + 160);
  parts2.push(`Components×${kcount}`);

  // --- Icons: every IconId of this design's pack (the one gpui resolves for it), as vectors painted
  // with the text colour variable. Dodai and Material share the Material Symbols pack.
  const iconPage = figma.createPage();
  iconPage.name = `${label} / Icons`;
  figma.currentPage = iconPage;
  const drawIcons = (page, list, cols, cellW, cellH, y0) => {
    list.forEach((ic, i) => {
      const node = figma.createNodeFromSvg(ic.svg);
      node.name = ic.id;
      paintIcon(node, ic.svg.includes('fill="none"'));
      page.appendChild(node);
      node.x = (i % cols) * cellW + (cellW - 24) / 2;
      node.y = y0 + Math.floor(i / cols) * cellH;
      const cap = text(ic.id.length > 14 ? ic.id.slice(0, 13) + '…' : ic.id, 9, 'color/text-muted');
      page.appendChild(cap);
      cap.x = (i % cols) * cellW + Math.max(0, (cellW - cap.width) / 2);
      cap.y = node.y + 30;
    });
  };
  const iconGround = figma.createRectangle();
  iconGround.name = 'Ground';
  iconGround.resize(20 * 96 + 40, Math.ceil(ICONS[packName].length / 20) * 56 + 40);
  iconGround.x = -20;
  iconGround.y = -20;
  bindFill(iconGround, 'color/background');
  iconPage.appendChild(iconGround);
  drawIcons(iconPage, ICONS[packName], 20, 96, 56, 0);
  const iconCount = ICONS[packName].length;

  // --- Symbols: the dashboard symbol groups (vehicle, medical, military, industrial, aviation).
  const symPage = figma.createPage();
  symPage.name = `${label} / Symbols`;
  figma.currentPage = symPage;
  const groupsOf = [...new Set(ICONS.symbols.map((x) => x.group))];
  const symRows = groupsOf.map((g) => Math.ceil(ICONS.symbols.filter((x) => x.group === g).length / 16));
  const symGround = figma.createRectangle();
  symGround.name = 'Ground';
  symGround.resize(16 * 96 + 40, symRows.reduce((a, r) => a + r * 56 + 56, 0) + 20);
  symGround.x = -20;
  symGround.y = -20;
  bindFill(symGround, 'color/background');
  symPage.appendChild(symGround);
  let symY = 0;
  groupsOf.forEach((g, gi) => {
    const h = text(g[0].toUpperCase() + g.slice(1), 18, 'color/text');
    symPage.appendChild(h);
    h.y = symY;
    drawIcons(symPage, ICONS.symbols.filter((x) => x.group === g), 16, 96, 56, symY + 32);
    symY += symRows[gi] * 56 + 56;
  });

  // --- Surfaces & glass: the design's frosted glass (tint, blur, saturation over the showcase
  // backdrop), and every texture and effect it owns, as the vectors the web toolkits draw.
  const SF = SURFACES[sys];
  const sfPage = figma.createPage();
  sfPage.name = `${label} / Surfaces & glass`;
  figma.currentPage = sfPage;
  const b64 = (s) => {
    const T = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/';
    const out = [];
    let acc = 0, bits = 0;
    for (const ch of s) {
      if (ch === '=') break;
      acc = (acc << 6) | T.indexOf(ch);
      bits += 6;
      if (bits >= 8) { bits -= 8; out.push((acc >> bits) & 255); }
    }
    return new Uint8Array(out);
  };
  const hexRgba = (h) => ({ r: parseInt(h.slice(1, 3), 16) / 255, g: parseInt(h.slice(3, 5), 16) / 255, b: parseInt(h.slice(5, 7), 16) / 255, a: h.length > 7 ? parseInt(h.slice(7, 9), 16) / 255 : 1 });
  const perRow = 6, tw = 160, th2 = 64, gapX = 40, gapY = 56;
  const rowsFor = (n) => Math.ceil(n / perRow);
  const sfH = 140 + 330 + 100 + (rowsFor(SF.textures.length) + rowsFor(SF.effects.length)) * (th2 + gapY) + 160;
  const sfGround = figma.createRectangle();
  sfGround.name = 'Ground';
  sfGround.resize(Math.max(perRow * (tw + gapX), 3 * 520) + 40, sfH);
  sfGround.x = -20;
  sfGround.y = -20;
  bindFill(sfGround, 'color/background');
  sfPage.appendChild(sfGround);
  const heading = (s, y) => { const t = text(s, 20, 'color/text'); sfPage.appendChild(t); t.x = 0; t.y = y; };
  heading('Frosted glass', 0);
  const note = text(`tint ${SF.frost.tint}  ·  blur ${SF.frost.blur}px  ·  saturation ${SF.frost.saturation}` + (SF.frost.edge ? `  ·  edge ${SF.frost.edge[0]} ${SF.frost.edge[1]}px` : ''), 12, 'color/text-muted');
  sfPage.appendChild(note);
  note.x = 0;
  note.y = 32;
  const scene = figma.createImage(b64(SF.scene));
  const pics = [0, SF.frost.blur / 2, SF.frost.blur].map((blur, i) => {
    const px = i * 520;
    const pic = figma.createRectangle();
    pic.name = 'Backdrop';
    pic.resize(480, 270);
    pic.fills = [{ type: 'IMAGE', imageHash: scene.hash, scaleMode: 'FILL' }];
    sfPage.appendChild(pic);
    pic.x = px;
    pic.y = 64;
    // The glass is baked (blur, saturation and tint, by the same code the toolkits run): the
    // backdrop blur effect is not drawn by every renderer, a picture is.
    const glass = figma.createFrame();
    glass.name = `Glass blur ${blur}px`;
    glass.layoutMode = 'NONE';
    glass.resize(280, 140);
    glass.clipsContent = true;
    bind(glass, 'cornerRadius', 'shape/radius');
    glass.fills = [];
    const baked = figma.createRectangle();
    baked.name = 'Blurred backdrop';
    baked.resize(480, 270);
    baked.fills = [{ type: 'IMAGE', imageHash: figma.createImage(b64(SF.panes[i].png)).hash, scaleMode: 'FILL' }];
    glass.appendChild(baked);
    baked.x = -100;
    baked.y = -66;
    if (SF.frost.edge) { glass.strokes = [{ type: 'SOLID', color: hexRgba(SF.frost.edge[0]) }]; glass.strokeWeight = SF.frost.edge[1]; }
    else { bindStroke(glass, 'color/border'); }
    const label = figma.createFrame();
    label.layoutMode = 'VERTICAL';
    label.fills = [];
    label.paddingLeft = label.paddingRight = label.paddingTop = label.paddingBottom = 16;
    label.itemSpacing = 6;
    label.appendChild(text('Glass', 18, 'color/text'));
    label.appendChild(text(`blur ${blur}px`, 13, 'color/text-muted'));
    glass.appendChild(label);
    sfPage.appendChild(glass);
    glass.x = px + 100;
    glass.y = 130;
    return pic;
  });
  let sfY = 64 + 270 + 56;
  const tiles = (title, list) => {
    heading(title, sfY);
    sfY += 40;
    list.forEach((it, i) => {
      const x = (i % perRow) * (tw + gapX);
      const y = sfY + Math.floor(i / perRow) * (th2 + gapY);
      const f = figma.createFrame();
      f.name = it.name;
      f.resize(tw, th2);
      f.clipsContent = true;
      bind(f, 'cornerRadius', 'shape/radius');
      bindFill(f, 'color/surface');
      bindStroke(f, 'color/border');
      const svg = figma.createNodeFromSvg(it.svg);
      svg.fills = [];
      f.appendChild(svg);
      svg.x = 0;
      svg.y = 0;
      sfPage.appendChild(f);
      f.x = x;
      f.y = y;
      const cap = text(it.name, 11, 'color/text-muted');
      sfPage.appendChild(cap);
      cap.x = x;
      cap.y = y + th2 + 8;
    });
    sfY += rowsFor(list.length) * (th2 + gapY) + 24;
  };
  tiles(`Textures (${SF.textures.length})`, SF.textures);
  tiles(`Effects (${SF.effects.length})`, SF.effects);

  // --- Shaders: a thumbnail of every shader of the catalogue (rendered on the GPU by the same
  // code every toolkit runs), grouped by family. Generators and post effects (over the family's
  // default source) in the family's palette.
  const SHADERS = SHADERS_BY[sys];
  const shPage = figma.createPage();
  shPage.name = `${label} / Shaders`;
  figma.currentPage = shPage;
  const shFamilies = [...new Set(SHADERS.map((x) => x.family))];
  const shCols = 6, shW = 192, shH = 108, shGapX = 40, shGapY = 64;
  const shHeight = shFamilies.reduce((a, f) => a + 56 + Math.ceil(SHADERS.filter((x) => x.family === f).length / shCols) * (shH + shGapY), 0) + 60;
  const shGround = figma.createRectangle();
  shGround.name = 'Ground';
  shGround.resize(shCols * (shW + shGapX) + 40, shHeight);
  shGround.x = -20;
  shGround.y = -20;
  bindFill(shGround, 'color/background');
  shPage.appendChild(shGround);
  let shY = 0;
  for (const fam of shFamilies) {
    const list = SHADERS.filter((x) => x.family === fam);
    const hd = text(`${fam} (${list.length})`, 20, 'color/text');
    shPage.appendChild(hd);
    hd.x = 0;
    hd.y = shY;
    shY += 44;
    list.forEach((sh, i) => {
      const x = (i % shCols) * (shW + shGapX);
      const y = shY + Math.floor(i / shCols) * (shH + shGapY);
      const img = figma.createImage(b64(sh.png));
      const tile = figma.createRectangle();
      tile.name = sh.id;
      tile.resize(shW, shH);
      bind(tile, 'cornerRadius', 'shape/radius');
      tile.fills = [{ type: 'IMAGE', imageHash: img.hash, scaleMode: 'FILL' }];
      bindStroke(tile, 'color/border');
      shPage.appendChild(tile);
      tile.x = x;
      tile.y = y;
      const cap = text(sh.name.length > 26 ? sh.name.slice(0, 25) + '…' : sh.name, 12, 'color/text');
      shPage.appendChild(cap);
      cap.x = x;
      cap.y = y + shH + 8;
      const kind = text(sh.kind, 10, 'color/text-muted');
      shPage.appendChild(kind);
      kind.x = x;
      kind.y = y + shH + 26;
    });
    shY += Math.ceil(list.length / shCols) * (shH + shGapY) + 12;
  }

  // --- Behaviour: the components that draw nothing themselves, each with what it does (the first
  // paragraph of its gpui doc comment) and the toolkits that have it.
  const bePage = figma.createPage();
  bePage.name = `${label} / Behaviour`;
  figma.currentPage = bePage;
  const beList = CATALOG.components.filter((c) => c.behaviour);
  const beGround = figma.createRectangle();
  beGround.name = 'Ground';
  beGround.resize(960, beList.length * 112 + 120);
  beGround.x = -20;
  beGround.y = -20;
  bindFill(beGround, 'color/background');
  bePage.appendChild(beGround);
  const beHead = text('Components with no visual of their own', 22, 'color/text');
  bePage.appendChild(beHead);
  let beY = 48;
  for (const c of beList) {
    cover(c.name);
    const row = figma.createFrame();
    row.name = c.name;
    row.layoutMode = 'VERTICAL';
    row.itemSpacing = 6;
    row.paddingLeft = row.paddingRight = row.paddingTop = row.paddingBottom = 16;
    row.resize(920, 96);
    row.primaryAxisSizingMode = 'AUTO';
    row.counterAxisSizingMode = 'FIXED';
    row.cornerRadius = 8;
    bindFill(row, 'color/surface');
    bindStroke(row, 'color/border');
    row.appendChild(text(c.name, 16, 'color/text'));
    const doc = text(c.doc || '-', 13, 'color/text-muted');
    doc.textAutoResize = 'HEIGHT';
    doc.resize(880, doc.height);
    row.appendChild(doc);
    row.appendChild(text(`toolkits: ${c.toolkits.join(', ')}`, 11, 'color/text-muted'));
    bePage.appendChild(row);
    row.y = beY;
    beY += row.height + 12;
  }
  beGround.resize(960, beY + 40);

  // --- Motion: the duration scale, every easing curve, every entry effect and the overscroll
  // feels, drawn from the same data every toolkit animates from (dist/design/motion.json).
  const moPage = figma.createPage();
  moPage.name = `${label} / Motion`;
  figma.currentPage = moPage;
  const moGround = figma.createRectangle();
  moGround.name = 'Ground';
  moGround.x = -20;
  moGround.y = -20;
  bindFill(moGround, 'color/background');
  moPage.appendChild(moGround);
  let my = 0;
  const mhead = (str) => { const h = text(str, 20, 'color/text'); moPage.appendChild(h); h.x = 0; h.y = my; my += 40; };
  const mrect = (w, h, fill, r = 0) => { const e = figma.createRectangle(); e.resize(w, h); e.cornerRadius = r; bindFill(e, fill); return e; };
  const mcurve = (fr, pts, stroke) => {
    const vec = figma.createVector();
    vec.vectorPaths = [{ windingRule: 'NONE', data: 'M' + pts.map((q) => `${q[0].toFixed(1)} ${q[1].toFixed(1)}`).join(' L ') }];
    vec.fills = [];
    bindStroke(vec, stroke);
    vec.strokeWeight = 2;
    vec.strokeCap = 'ROUND';
    vec.strokeJoin = 'ROUND';
    fr.appendChild(vec);
  };
  mhead('Duration scale (ms)');
  MOTION.durations.forEach((d) => {
    const lbl = text(d.name, 12, 'color/text-muted');
    moPage.appendChild(lbl); lbl.x = 0; lbl.y = my;
    const b = mrect(d.ms * 0.6, 14, 'color/primary', 3);
    moPage.appendChild(b); b.x = 110; b.y = my + 1;
    const t = text(`${d.ms}`, 12, 'color/text');
    moPage.appendChild(t); t.x = 110 + d.ms * 0.6 + 8; t.y = my;
    my += 22;
  });
  my += 28;
  mhead('Easing curves (progress over time; the faint lines are 0 and 1)');
  const cw = 170, chh = 110, lo = -0.35, hi = 1.45;
  const py = (val) => (1 - (val - lo) / (hi - lo)) * chh;
  MOTION.easings.forEach((e, i) => {
    const x = (i % 6) * 220, y = my + Math.floor(i / 6) * 190;
    const fr = figma.createFrame();
    fr.name = e.name;
    fr.layoutMode = 'NONE';
    fr.resize(cw, chh);
    bindFill(fr, 'color/surface');
    bindStroke(fr, 'color/border');
    moPage.appendChild(fr);
    fr.x = x; fr.y = y;
    for (const lvl of [0, 1]) { const l = mrect(cw, 1, 'color/outline-variant'); fr.appendChild(l); l.y = py(lvl); }
    mcurve(fr, e.points.map((val, k) => [(k / (e.points.length - 1)) * cw, py(val)]), 'color/primary');
    const cap = text(e.name, 12, 'color/text'); moPage.appendChild(cap); cap.x = x; cap.y = y + chh + 8;
    const grp = text(e.group, 10, 'color/text-muted'); moPage.appendChild(grp); grp.x = x; grp.y = y + chh + 26;
  });
  my += Math.ceil(MOTION.easings.length / 6) * 190 + 20;
  mhead('Entry effects (progress 0, 25, 50, 75 and 100 %: opacity and offset)');
  MOTION.entries.forEach((en) => {
    const lbl = text(en.name, 13, 'color/text'); moPage.appendChild(lbl); lbl.x = 0; lbl.y = my + 22;
    en.frames.forEach(([o, dx, dy], k) => {
      const cell = figma.createFrame();
      cell.layoutMode = 'NONE';
      cell.resize(110, 70);
      bindFill(cell, 'color/surface');
      bindStroke(cell, 'color/border');
      moPage.appendChild(cell);
      cell.x = 170 + k * 124; cell.y = my;
      const ghost = mrect(56, 30, 'color/surface', 6);
      bindStroke(ghost, 'color/outline-variant');
      cell.appendChild(ghost); ghost.x = 27; ghost.y = 20;
      const body = mrect(56, 30, 'color/primary-container', 6);
      body.opacity = Math.max(0, Math.min(1, o));
      cell.appendChild(body); body.x = 27 + dx; body.y = 20 + dy;
    });
    my += 86;
  });
  my += 24;
  mhead('Overscroll: the spring back after a pull (offset in px, 60 frames)');
  MOTION.overscroll.forEach((f, i) => {
    const x = i * 460;
    const fr = figma.createFrame();
    fr.name = f.name;
    fr.layoutMode = 'NONE';
    fr.resize(420, 120);
    bindFill(fr, 'color/surface');
    bindStroke(fr, 'color/border');
    moPage.appendChild(fr);
    fr.x = x; fr.y = my;
    const top = Math.max(...f.release.map(Math.abs)) || 1;
    const lvl = mrect(420, 1, 'color/outline-variant');
    fr.appendChild(lvl); lvl.y = 90;
    mcurve(fr, f.release.map((val, k) => [(k / (f.release.length - 1)) * 420, 90 - (val / top) * 80]), 'color/primary');
    const cap = text(`${f.name}: max ${f.max} px, stiffness ${f.stiffness}, damping ${f.damping}`, 12, 'color/text-muted');
    moPage.appendChild(cap); cap.x = x; cap.y = my + 128;
  });
  my += 170;
  moGround.resize(6 * 220 + 40, my + 40);

  // --- Foundations: type scale, spacing, radius and the design's own shape.
  figma.currentPage = foundations;
  const scale = figma.createFrame();
  scale.name = 'Type, space, shape';
  scale.layoutMode = 'VERTICAL';
  scale.itemSpacing = 16;
  scale.paddingLeft = scale.paddingRight = scale.paddingTop = scale.paddingBottom = 24;
  bindFill(scale, 'color/background');
  scale.x = 0;
  scale.y = 640;
  for (const n of ['xxl', 'xl', 'lg', 'md', 'sm', 'xs']) {
    const t = text(`Display ${n} — The quick brown fox`, num(`font/size/${n}`), 'color/text');
    scale.appendChild(t);
  }
  for (const n of ['xs', 'sm', 'md', 'lg', 'xl']) {
    const bar = figma.createRectangle();
    bar.name = `space/${n}`;
    bar.resize(num(`space/${n}`) * 4, 12);
    bindFill(bar, 'color/primary');
    scale.appendChild(bar);
  }
  const rad = figma.createRectangle();
  rad.name = 'shape/radius';
  rad.resize(160, 80);
  bind(rad, 'cornerRadius', 'shape/radius');
  bindFill(rad, 'color/surface-raised');
  bindStroke(rad, 'color/border-strong');
  scale.appendChild(rad);
  scale.primaryAxisSizingMode = 'AUTO';
  scale.counterAxisSizingMode = 'AUTO';
  foundations.appendChild(scale);

  // Scalar tokens of the system at its default mode: shape, motion, effect, size.
  const scalars = figma.createFrame();
  scalars.name = 'Tokens';
  scalars.layoutMode = 'VERTICAL';
  scalars.itemSpacing = 4;
  scalars.paddingLeft = scalars.paddingRight = scalars.paddingTop = scalars.paddingBottom = 24;
  bindFill(scalars, 'color/background');
  scalars.x = 700;
  scalars.y = 640;
  for (const x of inBase) {
    if (!/^(shape|motion|effect|size|font\/label)/.test(x.name)) continue;
    const val = x.valuesByMode[collection.defaultModeId];
    const hex = (c) => '#' + [c.r, c.g, c.b].map((n) => Math.round(n * 255).toString(16).padStart(2, '0')).join('') + (c.a < 1 ? Math.round(c.a * 255).toString(16).padStart(2, '0') : '');
    const shown = typeof val === 'object' && val !== null ? (val.r !== undefined ? hex(val) : val.value !== undefined ? `${val.value}${val.unit ?? ''}` : JSON.stringify(val)) : String(val);
    scalars.appendChild(text(`${x.name}  ${shown}`, 12, 'color/text-muted'));
  }
  scalars.primaryAxisSizingMode = 'AUTO';
  scalars.counterAxisSizingMode = 'AUTO';
  foundations.appendChild(scalars);

  // --- Charts: every chart of the Charts page, drawn from the showcase model's own plots
  // (dist/plots/<system>.json), so the design shows what each toolkit paints. Tile backgrounds
  // and captions are bound to the variables; the shapes carry the default theme's resolved colours.
  const charts = figma.createPage();
  charts.name = `${label} / Charts`;
  figma.currentPage = charts;
  const solid = (hex) => {
    const n = (i) => parseInt(hex.slice(i, i + 2), 16) / 255;
    return { type: 'SOLID', color: { r: n(1), g: n(3), b: n(5) }, opacity: hex.length > 7 ? n(7) : 1 };
  };
  const path = (pts, close) => 'M' + pts.map((q) => `${q[0]} ${q[1]}`).join(' L ') + (close ? ' Z' : '');
  const board = figma.createFrame();
  board.name = 'Charts';
  board.layoutMode = 'HORIZONTAL';
  board.layoutWrap = 'WRAP';
  board.itemSpacing = 24;
  board.counterAxisSpacing = 24;
  board.paddingLeft = board.paddingRight = board.paddingTop = board.paddingBottom = 24;
  bindFill(board, 'color/background');
  board.resize(1800, 100);
  board.layoutSizingVertical = 'HUG';
  let drawn = 0;
  for (const plot of PLOTS[sys] ?? []) {
    const tile = figma.createFrame();
    tile.name = `${plot.section} / ${plot.title || 'chart'}`;
    tile.layoutMode = 'VERTICAL';
    tile.itemSpacing = 8;
    tile.counterAxisSizingMode = 'AUTO';
    tile.primaryAxisSizingMode = 'AUTO';
    tile.fills = [];
    const box = figma.createFrame();
    box.name = 'plot';
    box.resize(plot.w, plot.h);
    bindFill(box, 'color/surface');
    box.clipsContent = true;
    box.layoutMode = 'NONE';
    for (const f of plot.fills) {
      if (f.points.length < 3) continue;
      const vec = figma.createVector();
      vec.vectorPaths = [{ windingRule: 'NONZERO', data: path(f.points, true) }];
      vec.fills = [solid(f.color)];
      vec.strokes = [];
      box.appendChild(vec);
    }
    for (const l of plot.lines) {
      if (l.points.length < 2) continue;
      const vec = figma.createVector();
      vec.vectorPaths = [{ windingRule: 'NONE', data: path(l.points, false) }];
      vec.fills = [];
      vec.strokes = [solid(l.color)];
      vec.strokeWeight = l.width;
      vec.strokeCap = 'ROUND';
      vec.strokeJoin = 'ROUND';
      box.appendChild(vec);
    }
    tile.appendChild(box);
    tile.appendChild(text(plot.title || plot.section, 12, 'color/text-muted'));
    board.appendChild(tile);
    drawn += 1;
  }
  charts.appendChild(board);
  // Every catalog component has a frame or a Behaviour row; every icon the frames use exists.
  for (const n of parts2) cover(n);
  for (const k of kits.keys()) cover(k);
  cover('Button');
  const absent = CATALOG.components.map((c) => c.name).filter((n) => !covered.has(n.toLowerCase()));
  if (absent.length) throw new Error(`${sys}: catalog components with no frame in the design: ${absent.join(', ')}`);
  if (missingIcons.size) throw new Error(`${sys}: icons the spec uses are not in the ${packName} pack: ${[...missingIcons].join(', ')}`);
  return { roles: roles.length, buttonVariants: comps.length, kitFamilies: kits.size, controls: parts2.length, charts: drawn, icons: iconCount, symbols: ICONS.symbols.length, textures: SF.textures.length, effects: SF.effects.length, shaders: SHADERS.length };


}

const report = {};
const names = collections.map((c) => c.name).sort();
for (const sys of names) report[sys] = buildSystem(sys, sys[0].toUpperCase() + sys.slice(1));
use(names[0]);

// --- Catalog: every component and the toolkits that have it.
const catalog = firstPage;
figma.currentPage = catalog;
catalog.name = 'Catalog';
catalog.children.forEach((c) => c.remove());
const grid = figma.createFrame();
grid.name = 'Components';
grid.layoutMode = 'HORIZONTAL';
grid.layoutWrap = 'WRAP';
grid.itemSpacing = 12;
grid.counterAxisSpacing = 12;
grid.paddingLeft = grid.paddingRight = grid.paddingTop = grid.paddingBottom = 24;
grid.resize(1400, 100);
grid.layoutSizingVertical = 'HUG';
bindFill(grid, 'color/background');
for (const c of CATALOG.components) {
  const card = figma.createFrame();
  card.name = c.name;
  card.layoutMode = 'VERTICAL';
  card.itemSpacing = 6;
  card.paddingLeft = card.paddingRight = card.paddingTop = card.paddingBottom = 12;
  card.resize(200, 80);
  card.cornerRadius = 8;
  bindFill(card, 'color/surface');
  bindStroke(card, 'color/border');
  card.appendChild(text(c.name, 15, 'color/text'));
  card.appendChild(text(`${c.toolkits.length}/${CATALOG.toolkits.length}: ${c.toolkits.join(', ')}`, 11, 'color/text-muted'));
  grid.appendChild(card);
}
catalog.appendChild(grid);
return { systems: report, components: CATALOG.components.length };
