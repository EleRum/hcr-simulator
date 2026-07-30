import { useCallback, useSyncExternalStore } from 'react';
import type {
  SimulationEngine,
  SimulationSnapshot,
} from './SimulationEngine';

export function useSimulationSnapshot(
  engine: SimulationEngine,
): SimulationSnapshot {
  const subscribe = useCallback(
    (listener: () => void) => engine.subscribe(listener),
    [engine],
  );
  const getSnapshot = useCallback(() => engine.getSnapshot(), [engine]);

  return useSyncExternalStore(subscribe, getSnapshot, getSnapshot);
}
