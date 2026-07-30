import type {
  Challenge,
  JointId,
  RobotGeometryConfig,
  Vec3Tuple,
} from '../../types/domain';

export interface RobotPose {
  base: Vec3Tuple;
  shoulder: Vec3Tuple;
  elbow: Vec3Tuple;
  wrist: Vec3Tuple;
  toolBase: Vec3Tuple;
  endEffector: Vec3Tuple;
  jointPositions: Record<JointId, Vec3Tuple>;
}

export function computeRobotPose(
  robotConfig: Challenge['robotConfig'],
  jointAngles: Readonly<Record<JointId, number>>,
): RobotPose {
  const { geometry } = robotConfig;
  const baseYaw = degreesToRadians(readAngle(jointAngles, 'baseYaw'));
  const shoulderAngle = degreesToRadians(
    readAngle(jointAngles, 'shoulder'),
  );
  const elbowAngle =
    shoulderAngle + degreesToRadians(readAngle(jointAngles, 'elbow'));
  const wristAngle =
    elbowAngle + degreesToRadians(readAngle(jointAngles, 'wrist'));

  const base = geometry.basePosition;
  const shoulder: Vec3Tuple = [
    base[0],
    base[1] + geometry.shoulderHeight,
    base[2],
  ];
  const elbow = addPlanarLink(
    shoulder,
    geometry.upperArmLength,
    shoulderAngle,
    baseYaw,
  );
  const wrist = addPlanarLink(
    elbow,
    geometry.forearmLength,
    elbowAngle,
    baseYaw,
  );
  const toolBase = wrist;
  const endEffector = addPlanarLink(
    toolBase,
    geometry.toolLength,
    wristAngle,
    baseYaw,
  );

  return {
    base,
    shoulder,
    elbow,
    wrist,
    toolBase,
    endEffector,
    jointPositions: {
      baseYaw: base,
      shoulder,
      elbow,
      wrist,
    },
  };
}

export function createInitialJointAngles(
  robotConfig: Challenge['robotConfig'],
): Record<JointId, number> {
  return Object.fromEntries(
    robotConfig.joints.map((joint) => [
      joint.id,
      joint.initialAngleDeg,
    ]),
  );
}

export function linkLengths(
  geometry: RobotGeometryConfig,
): readonly [number, number, number] {
  return [
    geometry.upperArmLength,
    geometry.forearmLength,
    geometry.toolLength,
  ];
}

function addPlanarLink(
  start: Vec3Tuple,
  length: number,
  planarAngle: number,
  yaw: number,
): Vec3Tuple {
  const horizontalLength = Math.cos(planarAngle) * length;
  return [
    start[0] + Math.cos(yaw) * horizontalLength,
    start[1] + Math.sin(planarAngle) * length,
    start[2] - Math.sin(yaw) * horizontalLength,
  ];
}

function readAngle(
  jointAngles: Readonly<Record<JointId, number>>,
  jointId: JointId,
): number {
  const angle = jointAngles[jointId];
  if (!Number.isFinite(angle)) {
    throw new Error(`Missing or invalid angle for joint "${jointId}".`);
  }
  return angle;
}

function degreesToRadians(degrees: number): number {
  return (degrees * Math.PI) / 180;
}
