import {
  Pause,
  Play,
  RotateCcw,
  SkipForward,
  Square,
} from 'lucide-react';
import type { SimulationStatus } from '../../features/simulation/SimulationEngine';

interface SimulationControlsProps {
  status: SimulationStatus;
  onRun: () => void;
  onPause: () => void;
  onResume: () => void;
  onStep: () => void;
  onStop: () => void;
  onReset: () => void;
}

export function SimulationControls({
  status,
  onRun,
  onPause,
  onResume,
  onStep,
  onStop,
  onReset,
}: SimulationControlsProps) {
  const running = status === 'running';
  const paused = status === 'paused';
  const canRun = ['idle', 'completed', 'stopped', 'error'].includes(status);

  return (
    <div className="control-dock" aria-label="仿真运行控制">
      <button
        className="control-button control-button--primary"
        type="button"
        onClick={onRun}
        disabled={!canRun}
        data-testid="run-button"
      >
        <Play size={16} fill="currentColor" />
        运行
      </button>
      {paused ? (
        <button
          className="control-button"
          type="button"
          onClick={onResume}
          data-testid="resume-button"
        >
          <Play size={16} />
          继续
        </button>
      ) : (
        <button
          className="control-button"
          type="button"
          onClick={onPause}
          disabled={!running}
          data-testid="pause-button"
        >
          <Pause size={16} />
          暂停
        </button>
      )}
      <button
        className="control-button"
        type="button"
        onClick={onStep}
        disabled={status !== 'idle' && !paused}
        data-testid="step-button"
      >
        <SkipForward size={16} />
        单步
      </button>
      <button
        className="control-button"
        type="button"
        onClick={onStop}
        disabled={!running && !paused}
        data-testid="stop-button"
      >
        <Square size={15} fill="currentColor" />
        停止
      </button>
      <span className="control-dock__divider" />
      <button
        className="control-button control-button--quiet"
        type="button"
        onClick={onReset}
        disabled={status === 'loading'}
        data-testid="reset-button"
      >
        <RotateCcw size={16} />
        重置
      </button>
    </div>
  );
}
