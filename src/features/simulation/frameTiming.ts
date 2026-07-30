const MAX_FRAME_DELTA_MS = 100;

export function clampFrameDeltaMs(deltaSeconds: number): number {
  return Math.min(
    Math.max(deltaSeconds * 1_000, 0),
    MAX_FRAME_DELTA_MS,
  );
}
