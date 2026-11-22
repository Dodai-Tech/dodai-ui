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
    c.primaryAxisAlignItems = 'CENTER';
    c.counterAxisAlignItems = 'CENTER';
    c.paddingLeft = c.paddingRight = 20;
    c.paddingTop = c.paddingBottom = 12;
    c.cornerRadius = 12;
    c.strokeWeight = 1;
    bindFill(c, `button/${variant}-${state}-bg`);
    bindStroke(c, `button/${variant}-${state}-border`);
    c.appendChild(text('Button', 15, `button/${variant}-${state}-fg`));
    buttons.appendChild(c);
    c.x = col * 140;
    c.y = row * 64;
    comps.push(c);
  });
});
const set = figma.combineAsVariants(comps, buttons);
set.name = 'Button';

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
return { roles: roles.length, buttonVariants: comps.length, components: CATALOG.components.length };
