import { Eye, EyeOff, Target } from 'lucide-react';
import type { Challenge } from '../../types/domain';
import type { SimulationSnapshot } from '../../features/simulation/SimulationEngine';

interface InspectorPanelProps {
  challenge: Challenge;
  snapshot: SimulationSnapshot;
  showTarget: boolean;
  onToggleTarget: () => void;
}

const STATUS_LABELS: Record<SimulationSnapshot['status'], string> = {
  loading: '加载中',
  idle: '待机',
  running: '运行中',
  paused: '已暂停',
  completed: '已完成',
  stopped: '已停止',
  error: '错误',
};

export function InspectorPanel({
  challenge,
  snapshot,
  showTarget,
  onToggleTarget,
}: InspectorPanelProps) {
  const result = snapshot.scoreResult;

  return (
    <div className="inspector">
      <section className="inspector-section challenge-card">
        <div className="section-heading">
          <span>CHALLENGE 01</span>
          <span
            className={`status-pill status-pill--${snapshot.status}`}
            data-testid="simulation-status"
          >
            {STATUS_LABELS[snapshot.status]}
          </span>
        </div>
        <h2>{challenge.name}</h2>
        <p>{challenge.description}</p>
        <button
          type="button"
          className={`target-toggle ${showTarget ? 'is-active' : ''}`}
          onClick={onToggleTarget}
          aria-pressed={showTarget}
        >
          {showTarget ? <Eye size={15} /> : <EyeOff size={15} />}
          目标发型预览
          <span>{showTarget ? 'ON' : 'OFF'}</span>
        </button>
      </section>

      <section className="inspector-section">
        <div className="section-heading">
          <span>JOINT TELEMETRY</span>
          <span>DEG</span>
        </div>
        <div className="joint-list">
          {challenge.robotConfig.joints.map((joint) => (
            <div
              className={`joint-row ${
                snapshot.activeJointId === joint.id ? 'is-active' : ''
              }`}
              key={joint.id}
            >
              <div>
                <strong>{joint.name}</strong>
                <small>{joint.id}</small>
              </div>
              <output>{formatNumber(snapshot.jointAngles[joint.id], 1)}°</output>
            </div>
          ))}
        </div>
        <div className="coordinate-readout">
          <span>END EFFECTOR</span>
          <div>
            {(['X', 'Y', 'Z'] as const).map((axis, index) => (
              <span key={axis}>
                <small>{axis}</small>
                {formatNumber(snapshot.endEffector[index], 2)}
              </span>
            ))}
          </div>
        </div>
      </section>

      <section className="inspector-section">
        <div className="section-heading">
          <span>RUN METRICS</span>
          <span>LIVE</span>
        </div>
        <div className="metric-grid">
          <Metric
            label="当前 Voxel"
            value={snapshot.hairVoxels.size}
            testId="current-voxel-count"
          />
          <Metric label="目标 Voxel" value={snapshot.targetVoxelCount} />
          <Metric
            label="源积木"
            value={snapshot.metrics.sourceBlockCount}
            testId="source-block-count"
          />
          <Metric
            label="已执行命令"
            value={snapshot.metrics.executedCommandCount}
            testId="executed-command-count"
          />
        </div>
        <div className="duration-row">
          <span>估算执行时间</span>
          <strong>
            {(snapshot.metrics.estimatedDurationMs / 1_000).toFixed(2)}s
          </strong>
        </div>
      </section>

      <section className="inspector-section result-section">
        <div className="section-heading">
          <span>SCORE BREAKDOWN</span>
          <Target size={14} />
        </div>
        {result ? (
          <>
            <div className="final-score">
              <span>FINAL SCORE</span>
              <strong data-testid="final-score">
                {result.finalScore.toFixed(1)}
              </strong>
              <small>/ 100</small>
            </div>
            <div className="score-bars">
              <ScoreBar
                label="完成度"
                score={result.completionScore}
                testId="completion-score"
              />
              <ScoreBar label="程序效率" score={result.efficiencyScore} />
              <ScoreBar label="时间" score={result.timeScore} />
            </div>
          </>
        ) : (
          <div className="result-placeholder">
            {snapshot.status === 'stopped'
              ? '已停止：当前指标为临时数据，不生成正式成绩。'
              : '程序自然结束后将在此显示正式成绩。'}
          </div>
        )}
      </section>
    </div>
  );
}

function Metric({
  label,
  value,
  testId,
}: {
  label: string;
  value: number;
  testId?: string;
}) {
  return (
    <div className="metric-card">
      <span>{label}</span>
      <strong data-testid={testId}>{value}</strong>
    </div>
  );
}

function ScoreBar({
  label,
  score,
  testId,
}: {
  label: string;
  score: number;
  testId?: string;
}) {
  return (
    <div className="score-row">
      <div>
        <span>{label}</span>
        <strong data-testid={testId}>{score.toFixed(1)}</strong>
      </div>
      <div className="score-track">
        <span style={{ width: `${Math.max(0, Math.min(100, score))}%` }} />
      </div>
    </div>
  );
}

function formatNumber(value: number | undefined, digits: number): string {
  return Number.isFinite(value) ? Number(value).toFixed(digits) : '—';
}
