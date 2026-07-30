import { useLayoutEffect, useMemo, useRef } from 'react';
import { Matrix4, type InstancedMesh } from 'three';
import type { Challenge, VoxelKey } from '../../types/domain';
import { keyToCoord, voxelCoordToWorld } from './voxelKey';

interface VoxelHairProps {
  voxels: ReadonlySet<VoxelKey>;
  voxelConfig: Challenge['voxelConfig'];
  variant?: 'current' | 'target';
}

export function VoxelHair({
  voxels,
  voxelConfig,
  variant = 'current',
}: VoxelHairProps) {
  const positions = useMemo(
    () =>
      [...voxels].map((key) => ({
        key,
        position: voxelCoordToWorld(
          keyToCoord(key),
          voxelConfig.origin,
          voxelConfig.size,
        ),
      })),
    [voxelConfig.origin, voxelConfig.size, voxels],
  );
  const isTarget = variant === 'target';
  const size = voxelConfig.size * (isTarget ? 1.035 : 0.94);
  const meshRef = useRef<InstancedMesh>(null);

  useLayoutEffect(() => {
    const mesh = meshRef.current;
    if (!mesh) {
      return;
    }

    const matrix = new Matrix4();
    positions.forEach(({ position }, index) => {
      matrix.makeTranslation(...position);
      mesh.setMatrixAt(index, matrix);
    });
    mesh.instanceMatrix.needsUpdate = true;
    mesh.computeBoundingSphere();
  }, [positions]);

  return (
    <instancedMesh
      key={`${variant}-${positions.length}`}
      ref={meshRef}
      args={[undefined, undefined, positions.length]}
      castShadow={false}
      receiveShadow={false}
      frustumCulled={false}
    >
      <boxGeometry args={[size, size, size]} />
      <meshStandardMaterial
        color={isTarget ? '#64d9e8' : '#8b4f35'}
        emissive={isTarget ? '#2b9fac' : '#241108'}
        emissiveIntensity={isTarget ? 0.18 : 0.08}
        metalness={isTarget ? 0.05 : 0}
        roughness={isTarget ? 0.42 : 0.78}
        transparent={isTarget}
        opacity={isTarget ? 0.18 : 1}
        depthWrite={!isTarget}
      />
    </instancedMesh>
  );
}
