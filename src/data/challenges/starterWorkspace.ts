import { BLOCK_FIELDS, BLOCK_TYPES } from '../../features/blockly/blockConstants';

interface SerializedBlock {
  type: string;
  id: string;
  x?: number;
  y?: number;
  fields?: Record<string, string | number>;
  inputs?: Record<string, { block: SerializedBlock }>;
  next?: { block: SerializedBlock };
}

function chain(
  block: SerializedBlock,
  next?: SerializedBlock,
): SerializedBlock {
  return next ? { ...block, next: { block: next } } : block;
}

function setJoint(
  id: string,
  jointId: string,
  angleDeg: number,
  next?: SerializedBlock,
): SerializedBlock {
  return chain(
    {
      type: BLOCK_TYPES.setJointAngle,
      id,
      fields: {
        [BLOCK_FIELDS.jointId]: jointId,
        [BLOCK_FIELDS.angle]: angleDeg,
      },
    },
    next,
  );
}

function wait(
  id: string,
  durationMs: number,
  next?: SerializedBlock,
): SerializedBlock {
  return chain(
    {
      type: BLOCK_TYPES.wait,
      id,
      fields: {
        [BLOCK_FIELDS.duration]: durationMs,
      },
    },
    next,
  );
}

function repeat(
  id: string,
  count: number,
  body: SerializedBlock,
  next?: SerializedBlock,
): SerializedBlock {
  return chain(
    {
      type: BLOCK_TYPES.repeat,
      id,
      fields: {
        [BLOCK_FIELDS.count]: count,
      },
      inputs: {
        [BLOCK_FIELDS.body]: { block: body },
      },
    },
    next,
  );
}

const starterProgram = setJoint(
  'starter-shoulder',
  'shoulder',
  50,
  setJoint(
    'starter-elbow',
    'elbow',
    -15,
    setJoint(
      'starter-wrist',
      'wrist',
      -30,
      setJoint(
        'starter-base-left',
        'baseYaw',
        -24,
        wait(
          'starter-settle',
          200,
          setJoint(
            'starter-base-right',
            'baseYaw',
            24,
            repeat(
              'starter-repeat',
              2,
              wait('starter-repeat-wait', 100),
              setJoint(
                'starter-base-return',
                'baseYaw',
                -24,
                setJoint('starter-base-finish', 'baseYaw', 24),
              ),
            ),
          ),
        ),
      ),
    ),
  ),
);

starterProgram.x = 40;
starterProgram.y = 40;

export const starterWorkspaceState: Record<string, unknown> = {
  blocks: {
    languageVersion: 0,
    blocks: [starterProgram],
  },
};
