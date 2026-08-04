//! Realistic head model loaded from OBJ (hcr_s4 Blender head).
//! Renders as an alternative to the simple ellipsoid sphere head.

import { useEffect, useMemo } from 'react';
import { useLoader } from '@react-three/fiber';
import { OBJLoader } from 'three/examples/jsm/loaders/OBJLoader.js';
import { MeshStandardMaterial, type Mesh, type BufferGeometry } from 'three';
import type { Vec3Tuple } from '../../types/domain';

interface RealisticHeadProps {
  center: Vec3Tuple;
  scale: number;
  visible: boolean;
}

export function RealisticHead({ center, scale, visible }: RealisticHeadProps) {
  const obj = useLoader(OBJLoader, '/head.obj');

  // Apply double-sided skin material to all child meshes
  useEffect(() => {
    obj.traverse((child) => {
      if ((child as Mesh).isMesh) {
        const mesh = child as Mesh;
        mesh.material = new MeshStandardMaterial({
          color: '#d2a184',
          roughness: 0.75,
          metalness: 0.02,
          side: 2, // DoubleSide
        });
      }
    });
  }, [obj]);

  const geometry = useMemo(() => {
    // OBJ may have multiple child meshes — merge into one geometry
    const geos: BufferGeometry[] = [];
    obj.traverse((child) => {
      if ((child as Mesh).isMesh) {
        const mesh = child as Mesh;
        mesh.updateMatrixWorld();
        const cloned = mesh.geometry.clone();
        cloned.applyMatrix4(mesh.matrixWorld);
        geos.push(cloned);
      }
    });
    if (geos.length === 0) return null;
    // Return the first geometry if only one, otherwise we'd need BufferGeometryUtils.mergeGeometries
    // For now, we'll render each child mesh individually via the group
    return geos.length === 1 ? geos[0] : null;
  }, [obj]);

  if (!visible) return null;

  // If single merged geometry
  if (geometry) {
    return (
      <group position={center} scale={[scale, scale, scale]}>
        <mesh
          geometry={geometry}
          frustumCulled
        >
          <meshStandardMaterial
            color="#d2a184"
            roughness={0.75}
            metalness={0.02}
          />
        </mesh>
      </group>
    );
  }

  // Multiple child meshes — render the group directly
  return (
    <group
      position={center}
      scale={[scale, scale, scale]}
      visible={visible}
      frustumCulled
    >
      <primitive object={obj} />
    </group>
  );
}
