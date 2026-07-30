export const BLOCK_TYPES = {
  setJointAngle: 'hcr_set_joint_angle',
  wait: 'hcr_wait',
  repeat: 'hcr_repeat',
} as const;

export const BLOCK_FIELDS = {
  jointId: 'JOINT_ID',
  angle: 'ANGLE',
  duration: 'DURATION',
  count: 'COUNT',
  body: 'DO',
} as const;
