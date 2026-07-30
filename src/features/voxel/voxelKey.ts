import type { Vec3Tuple, VoxelCoord, VoxelKey } from '../../types/domain';

export function coordToKey(coord: VoxelCoord): VoxelKey {
  assertIntegerCoordinate(coord.x, 'x');
  assertIntegerCoordinate(coord.y, 'y');
  assertIntegerCoordinate(coord.z, 'z');
  return `${coord.x},${coord.y},${coord.z}`;
}

export function keyToCoord(key: VoxelKey | string): VoxelCoord {
  const parts = key.split(',');
  if (parts.length !== 3) {
    throw new Error(`Invalid voxel key "${key}".`);
  }

  const [x, y, z] = parts.map(Number);
  assertIntegerCoordinate(x, 'x');
  assertIntegerCoordinate(y, 'y');
  assertIntegerCoordinate(z, 'z');
  return { x, y, z };
}

export function voxelCoordToWorld(
  coord: VoxelCoord,
  origin: Vec3Tuple,
  size: number,
): Vec3Tuple {
  return [
    origin[0] + coord.x * size,
    origin[1] + coord.y * size,
    origin[2] + coord.z * size,
  ];
}

function assertIntegerCoordinate(value: number, axis: string): void {
  if (!Number.isInteger(value)) {
    throw new Error(`Voxel ${axis} coordinate must be an integer.`);
  }
}
