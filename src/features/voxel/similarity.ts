import type { VoxelKey } from '../../types/domain';

export function calculateVoxelIoU(
  target: ReadonlySet<VoxelKey>,
  result: ReadonlySet<VoxelKey>,
): number {
  if (target.size === 0 && result.size === 0) {
    return 100;
  }

  let intersectionSize = 0;
  for (const key of target) {
    if (result.has(key)) {
      intersectionSize += 1;
    }
  }

  const unionSize = target.size + result.size - intersectionSize;
  return unionSize === 0 ? 100 : (intersectionSize / unionSize) * 100;
}
