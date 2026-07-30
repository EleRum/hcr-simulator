import { useEffect, useMemo, useState } from 'react';
import { AlertTriangle, LoaderCircle, RotateCcw } from 'lucide-react';
import type { Challenge } from '../types/domain';
import { SimulationEngine } from '../features/simulation/SimulationEngine';
import { SimulationWorkbench } from '../components/layout/SimulationWorkbench';
import { useServices } from './servicesContext';

export function WorkbenchBootstrap() {
  const { challengeProvider, scoreProvider } = useServices();
  const [challenge, setChallenge] = useState<Challenge>();
  const [error, setError] = useState<string>();
  const [retryToken, setRetryToken] = useState(0);

  useEffect(() => {
    let active = true;

    void challengeProvider
      .listChallenges()
      .then((summaries) => {
        const first = summaries[0];
        if (!first) {
          throw new Error('本地 Challenge 列表为空。');
        }
        return challengeProvider.getChallenge(first.id);
      })
      .then((loadedChallenge) => {
        if (active) {
          setChallenge(loadedChallenge);
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
  }, [challengeProvider, retryToken]);

  const engine = useMemo(
    () =>
      challenge
        ? new SimulationEngine(challenge, scoreProvider)
        : undefined,
    [challenge, scoreProvider],
  );

  if (error) {
    return (
      <main className="bootstrap-screen" role="alert">
        <AlertTriangle size={30} />
        <p className="phase-kicker">HCR / PROVIDER ERROR</p>
        <h1>Challenge 无法加载</h1>
        <p>{error}</p>
        <button
          type="button"
          onClick={() => {
            setChallenge(undefined);
            setError(undefined);
            setRetryToken((token) => token + 1);
          }}
        >
          <RotateCcw size={16} />
          重试
        </button>
      </main>
    );
  }

  if (!challenge || !engine) {
    return (
      <main className="bootstrap-screen">
        <LoaderCircle className="spin" size={30} />
        <p className="phase-kicker">HCR / LOCAL PROVIDER</p>
        <h1>HCR Simulator</h1>
        <p>正在加载本地 Challenge 与仿真引擎…</p>
      </main>
    );
  }

  return <SimulationWorkbench challenge={challenge} engine={engine} />;
}
