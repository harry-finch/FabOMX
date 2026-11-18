/**
 * Waterjet: a cutting head above a sheet lying on the slats of the tank. The
 * part's outline is the toolpath, dashed, from a pierce point outside it. The
 * pointer is projected onto the sheet and picks the nearest point of the path;
 * the head rides the path to it on a spring, and the slot cut so far, from the
 * pierce to the jet, takes the bright stroke. At rest the cut is two-thirds
 * round. The slider is the kerf's width.
 *
 * The pattern: a continuous position on a fixed path. One spring on the
 * distance along the path, a hit test on the sheet (which never moves).
 */
const {
  Cam, circ, clamp, facing, fit, open, poly, prism, proj, rings, rrect, unproj, seg,
  spring, stepS, flatDot, mk, place, pointer, put, register, disposer, solid,
} = HL;

const PX = 150, PY = 120, PT = 3, SZ = -14, REST = 0.64;
const SLATS = [12, 41, 70, 99, 128];
// The head, bottom to top: [radius, z0, z1] above the sheet. A thin mixing tube, its nut, the body, the inlet.
const HEAD = [[2.1, 7, 24], [4.4, 24, 31], [8.5, 31, 53], [3.6, 53, 64]];

/** The cut path: a lead-in from the pierce point, then once round the part, as world points with their lengths. */
function path() {
  const n = 5, ring = rrect(42, 30, 118, 92, 17, n), k = 2 * (n + 1);
  const pts = [[24, 61], [42, 61], ...ring.slice(k).concat(ring.slice(0, k)).map((q) => [q.u, q.v]), [42, 61]];
  const at = [0];
  for (let i = 1; i < pts.length; i++) at.push(at[i - 1] + Math.hypot(pts[i][0] - pts[i - 1][0], pts[i][1] - pts[i - 1][1]));
  return { pts, at, L: at[at.length - 1] };
}

/** The point at distance s along the path, and the index of the segment it lies on. */
function pointAt({ pts, at }, s) {
  let i = 1;
  while (i < at.length - 1 && at[i] < s) i++;
  const t = clamp((s - at[i - 1]) / (at[i] - at[i - 1] || 1), 0, 1);
  return { i, p: [pts[i - 1][0] + (pts[i][0] - pts[i - 1][0]) * t, pts[i - 1][1] + (pts[i][1] - pts[i - 1][1]) * t] };
}

/** The distance along the path of its point nearest to q. */
function nearest({ pts, at }, q) {
  let best = 1e9, bs = 0;
  for (let i = 1; i < pts.length; i++) {
    const [ax, ay] = pts[i - 1], dx = pts[i][0] - ax, dy = pts[i][1] - ay, l2 = dx * dx + dy * dy || 1;
    const t = clamp(((q[0] - ax) * dx + (q[1] - ay) * dy) / l2, 0, 1);
    const d = Math.hypot(ax + dx * t - q[0], ay + dy * t - q[1]);
    if (d < best) { best = d; bs = at[i - 1] + t * (at[i] - at[i - 1]); }
  }
  return bs;
}

/** The slot from the pierce to s, w wide, as one closed outline of world points. */
function slot(W, s, w) {
  const { i, p } = pointAt(W, s), line = W.pts.slice(0, i).concat([p]);
  const nrm = (a, b) => { const l = Math.hypot(b[0] - a[0], b[1] - a[1]) || 1; return [-(b[1] - a[1]) / l, (b[0] - a[0]) / l]; };
  const left = [], right = [];
  line.forEach((q, j) => {
    const a = nrm(line[Math.max(0, j - 1)], line[Math.max(1, j)]), b = nrm(line[Math.min(j, line.length - 2)], line[Math.min(j + 1, line.length - 1)]);
    let nx = a[0] + b[0], ny = a[1] + b[1];
    const l = Math.hypot(nx, ny) || 1;
    nx /= l; ny /= l;
    left.push([q[0] + nx * w / 2, q[1] + ny * w / 2]);
    right.push([q[0] - nx * w / 2, q[1] - ny * w / 2]);
  });
  return left.concat(right.reverse());
}

function mount({ stage, svg, read }, value) {
  const bag = disposer();
  const C = Cam(45, 0.5, 1.7);
  const W = path(), top = HEAD[HEAD.length - 1][2] + PT;
  fit(C, [[0, -10, SZ], [PX, -10, SZ], [0, PY + 10, SZ], [PX, PY + 10, SZ], ...W.pts.map((q) => [q[0], q[1], top])], 200, 166);
  const P = proj(C), front = facing(C);
  let kerf = value, over = null, drawn = NaN;
  const sp = spring(W.L * REST, { eps: 0.05 });

  const g = mk("g", {}, svg);
  // The slats run the whole depth of the tank, under the sheet, so they are painted first.
  for (const x of SLATS) {
    const [r, ri] = rings(x - 1.6, -10, x + 1.6, PY + 10, 1, 0.6);
    put(solid(g), prism(P, front, r, ri, SZ, 0));
  }
  const [pr, pi] = rings(0, 0, PX, PY, 6, 1.8);
  put(solid(g), prism(P, front, pr, pi, 0, PT));

  const on = (q) => P(q[0], q[1], PT);
  mk("path", { class: "dash nf", d: open(W.pts.map(on)) }, g);
  const cut = mk("path", { class: "hi" }, g);
  const pierce = flatDot(g, C, 1.2, "dot m");
  place(pierce, on(W.pts[0]));
  const jet = mk("path", { class: "hi nf" }, g);
  const head = HEAD.map(() => solid(g));
  const shells = HEAD.map(([R]) => [circ(R, 32), circ(R - 0.7, 32)]);

  function draw() {
    const s = sp.x;
    if (s === drawn) return;
    drawn = s;
    const { p } = pointAt(W, s);
    cut.setAttribute("d", poly(slot(W, Math.max(0.5, s), kerf).map(on)));
    jet.setAttribute("d", seg(P(p[0], p[1], PT), P(p[0], p[1], PT + HEAD[0][1])));
    const move = (ring) => ring.map((q) => ({ ...q, u: q.u + p[0], v: q.v + p[1] }));
    HEAD.forEach(([, z0, z1], k) => put(head[k], prism(P, front, move(shells[k][0]), move(shells[k][1]), PT + z0, PT + z1)));
  }

  const B = register(stage, (dt) => {
    const m = stepS(sp, dt);
    draw();
    return m;
  });
  bag.add(B.unregister);

  function retarget() {
    if (over) {
      sp.t = nearest(W, over);
      read.textContent = `cut ${Math.round((100 * sp.t) / W.L)}%`;
    } else { sp.t = W.L * REST; read.textContent = "rest"; }
    B.wake();
  }

  bag.add(pointer(stage, {
    move: (q) => { over = unproj(C, q[0], q[1], PT); retarget(); },
    leave: () => { over = null; retarget(); },
  }));
  bag.add(() => svg.replaceChildren());

  return {
    set: (v) => { kerf = v; drawn = NaN; B.wake(); },
    destroy: bag.dispose,
  };
}

hairline({
  name: "waterjet",
  means: "A waterjet head rides the toolpath round a part; the slot cut so far follows the pointer.",
  rules: [1, 3, 5, 8],
  range: [1.6, 3, 4.6],
  tour: [[242, 158], [141, 182], [163, 152], null],
  mount,
});
