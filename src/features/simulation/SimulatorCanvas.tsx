import {
  useCallback,
  useEffect,
  useRef,
  useState,
} from 'react';
import { Canvas, useThree } from '@react-three/fiber';
import { OrbitControls } from '@react-three/drei';
import { AlertTriangle, LoaderCircle, RotateCcw } from 'lucide-react';
import type { WebGLRenderer } from 'three';
import type { SimulationEngine } from './SimulationEngine';
import { useSimulationSnapshot } from './useSimulationSnapshot';
import { SimulationTicker } from './SimulationTicker';
import { RobotModel } from '../robot/RobotModel';
import { VoxelHair } from '../voxel/VoxelHair';
import { RealisticHead } from '../head/RealisticHead';
import { CardHair } from '../card/CardHair';
import { supportsWebGL } from './webglSupport';

interface SimulatorCanvasProps {
  engine: SimulationEngine;
  showTarget: boolean;
  realisticHead?: boolean;
}

export type SceneRenderState =
  | 'ready'
  | 'context-lost'
  | 'recovering';

export function SimulatorCanvas({
  engine,
  showTarget,
  realisticHead = false,
}: SimulatorCanvasProps) {
  const [webglSupported] = useState(supportsWebGL);
  const [renderState, setRenderState] =
    useState<SceneRenderState>('recovering');
  const [canvasGeneration, setCanvasGeneration] = useState(0);
  const resumeAfterRecoveryRef = useRef(false);

  const reinitializeCanvas = useCallback(() => {
    setRenderState('recovering');
    setCanvasGeneration((generation) => generation + 1);
  }, []);

  const handleContextLost = useCallback(() => {
    if (engine.getSnapshot().status === 'running') {
      resumeAfterRecoveryRef.current = true;
      engine.pause();
    } else {
      resumeAfterRecoveryRef.current = false;
    }
    setRenderState('context-lost');
  }, [engine]);

  const handleCanvasCreated = useCallback(
    (gl: WebGLRenderer) => {
      gl.setClearColor('#0a141d', 1);
      setRenderState('ready');

      if (resumeAfterRecoveryRef.current) {
        resumeAfterRecoveryRef.current = false;
        engine.resume();
      }
    },
    [engine],
  );

  if (!webglSupported) {
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
      data-render-state={renderState}
      aria-label="HCR 三维仿真场景"
    >
      <Canvas
        key={canvasGeneration}
        shadows="basic"
        dpr={[1, 1.25]}
        camera={{
          position: [3.8, 3.1, 4.8],
          fov: 42,
          near: 0.1,
          far: 50,
        }}
        gl={{
          antialias: true,
          alpha: false,
        }}
        onCreated={({ gl }) => handleCanvasCreated(gl)}
      >
        <WebGLContextGuard
          onContextLost={handleContextLost}
          onContextRestored={reinitializeCanvas}
        />
        <SimulatorScene engine={engine} showTarget={showTarget} realisticHead={realisticHead} />
      </Canvas>
      {renderState !== 'ready' ? (
        <div
          className="scene-status-overlay"
          role={renderState === 'context-lost' ? 'alert' : 'status'}
        >
          {renderState === 'context-lost' ? (
            <>
              <AlertTriangle size={24} />
              <strong>3D 渲染已中断</strong>
              <span>
                WebGL 上下文已丢失，仿真已安全暂停。恢复场景后将自动继续。
              </span>
              <button type="button" onClick={reinitializeCanvas}>
                <RotateCcw size={15} />
                重新初始化 3D
              </button>
            </>
          ) : (
            <>
              <LoaderCircle className="spin" size={22} />
              <strong>正在初始化 3D 场景</strong>
            </>
          )}
        </div>
      ) : null}
    </div>
  );
}

interface SimulatorSceneProps {
  engine: SimulationEngine;
  showTarget: boolean;
  realisticHead?: boolean;
}

function SimulatorScene({
  engine,
  showTarget,
  realisticHead = false,
}: SimulatorSceneProps) {
  const snapshot = useSimulationSnapshot(engine);
  const challenge = engine.getChallenge();

  return (
    <>
      <SimulationTicker engine={engine} />
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

      <RobotModel
        engine={engine}
        activeJointId={snapshot.activeJointId}
      />
      <Head
        center={challenge.voxelConfig.headCenter}
        scale={challenge.voxelConfig.headScale}
        visible={!realisticHead}
      />
      <RealisticHead
        center={challenge.voxelConfig.realisticHeadCenter}
        scale={challenge.voxelConfig.realisticHeadScale[0]}
        visible={realisticHead}
      />
      {!realisticHead && (
        <VoxelHair
          voxels={snapshot.hairVoxels}
          voxelConfig={challenge.voxelConfig}
        />
      )}
      {!realisticHead && showTarget ? (
        <VoxelHair
          voxels={challenge.targetHair.voxels}
          voxelConfig={challenge.voxelConfig}
          variant="target"
        />
      ) : null}
      {realisticHead && <CardHair center={challenge.voxelConfig.cardHairCenter} scale={challenge.voxelConfig.cardHairScale} />}

      <gridHelper
        args={[12, 48, '#294454', '#172b37']}
        position={[0, 0.002, 0]}
      />
      <mesh
        receiveShadow
        rotation={[-Math.PI / 2, 0, 0]}
        position={[0, -0.012, 0]}
      >
        <planeGeometry args={[12, 12]} />
        <meshStandardMaterial color="#0a141d" roughness={1} />
      </mesh>
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

function WebGLContextGuard({
  onContextLost,
  onContextRestored,
}: {
  onContextLost: () => void;
  onContextRestored: () => void;
}) {
  const gl = useThree((state) => state.gl) as WebGLRenderer;

  useEffect(() => {
    const canvas = gl.domElement;
    const handleLost = (event: Event) => {
      event.preventDefault();
      onContextLost();
    };
    const handleRestored = () => onContextRestored();

    canvas.addEventListener('webglcontextlost', handleLost);
    canvas.addEventListener('webglcontextrestored', handleRestored);

    return () => {
      canvas.removeEventListener('webglcontextlost', handleLost);
      canvas.removeEventListener(
        'webglcontextrestored',
        handleRestored,
      );
    };
  }, [gl, onContextLost, onContextRestored]);

  return null;
}

function Head({
  center,
  scale,
  visible = true,
}: {
  center: readonly [number, number, number];
  scale: readonly [number, number, number];
  visible?: boolean;
}) {
  return (
    <group position={center} visible={visible}>
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
