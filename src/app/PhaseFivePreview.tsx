import { useEffect, useState } from 'react';
import {
  DEFAULT_CHALLENGE_ID,
} from '../data/challenges/defaultChallenge';
import { SimulatorCanvas } from '../features/simulation/SimulatorCanvas';
import { SimulationEngine } from '../features/simulation/SimulationEngine';
import { LocalChallengeProvider } from '../services/local/LocalChallengeProvider';
import { LocalScoreProvider } from '../services/local/LocalScoreProvider';

const challengeProvider = new LocalChallengeProvider();
const scoreProvider = new LocalScoreProvider();

export function PhaseFivePreview() {
  const [engine, setEngine] = useState<SimulationEngine>();
  const [error, setError] = useState<string>();

  useEffect(() => {
    let active = true;

    void challengeProvider
      .getChallenge(DEFAULT_CHALLENGE_ID)
      .then((challenge) => {
        if (active) {
          setEngine(new SimulationEngine(challenge, scoreProvider));
        }
      })
      .catch((reason: unknown) => {
        if (active) {
          setError(
            reason instanceof Error
              ? reason.message
              : 'Challenge 加载失败。',
          );
        }
      });

    return () => {
      active = false;
    };
  }, []);

  if (error) {
    return (
      <main className="phase-shell">
        <p className="phase-kicker">HCR / LOAD ERROR</p>
        <h1>Challenge 无法加载</h1>
        <p>{error}</p>
      </main>
    );
  }

  if (!engine) {
    return (
      <main className="phase-shell">
        <p className="phase-kicker">HCR / LOCAL SIMULATION</p>
        <h1>HCR Simulator</h1>
        <p>正在加载本地 Challenge…</p>
      </main>
    );
  }

  return (
    <main className="phase-five-shell">
      <header className="phase-five-header">
        <div>
          <p className="phase-kicker">HCR / LOCAL SIMULATION</p>
          <h1>HCR Simulator</h1>
        </div>
        <span>PHASE 5 · 3D SCENE</span>
      </header>
      <SimulatorCanvas engine={engine} showTarget />
    </main>
  );
}
