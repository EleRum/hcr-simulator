//! Card vertex contact detection — replaces voxel AABB sweep with vertex-sphere test.
//! Ported from hcr_s4 main.rs step_and_sync + HairCard::tipmost_row_in_sphere.

import type { Vec3Tuple } from '../../types/domain';

/** One vertex on a hair card. */
export interface CardVertexState {
  position: Vec3Tuple;
  active: boolean;
}

/** One hair card = rows of vertex columns. */
export interface HairCardState {
  name: string;
  layer: number;
  /** vertices[row][col] */
  rows: CardVertexState[][];
  isCut: boolean;
  /** Original indices for mesh rebuilding */
  originalIndices: Uint32Array;
  originalPositions: Float32Array;
}

/**
 * Find card vertices within `radius` of the line segment from `start` to `end`.
 * Returns array of {cardIndex, rowIndex, colIndex} for vertices to deactivate.
 */
export function findSweptCardVertexHits(
  start: Vec3Tuple,
  end: Vec3Tuple,
  cards: HairCardState[],
  radius: number,
): Array<{ card: HairCardState; row: number; col: number }> {
  const hits: Array<{ card: HairCardState; row: number; col: number }> = [];
  const r2 = radius * radius;
  const steps = Math.max(1, Math.ceil(distance(start, end) / (radius * 0.5)));

  for (const card of cards) {
    for (let ri = 0; ri < card.rows.length; ri++) {
      for (let ci = 0; ci < card.rows[ri].length; ci++) {
        const v = card.rows[ri][ci];
        if (!v.active) continue;

        // Check multiple points along the segment
        for (let s = 0; s <= steps; s++) {
          const t = s / steps;
          const px = start[0] + (end[0] - start[0]) * t;
          const py = start[1] + (end[1] - start[1]) * t;
          const pz = start[2] + (end[2] - start[2]) * t;
          const dx = v.position[0] - px;
          const dy = v.position[1] - py;
          const dz = v.position[2] - pz;
          if (dx * dx + dy * dy + dz * dz < r2) {
            // Find the tipmost (highest row index) per card
            // Mark from this row downward as cut
            if (ri >= 1 && !hits.some(h => h.card === card && h.row <= ri)) {
              hits.push({ card, row: ri, col: ci });
            }
            break;
          }
        }
      }
    }
  }

  return hits;
}

function distance(a: Vec3Tuple, b: Vec3Tuple): number {
  const dx = a[0] - b[0];
  const dy = a[1] - b[1];
  const dz = a[2] - b[2];
  return Math.sqrt(dx * dx + dy * dy + dz * dz);
}
