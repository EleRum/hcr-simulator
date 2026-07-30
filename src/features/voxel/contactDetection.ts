import type {
  Challenge,
  Vec3Tuple,
  VoxelKey,
} from '../../types/domain';
import { keyToCoord, voxelCoordToWorld } from './voxelKey';

export function findSweptVoxelHits(
  start: Vec3Tuple,
  end: Vec3Tuple,
  voxels: ReadonlySet<VoxelKey>,
  voxelConfig: Challenge['voxelConfig'],
  sphereRadius: number,
): VoxelKey[] {
  const hits: VoxelKey[] = [];
  const halfVoxel = voxelConfig.size / 2;

  for (const key of voxels) {
    const center = voxelCoordToWorld(
      keyToCoord(key),
      voxelConfig.origin,
      voxelConfig.size,
    );
    const expansion = halfVoxel + sphereRadius;
    const min: Vec3Tuple = [
      center[0] - expansion,
      center[1] - expansion,
      center[2] - expansion,
    ];
    const max: Vec3Tuple = [
      center[0] + expansion,
      center[1] + expansion,
      center[2] + expansion,
    ];

    if (segmentIntersectsAabb(start, end, min, max)) {
      hits.push(key);
    }
  }

  return hits;
}

export function segmentIntersectsAabb(
  start: Vec3Tuple,
  end: Vec3Tuple,
  min: Vec3Tuple,
  max: Vec3Tuple,
): boolean {
  let minimumT = 0;
  let maximumT = 1;

  for (let axis = 0; axis < 3; axis += 1) {
    const direction = end[axis] - start[axis];
    if (Math.abs(direction) < Number.EPSILON) {
      if (start[axis] < min[axis] || start[axis] > max[axis]) {
        return false;
      }
      continue;
    }

    const inverse = 1 / direction;
    let firstT = (min[axis] - start[axis]) * inverse;
    let secondT = (max[axis] - start[axis]) * inverse;
    if (firstT > secondT) {
      [firstT, secondT] = [secondT, firstT];
    }
    minimumT = Math.max(minimumT, firstT);
    maximumT = Math.min(maximumT, secondT);
    if (minimumT > maximumT) {
      return false;
    }
  }

  return true;
}
