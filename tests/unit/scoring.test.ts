import { describe, expect, it } from 'vitest';
import {
  calculateScore,
  estimateProgramDuration,
  validateScoringConfig,
} from '../../src/features/scoring/scoring';
import { calculateVoxelIoU } from '../../src/features/voxel/similarity';
import type {
  JointConfig,
  ScoringConfig,
  VoxelKey,
} from '../../src/types/domain';

const scoring: ScoringConfig = {
  weights: { completion: 0.6, efficiency: 0.25, time: 0.15 },
  referenceProgramCost: 10,
  referenceTimeMs: 5_000,
  commandWeight: 0.25,
};

describe('voxel IoU', () => {
  it('covers empty, disjoint and overlapping sets', () => {
    expect(calculateVoxelIoU(new Set(), new Set())).toBe(100);
    expect(
      calculateVoxelIoU(
        new Set<VoxelKey>(['0,0,0']),
        new Set<VoxelKey>(),
      ),
    ).toBe(0);
    expect(
      calculateVoxelIoU(
        new Set<VoxelKey>(['0,0,0', '1,0,0']),
        new Set<VoxelKey>(['1,0,0', '2,0,0']),
      ),
    ).toBeCloseTo(100 / 3);
  });
});

describe('score calculation', () => {
  it('returns a weighted and clamped breakdown', () => {
    const result = calculateScore({
      targetVoxels: new Set<VoxelKey>(['0,0,0', '1,0,0']),
      resultVoxels: new Set<VoxelKey>(['0,0,0']),
      programMetrics: {
        sourceBlockCount: 10,
        executedCommandCount: 8,
        estimatedDurationMs: 10_000,
      },
      scoring,
    });

    expect(result.completionScore).toBe(50);
    expect(result.programCost).toBe(12);
    expect(result.efficiencyScore).toBeCloseTo(83.3333);
    expect(result.timeScore).toBe(50);
    expect(result.finalScore).toBeCloseTo(58.3333);
  });

  it('treats zero cost and zero duration as full scores', () => {
    const result = calculateScore({
      targetVoxels: new Set(),
      resultVoxels: new Set(),
      programMetrics: {
        sourceBlockCount: 0,
        executedCommandCount: 0,
        estimatedDurationMs: 0,
      },
      scoring,
    });

    expect(result).toMatchObject({
      completionScore: 100,
      efficiencyScore: 100,
      timeScore: 100,
      finalScore: 100,
    });
  });

  it('rejects scoring weights that do not sum to one', () => {
    expect(() =>
      validateScoringConfig({
        ...scoring,
        weights: { completion: 0.5, efficiency: 0.2, time: 0.2 },
      }),
    ).toThrow('must sum to 1');
  });
});

describe('program duration estimation', () => {
  const joints: JointConfig[] = [
    {
      id: 'joint',
      name: 'Joint',
      axis: 'z',
      minAngleDeg: -90,
      maxAngleDeg: 90,
      initialAngleDeg: 0,
      speedDegPerSec: 45,
    },
  ];

  it('updates simulated joint state sequentially', () => {
    const duration = estimateProgramDuration(
      [
        { type: 'set-joint-angle', jointId: 'joint', angleDeg: 45 },
        { type: 'wait', durationMs: 500 },
        { type: 'set-joint-angle', jointId: 'joint', angleDeg: -45 },
      ],
      joints,
    );

    expect(duration).toBe(3_500);
  });

  it('rejects unknown joints and invalid waits', () => {
    expect(() =>
      estimateProgramDuration(
        [{ type: 'set-joint-angle', jointId: 'missing', angleDeg: 0 }],
        joints,
      ),
    ).toThrow('Unknown joint');
    expect(() =>
      estimateProgramDuration(
        [{ type: 'wait', durationMs: -1 }],
        joints,
      ),
    ).toThrow('non-negative');
  });
});
