import type { RobotCommand } from '../blockly/programTypes';
import {
  RobotController,
  type MoveAdvanceResult,
} from '../robot/RobotController';

interface ActiveWait {
  command: Extract<RobotCommand, { type: 'wait' }>;
  elapsedMs: number;
}

export interface ProgramExecutorHooks {
  onCommandStart?: (command: RobotCommand, index: number) => void;
  onCommandComplete?: (command: RobotCommand, index: number) => void;
  onMovement?: (movement: MoveAdvanceResult) => void;
}

export interface ExecutorAdvanceResult {
  consumedMs: number;
  commandsCompleted: number;
  programCompleted: boolean;
}

export class ProgramExecutor {
  private commands: readonly RobotCommand[] = [];
  private commandIndex = 0;
  private activeCommand: RobotCommand | undefined;
  private activeWait: ActiveWait | undefined;

  constructor(private readonly robotController: RobotController) {}

  load(commands: readonly RobotCommand[]): void {
    this.commands = commands.map((command) => ({ ...command }));
    this.commandIndex = 0;
    this.activeCommand = undefined;
    this.activeWait = undefined;
  }

  reset(): void {
    this.commands = [];
    this.commandIndex = 0;
    this.activeCommand = undefined;
    this.activeWait = undefined;
  }

  advance(
    deltaMs: number,
    hooks: ProgramExecutorHooks = {},
    maxCommandsToComplete = Number.POSITIVE_INFINITY,
  ): ExecutorAdvanceResult {
    if (!Number.isFinite(deltaMs) || deltaMs < 0) {
      throw new Error('Delta must be a finite non-negative number.');
    }

    let remainingMs = deltaMs;
    let consumedMs = 0;
    let commandsCompleted = 0;
    let safetyCounter = 0;

    while (
      this.commandIndex < this.commands.length &&
      commandsCompleted < maxCommandsToComplete
    ) {
      safetyCounter += 1;
      if (safetyCounter > this.commands.length + 1) {
        throw new Error('Program executor failed to make progress.');
      }

      const command = this.commands[this.commandIndex];
      if (!this.activeCommand) {
        this.activeCommand = command;
        hooks.onCommandStart?.(command, this.commandIndex);
        if (command.type === 'set-joint-angle') {
          this.robotController.beginMove(
            command.jointId,
            command.angleDeg,
          );
        } else {
          this.activeWait = { command, elapsedMs: 0 };
        }
      }

      let commandCompleted = false;
      let commandConsumedMs = 0;

      if (command.type === 'set-joint-angle') {
        const movement = this.robotController.advanceMove(remainingMs);
        commandConsumedMs = movement.consumedMs;
        commandCompleted = movement.completed;
        if (movement.moved) {
          hooks.onMovement?.(movement);
        }
      } else {
        const wait = this.activeWait;
        if (!wait) {
          throw new Error('Wait command state is missing.');
        }
        const waitRemaining = Math.max(
          0,
          command.durationMs - wait.elapsedMs,
        );
        commandConsumedMs = Math.min(remainingMs, waitRemaining);
        wait.elapsedMs += commandConsumedMs;
        commandCompleted = wait.elapsedMs >= command.durationMs;
      }

      consumedMs += commandConsumedMs;
      remainingMs = Math.max(0, remainingMs - commandConsumedMs);

      if (!commandCompleted) {
        break;
      }

      hooks.onCommandComplete?.(command, this.commandIndex);
      this.commandIndex += 1;
      commandsCompleted += 1;
      this.activeCommand = undefined;
      this.activeWait = undefined;

      if (remainingMs === 0 && this.nextCommandRequiresTime()) {
        break;
      }
    }

    return {
      consumedMs,
      commandsCompleted,
      programCompleted: this.commandIndex >= this.commands.length,
    };
  }

  getCurrentCommand(): RobotCommand | undefined {
    return this.activeCommand ?? this.commands[this.commandIndex];
  }

  getCommandIndex(): number {
    return this.commandIndex;
  }

  getCommandCount(): number {
    return this.commands.length;
  }

  isComplete(): boolean {
    return this.commandIndex >= this.commands.length;
  }

  private nextCommandRequiresTime(): boolean {
    const next = this.commands[this.commandIndex];
    if (!next) {
      return false;
    }
    if (next.type === 'wait') {
      return next.durationMs > 0;
    }
    const currentAngle = this.robotController.getAngles()[next.jointId];
    return currentAngle !== next.angleDeg;
  }
}
