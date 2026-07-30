import { useFrame } from '@react-three/fiber';
import type { SimulationEngine } from './SimulationEngine';
import { clampFrameDeltaMs } from './frameTiming';

export function SimulationTicker({
  engine,
}: {
  engine: SimulationEngine;
}) {
  useFrame((_, deltaSeconds) => {
    engine.tick(clampFrameDeltaMs(deltaSeconds));
  });

  return null;
}
