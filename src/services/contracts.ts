import type {
  Challenge,
  ChallengeSummary,
  ScoreInput,
  ScoreResult,
} from '../types/domain';

export interface ChallengeProvider {
  listChallenges(): Promise<ChallengeSummary[]>;
  getChallenge(id: string): Promise<Challenge>;
}

export interface ScoreProvider {
  score(input: ScoreInput): Promise<ScoreResult>;
}
