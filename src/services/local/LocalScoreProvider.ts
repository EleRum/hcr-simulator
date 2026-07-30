import { calculateScore } from '../../features/scoring/scoring';
import type { ScoreInput, ScoreResult } from '../../types/domain';
import type { ScoreProvider } from '../contracts';

export class LocalScoreProvider implements ScoreProvider {
  async score(input: ScoreInput): Promise<ScoreResult> {
    return calculateScore(input);
  }
}
