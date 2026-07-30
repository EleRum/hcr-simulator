import type { JointId } from '../../types/domain';

export type RobotCommand =
  | {
      type: 'set-joint-angle';
      jointId: JointId;
      angleDeg: number;
      sourceBlockId: string;
    }
  | {
      type: 'wait';
      durationMs: number;
      sourceBlockId: string;
    };

export type ProgramNode =
  | RobotCommand
  | {
      type: 'repeat';
      count: number;
      body: ProgramNode[];
      sourceBlockId: string;
    };

export interface Program {
  nodes: ProgramNode[];
  sourceBlockCount: number;
}

export interface CompiledProgram {
  program: Program;
  runtimeCommands: RobotCommand[];
  executedCommandCount: number;
}
