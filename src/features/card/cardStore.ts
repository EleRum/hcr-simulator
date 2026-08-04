//! Ported exactly from hcr_s4 main.rs step_and_sync:
//! Find first vertex within sphere → record its Y → deactivate all Y ≤ hit_Y.
//! In hcr_s4 Bevy Z-up, the equivalent is Z. In Three.js Y-up, it's Y.

import type { Vec3Tuple } from '../../types/domain';

export interface CardPiece {
  name: string;
  positions: Float32Array;
  originalPositions: Float32Array;
  indices: Uint32Array;
  active: boolean[];
  isCut: boolean;
}

interface CardStore {
  pieces: CardPiece[];
  cutVersion: number;
  offset: [number, number, number];
}

export const cardStore: CardStore = {
  pieces: [],
  cutVersion: 0,
  offset: [0, 0, 0],
};

export function setPieces(pieces: CardPiece[], offset: [number, number, number]) {
  cardStore.pieces = pieces;
  cardStore.offset = offset;
}

/**
 * Exact port of hcr_s4 main.rs step_and_sync (lines 155-190):
 *   for each card:
 *     find first active vertex within sphere radius of clipper position
 *     mark all vertices with Y <= hit_Y as inactive (tip = lower Y)
 * point is world-space; offset converts to card local-space.
 */
let _last = 0;
export function cutAtPoint(point: Vec3Tuple, radius: number): number {
  if (cardStore.pieces.length === 0) return 0;

  const r2 = radius * radius;
  let totalCut = 0;
  const [ox, oy, oz] = cardStore.offset;
  const px = point[0] - ox;
  const py = point[1] - oy;
  const pz = point[2] - oz;

  // Debug: every 1s
  if (Date.now() - _last > 1000) { _last = Date.now();
    let md = Infinity, mn = ''; let ac = 0;
    for (const p of cardStore.pieces) for (let vi=0;vi<p.positions.length/3;vi++) if(p.active[vi]) {
      ac++; const dx=p.positions[vi*3]-px,dy=p.positions[vi*3+1]-py,dz=p.positions[vi*3+2]-pz;
      const d=Math.sqrt(dx*dx+dy*dy+dz*dz); if(d<md){md=d;mn=p.name;}
    }
    console.log(`[cutAtPoint] world=(${point.map(v=>v.toFixed(2))}) local=(${px.toFixed(2)},${py.toFixed(2)},${pz.toFixed(2)}) r=${radius.toFixed(3)} nearest="${mn}" dist=${md.toFixed(3)} activeVerts=${ac}`); }

  for (const piece of cardStore.pieces) {
    let hitY: number | null = null;
    for (let vi = 0; vi < piece.positions.length / 3; vi++) {
      if (!piece.active[vi]) continue;
      const dx = piece.positions[vi * 3] - px;
      const dy = piece.positions[vi * 3 + 1] - py;
      const dz = piece.positions[vi * 3 + 2] - pz;
      if (dx * dx + dy * dy + dz * dz < r2) {
        hitY = piece.positions[vi * 3 + 1];
        break;
      }
    }
    if (hitY === null) continue;

    let cutCount = 0;
    for (let vi = 0; vi < piece.positions.length / 3; vi++) {
      if (piece.active[vi] && piece.positions[vi * 3 + 1] <= hitY!) {
        piece.active[vi] = false;
        cutCount++;
      }
    }
    if (cutCount > 0) { piece.isCut = true; totalCut += cutCount; }
  }

  if (totalCut > 0) cardStore.cutVersion++;
  return totalCut;
}
