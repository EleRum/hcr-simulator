import type { HairstyleDefinition, VoxelCoord } from '../../types/domain';
import { coordToKey } from './voxelKey';

const TARGET_INNER_BOUND = 0.68;
const TARGET_OUTER_BOUND = 1.18;
const INITIAL_OUTER_BOUND = 1.24;

export interface GeneratedHairstyles {
  initialHair: HairstyleDefinition;
  targetHair: HairstyleDefinition;
}

export function generateDefaultHairstyles(): GeneratedHairstyles {
  const target = generateShell(TARGET_INNER_BOUND, TARGET_OUTER_BOUND, -1);
  const initial = generateShell(TARGET_INNER_BOUND, INITIAL_OUTER_BOUND, -2);

  return {
    initialHair: {
      id: 'thick-cap',
      name: '厚帽型初始发型',
      voxels: initial,
    },
    targetHair: {
      id: 'neat-short-cap',
      name: '对称整齐短发',
      voxels: target,
    },
  };
}

function generateShell(
  innerBound: number,
  outerBound: number,
  minimumY: number,
): VoxelCoord[] {
  const voxels: VoxelCoord[] = [];

  for (let x = -6; x <= 6; x += 1) {
    for (let y = minimumY; y <= 7; y += 1) {
      for (let z = -6; z <= 6; z += 1) {
        const normalized =
          (x * x) / (4.2 * 4.2) +
          (y * y) / (5.2 * 5.2) +
          (z * z) / (4.2 * 4.2);

        if (normalized >= innerBound && normalized <= outerBound) {
          voxels.push({ x, y, z });
        }
      }
    }
  }

  return voxels.sort(compareCoords);
}

function compareCoords(a: VoxelCoord, b: VoxelCoord): number {
  return a.x - b.x || a.y - b.y || a.z - b.z;
}

export function hasDuplicateVoxels(voxels: readonly VoxelCoord[]): boolean {
  return new Set(voxels.map(coordToKey)).size !== voxels.length;
}

export function isSymmetricAcrossZ(voxels: readonly VoxelCoord[]): boolean {
  const keys = new Set(voxels.map(coordToKey));
  return voxels.every((voxel) =>
    keys.has(coordToKey({ ...voxel, z: -voxel.z })),
  );
}
