import type { Challenge, JointId, Vec3Tuple } from '../../types/domain';
import {
  computeRobotPose,
  createInitialJointAngles,
  type RobotPose,
} from './kinematics';

interface ActiveMove {
  jointId: JointId;
  startAngleDeg: number;
  targetAngleDeg: number;
  durationMs: number;
  elapsedMs: number;
}

export interface MoveAdvanceResult {
  consumedMs: number;
  completed: boolean;
  moved: boolean;
  previousEndEffector: Vec3Tuple;
  currentEndEffector: Vec3Tuple;
}

export class RobotController {
  private readonly configById: Map<
    JointId,
    Challenge['robotConfig']['joints'][number]
  >;
  private jointAngles: Record<JointId, number>;
  private activeMove: ActiveMove | undefined;

  constructor(private readonly robotConfig: Challenge['robotConfig']) {
    this.configById = new Map(
      robotConfig.joints.map((joint) => [joint.id, joint]),
    );
    this.jointAngles = createInitialJointAngles(robotConfig);
  }

  reset(): void {
    this.jointAngles = createInitialJointAngles(this.robotConfig);
    this.activeMove = undefined;
  }

  beginMove(jointId: JointId, targetAngleDeg: number): void {
    const config = this.configById.get(jointId);
    if (!config) {
      throw new Error(`Unknown joint "${jointId}".`);
    }
    if (
      !Number.isFinite(targetAngleDeg) ||
      targetAngleDeg < config.minAngleDeg ||
      targetAngleDeg > config.maxAngleDeg
    ) {
      throw new Error(
        `Angle ${targetAngleDeg} is outside the range for "${jointId}".`,
      );
    }

    const startAngleDeg = this.jointAngles[jointId];
    this.activeMove = {
      jointId,
      startAngleDeg,
      targetAngleDeg,
      durationMs:
        (Math.abs(targetAngleDeg - startAngleDeg) /
          config.speedDegPerSec) *
        1000,
      elapsedMs: 0,
    };
  }

  advanceMove(deltaMs: number): MoveAdvanceResult {
    if (!this.activeMove) {
      throw new Error('No active robot move.');
    }
    if (!Number.isFinite(deltaMs) || deltaMs < 0) {
      throw new Error('Delta must be a finite non-negative number.');
    }

    const move = this.activeMove;
    const previousEndEffector = this.getPose().endEffector;
    const remainingMs = Math.max(0, move.durationMs - move.elapsedMs);
    const consumedMs = Math.min(deltaMs, remainingMs);
    move.elapsedMs += consumedMs;

    const progress =
      move.durationMs === 0 ? 1 : move.elapsedMs / move.durationMs;
    const nextAngle =
      move.startAngleDeg +
      (move.targetAngleDeg - move.startAngleDeg) * progress;
    this.jointAngles[move.jointId] = nextAngle;

    const currentEndEffector = this.getPose().endEffector;
    const completed = progress >= 1;
    if (completed) {
      this.jointAngles[move.jointId] = move.targetAngleDeg;
      this.activeMove = undefined;
    }

    return {
      consumedMs,
      completed,
      moved: !pointsEqual(previousEndEffector, currentEndEffector),
      previousEndEffector,
      currentEndEffector,
    };
  }

  getAngles(): Readonly<Record<JointId, number>> {
    return { ...this.jointAngles };
  }

  getPose(): RobotPose {
    return computeRobotPose(this.robotConfig, this.jointAngles);
  }

  hasActiveMove(): boolean {
    return this.activeMove !== undefined;
  }
}

function pointsEqual(a: Vec3Tuple, b: Vec3Tuple): boolean {
  return (
    Math.abs(a[0] - b[0]) < Number.EPSILON &&
    Math.abs(a[1] - b[1]) < Number.EPSILON &&
    Math.abs(a[2] - b[2]) < Number.EPSILON
  );
}
