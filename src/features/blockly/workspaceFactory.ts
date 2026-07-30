import * as Blockly from 'blockly/core';
import type { Challenge } from '../../types/domain';
import { registerHcrBlocks } from './blockDefinitions';

export function createHeadlessWorkspace(challenge: Challenge): Blockly.Workspace {
  registerHcrBlocks(challenge.robotConfig.joints);
  const workspace = new Blockly.Workspace();
  loadWorkspaceState(workspace, challenge.starterWorkspace);
  return workspace;
}

export function loadWorkspaceState(
  workspace: Blockly.Workspace,
  state: Record<string, unknown>,
): void {
  workspace.clear();
  Blockly.serialization.workspaces.load(state, workspace);
}

export function saveWorkspaceState(
  workspace: Blockly.Workspace,
): Record<string, unknown> {
  return Blockly.serialization.workspaces.save(workspace);
}
