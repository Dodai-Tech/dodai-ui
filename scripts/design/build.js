// Run by `nu scripts/design.nu build` through `openpencil eval` (Figma Plugin API), after the
// token files are imported as variables and `CATALOG` (dist/design/components.json) is prepended.
// Everything here reads variables or the catalog; no value is written by hand.

const BASE = 'dodai'; // the collection components bind to; every other collection has the same names
const vars = figma.variables.getLocalVariables();
const collection = figma.variables.getLocalVariableCollections().find((c) => c.name === BASE);
const inBase = vars.filter((v) => v.variableCollectionId === collection.id);
const v = (name) => {
  const found = inBase.find((x) => x.name === name);
  if (!found) throw new Error(`no variable ${name}`);
  return found;
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

// --- Foundations: colour roles as swatches bound to the variables.
const foundations = figma.currentPage;
foundations.name = 'Foundations';
foundations.children.forEach((c) => c.remove());
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
buttons.name = 'Button';
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
surfaces.name = 'Surfaces';
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
    surfaces.appendChild(c);
    c.x = col * 200;
    c.y = surfaceY;
    return c;
  });
  surfaceY += 90;
  if (made.length > 1) {
    const set = figma.combineAsVariants(made, surfaces);
    set.name = base[0].toUpperCase() + base.slice(1);
    set.y = surfaceY - 90;
    surfaceSets.push(set);
  }
}

// --- Controls: sized from the size/* tokens, coloured from the roles (the gpui control geometry).
const controls = figma.createPage();
controls.name = 'Controls';
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
    const t = text('✓', n - 6, 'color/on-primary');
    c.appendChild(t);
    t.x = 3;
    t.y = 1;
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
foundations.appendChild(scale);

// --- Catalog: every component and the toolkits that have it.
const catalog = figma.createPage();
catalog.name = 'Catalog';
figma.currentPage = catalog;
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
return { roles: roles.length, buttonVariants: comps.length, kitFamilies: kits.size, controls: parts2.length, components: CATALOG.components.length };
