const p = figma.root.children.find((p) => p.name === 'PAGE');
const o = {};
for (const c of p.children) if (c.type === 'FRAME' || c.type === 'COMPONENT') o[c.name] = c.id;
return o;
