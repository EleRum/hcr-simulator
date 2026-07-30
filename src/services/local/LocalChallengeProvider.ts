import { defaultChallengeDefinition } from '../../data/challenges/defaultChallenge';
import { coordToKey } from '../../features/voxel/voxelKey';
import type {
  Challenge,
  ChallengeDefinition,
  ChallengeSummary,
} from '../../types/domain';
import type { ChallengeProvider } from '../contracts';
import { validateChallengeDefinition } from '../validation';

export class LocalChallengeProvider implements ChallengeProvider {
  private readonly definitions: readonly ChallengeDefinition[];

  constructor(
    definitions: readonly ChallengeDefinition[] = [
      defaultChallengeDefinition,
    ],
  ) {
    definitions.forEach(validateChallengeDefinition);
    this.definitions = definitions;
  }

  async listChallenges(): Promise<ChallengeSummary[]> {
    return this.definitions.map(({ id, name, description }) => ({
      id,
      name,
      description,
    }));
  }

  async getChallenge(id: string): Promise<Challenge> {
    const definition = this.definitions.find((item) => item.id === id);
    if (!definition) {
      throw new Error(`Challenge "${id}" was not found.`);
    }

    return normalizeChallenge(definition);
  }
}

function normalizeChallenge(definition: ChallengeDefinition): Challenge {
  return {
    ...definition,
    robotConfig: {
      joints: definition.robotConfig.joints.map((joint) => ({ ...joint })),
      geometry: {
        ...definition.robotConfig.geometry,
        basePosition: [...definition.robotConfig.geometry.basePosition],
        collision: {
          ...definition.robotConfig.geometry.collision,
        },
      },
    },
    voxelConfig: {
      ...definition.voxelConfig,
      origin: [...definition.voxelConfig.origin],
      headCenter: [...definition.voxelConfig.headCenter],
      headScale: [...definition.voxelConfig.headScale],
    },
    initialHair: {
      id: definition.initialHair.id,
      name: definition.initialHair.name,
      voxels: new Set(definition.initialHair.voxels.map(coordToKey)),
    },
    targetHair: {
      id: definition.targetHair.id,
      name: definition.targetHair.name,
      voxels: new Set(definition.targetHair.voxels.map(coordToKey)),
    },
    allowedBlocks: [...definition.allowedBlocks],
    starterWorkspace: structuredClone(definition.starterWorkspace),
    scoring: {
      ...definition.scoring,
      weights: { ...definition.scoring.weights },
    },
  };
}
