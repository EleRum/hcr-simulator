import { Canvas } from '@react-three/fiber';
import { ContactShadows, OrbitControls } from '@react-three/drei';
import type { SimulationEngine } from './SimulationEngine';
import { useSimulationSnapshot } from './useSimulationSnapshot';
import { RobotModel } from '../robot/RobotModel';
import { VoxelHair } from '../voxel/VoxelHair';
import { supportsWebGL } from './webglSupport';

interface SimulatorCanvasProps {
  engine: SimulationEngine;
  showTarget: boolean;
}

export function SimulatorCanvas({
  engine,
  showTarget,
}: SimulatorCanvasProps) {
  if (!supportsWebGL()) {
    return (
      <div className="webgl-fallback" role="alert">
        <strong>无法启动 3D 场景</strong>
        <span>当前浏览器未提供 WebGL，请使用最新版 Chrome 或 Edge。</span>
      </div>
    );
  }

  return (
    <div
      className="simulator-canvas"
      data-testid="simulator-canvas"
      aria-label="HCR 三维仿真场景"
    >
      <Canvas
        shadows
        dpr={[1, 1.5]}
        camera={{
          position: [3.8, 3.1, 4.8],
          fov: 42,
          near: 0.1,
          far: 50,
        }}
        gl={{ antialias: true, alpha: false }}
      >
        <SimulatorScene engine={engine} showTarget={showTarget} />
      </Canvas>
    </div>
  );
}

function SimulatorScene({
  engine,
  showTarget,
}: SimulatorCanvasProps) {
  const snapshot = useSimulationSnapshot(engine);
  const challenge = engine.getChallenge();

  return (
    <>
      <color attach="background" args={['#0a141d']} />
      <fog attach="fog" args={['#0a141d', 6, 12]} />
      <ambientLight intensity={0.78} />
      <hemisphereLight
        intensity={0.46}
        color="#bfe7ff"
        groundColor="#10171d"
      />
      <directionalLight
        castShadow
        position={[3.5, 6, 3.5]}
        intensity={2.2}
        color="#f1f7ff"
        shadow-mapSize-width={1024}
        shadow-mapSize-height={1024}
      />

      <RobotModel engine={engine} />
      <Head
        center={challenge.voxelConfig.headCenter}
        scale={challenge.voxelConfig.headScale}
      />
      <VoxelHair
        voxels={snapshot.hairVoxels}
        voxelConfig={challenge.voxelConfig}
      />
      {showTarget ? (
        <VoxelHair
          voxels={challenge.targetHair.voxels}
          voxelConfig={challenge.voxelConfig}
          variant="target"
        />
      ) : null}

      <gridHelper
        args={[12, 48, '#294454', '#172b37']}
        position={[0, 0.002, 0]}
      />
      <ContactShadows
        position={[0, 0.006, 0]}
        opacity={0.34}
        scale={7}
        blur={2.4}
        far={4}
      />
      <OrbitControls
        makeDefault
        target={[1.15, 1.25, 0]}
        minDistance={2.4}
        maxDistance={8}
        minPolarAngle={0.25}
        maxPolarAngle={Math.PI / 2.04}
        enableDamping
      />
    </>
  );
}

function Head({
  center,
  scale,
}: {
  center: readonly [number, number, number];
  scale: readonly [number, number, number];
}) {
  return (
    <group position={center}>
      <mesh castShadow receiveShadow scale={scale}>
        <sphereGeometry args={[1, 24, 18]} />
        <meshStandardMaterial color="#d2a184" roughness={0.82} />
      </mesh>
      <mesh position={[scale[0] * 0.94, 0.02, -0.2]}>
        <sphereGeometry args={[0.045, 12, 8]} />
        <meshStandardMaterial color="#19242c" />
      </mesh>
      <mesh position={[scale[0] * 0.94, 0.02, 0.2]}>
        <sphereGeometry args={[0.045, 12, 8]} />
        <meshStandardMaterial color="#19242c" />
      </mesh>
    </group>
  );
}
