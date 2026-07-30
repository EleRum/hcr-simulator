import * as Blockly from 'blockly/core';
import * as zhHans from 'blockly/msg/zh-hans';
import type {
  AllowedBlockType,
  Challenge,
  JointConfig,
} from '../../types/domain';
import { BLOCK_FIELDS, BLOCK_TYPES } from './blockConstants';

const semanticToBlocklyType: Record<AllowedBlockType, string> = {
  'set-joint-angle': BLOCK_TYPES.setJointAngle,
  wait: BLOCK_TYPES.wait,
  repeat: BLOCK_TYPES.repeat,
};

export function registerHcrBlocks(joints: readonly JointConfig[]): void {
  if (joints.length === 0) {
    throw new Error('At least one joint is required to register HCR blocks.');
  }

  const locale = Object.fromEntries(
    Object.entries(zhHans).filter(([, value]) => typeof value === 'string'),
  ) as Record<string, string>;
  Blockly.setLocale(locale);
  const options = joints.map(
    (joint) => [joint.name, joint.id] as [string, string],
  );
  const initialJoint = joints[0];
  const jointById = new Map(joints.map((joint) => [joint.id, joint]));

  Blockly.Blocks[BLOCK_TYPES.setJointAngle] = {
    init(this: Blockly.Block) {
      const angleField = new Blockly.FieldNumber(
        initialJoint.initialAngleDeg,
        Math.min(...joints.map((joint) => joint.minAngleDeg)),
        Math.max(...joints.map((joint) => joint.maxAngleDeg)),
        1,
        function validateAngle(
          this: Blockly.FieldNumber,
          value: string | number,
        ) {
          const block = this.getSourceBlock();
          const jointId = block?.getFieldValue(BLOCK_FIELDS.jointId);
          const joint = jointById.get(jointId);
          const numericValue = Number(value);
          if (
            !joint ||
            numericValue < joint.minAngleDeg ||
            numericValue > joint.maxAngleDeg
          ) {
            return null;
          }
          return numericValue;
        },
      );

      this.appendDummyInput()
        .appendField('设置')
        .appendField(
          new Blockly.FieldDropdown(options),
          BLOCK_FIELDS.jointId,
        )
        .appendField('到')
        .appendField(angleField, BLOCK_FIELDS.angle)
        .appendField('°');
      this.setPreviousStatement(true);
      this.setNextStatement(true);
      this.setColour('#18a6a6');
      this.setTooltip('将一个关节移动到指定绝对角度');
    },
  };

  Blockly.Blocks[BLOCK_TYPES.wait] = {
    init(this: Blockly.Block) {
      this.appendDummyInput()
        .appendField('等待')
        .appendField(
          new Blockly.FieldNumber(200, 0, 5_000, 100),
          BLOCK_FIELDS.duration,
        )
        .appendField('ms');
      this.setPreviousStatement(true);
      this.setNextStatement(true);
      this.setColour('#7c6ee6');
      this.setTooltip('保持当前姿态并等待指定时间');
    },
  };

  Blockly.Blocks[BLOCK_TYPES.repeat] = {
    init(this: Blockly.Block) {
      this.appendDummyInput()
        .appendField('重复')
        .appendField(
          new Blockly.FieldNumber(2, 1, 20, 1),
          BLOCK_FIELDS.count,
        )
        .appendField('次');
      this.appendStatementInput(BLOCK_FIELDS.body).appendField('执行');
      this.setPreviousStatement(true);
      this.setNextStatement(true);
      this.setColour('#df8a35');
      this.setTooltip('按顺序重复内部命令');
    },
  };
}

export function createToolbox(
  challenge: Pick<Challenge, 'allowedBlocks'>,
): Blockly.utils.toolbox.ToolboxDefinition {
  const servoContents = challenge.allowedBlocks
    .filter((type) => type === 'set-joint-angle')
    .map((type) => ({
      kind: 'block',
      type: semanticToBlocklyType[type],
    }));
  const controlContents = challenge.allowedBlocks
    .filter((type) => type !== 'set-joint-angle')
    .map((type) => ({
      kind: 'block',
      type: semanticToBlocklyType[type],
    }));

  return {
    kind: 'categoryToolbox',
    contents: [
      {
        kind: 'category',
        name: '舵机',
        colour: '#18a6a6',
        contents: servoContents,
      },
      {
        kind: 'category',
        name: '控制',
        colour: '#7c6ee6',
        contents: controlContents,
      },
    ],
  } as Blockly.utils.toolbox.ToolboxDefinition;
}

export function blockTypeToSemantic(
  blockType: string,
): AllowedBlockType | undefined {
  return (
    Object.entries(semanticToBlocklyType).find(
      ([, type]) => type === blockType,
    )?.[0] as AllowedBlockType | undefined
  );
}
