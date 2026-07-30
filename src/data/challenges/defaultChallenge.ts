import { generateDefaultHairstyles } from '../../features/voxel/hairGenerator';
import type { ChallengeDefinition } from '../../types/domain';
import { starterWorkspaceState } from './starterWorkspace';

const { initialHair, targetHair } = generateDefaultHairstyles();

export const DEFAULT_CHALLENGE_ID = 'neat-short-cap';

export const defaultChallengeDefinition: ChallengeDefinition = {
  id: DEFAULT_CHALLENGE_ID,
  name: '整齐短发修剪',
  description:
    '使用四关节机械臂移除厚帽型外层 voxel，尽量保留对称短发目标。',
  robotConfig: {
    joints: [
      {
        id: 'baseYaw',
        name: '底座旋转',
        axis: 'y',
        minAngleDeg: -60,
        maxAngleDeg: 60,
        initialAngleDeg: -45,
        speedDegPerSec: 60,
      },
      {
        id: 'shoulder',
        name: '肩关节',
        axis: 'z',
        minAngleDeg: -20,
        maxAngleDeg: 100,
        initialAngleDeg: 45,
        speedDegPerSec: 45,
      },
      {
        id: 'elbow',
        name: '肘关节',
        axis: 'z',
        minAngleDeg: -135,
        maxAngleDeg: 10,
        initialAngleDeg: -80,
        speedDegPerSec: 60,
      },
      {
        id: 'wrist',
        name: '腕关节',
        axis: 'z',
        minAngleDeg: -100,
        maxAngleDeg: 100,
        initialAngleDeg: 35,
        speedDegPerSec: 75,
      },
    ],
    geometry: {
      basePosition: [0, 0, 0],
      shoulderHeight: 0.4,
      upperArmLength: 1.05,
      forearmLength: 0.9,
      toolLength: 0.35,
      toolRadius: 0.12,
    },
  },
  voxelConfig: {
    origin: [1.35, 1.5, 0],
    size: 0.16,
    headCenter: [1.35, 1.42, 0],
    headScale: [0.68, 0.86, 0.68],
  },
  initialHair,
  targetHair,
  allowedBlocks: ['set-joint-angle', 'wait', 'repeat'],
  starterWorkspace: starterWorkspaceState,
  scoring: {
    weights: {
      completion: 0.6,
      efficiency: 0.25,
      time: 0.15,
    },
    referenceProgramCost: 8,
    referenceTimeMs: 12_000,
    commandWeight: 0.25,
  },
};
