# HCR Simulator 前端 Demo 技术设计说明（v0.2）

> 用途：统一团队成员认知，并作为 Codex / AI 编程代理的项目上下文、实现边界与验收依据。
>
> 状态：**当前前端 Demo 基线**。本文固化已确认方向；尚未确定的细节统一标记为 **TBD**，实现时不得擅自写死。
>
> 依据：2026-07-29 项目会议 + 当前团队进一步对齐结果。

---

## 1. 当前已确认的项目基线

本版本以以下 7 点为当前确定方案：

1. **前端技术架构**：React + TypeScript + React Three Fiber (R3F) + Three.js + Blockly。
2. **头部/头发建模**：采用“像素化 / 方块化 / voxel”方案，整体视觉与数据结构接近 Minecraft 式方块模型。
3. **机械臂操作逻辑**：大方向采用真实舵机 / 关节控制逻辑；具体舵机数量、角度范围、Block 形式、速度与控制细则仍为 TBD。
4. **剪发判定**：暂不考虑剪刀开关。机械臂末端执行器与“头发 voxel”发生有效接触时，即视为该 voxel 被剪掉/删除。
5. **评分 / Difficulty**：由目标发型完成度、程序效率、完成时间三项组成，优先级依次降低，并通过可配置权重求加权平均分。
6. **当前阶段只实现前端可执行 Demo**：不要求后端参与运行；但代码必须预留清晰的后端接口层，避免未来接入时重构核心逻辑。
7. **当前阶段不接真实 ESP / 舵机硬件**：只模拟虚拟机械臂。真实硬件接入属于未来扩展。

---

## 2. 当前阶段目标与非目标

### 2.1 当前目标

当前目标是完成一个可在浏览器独立运行的 HCR Simulator 前端 Demo，至少能够展示以下闭环：

```text
选择 / 加载本地 Challenge
        ↓
Blockly 编排舵机控制程序
        ↓
Program IR / Command List
        ↓
虚拟机械臂按关节指令运动
        ↓
末端执行器触碰 Hair Voxel
        ↓
Hair Voxel 被删除
        ↓
与 Target Hairstyle 比较
        ↓
计算完成度 / 程序效率 / 时间
        ↓
输出最终加权得分
```

### 2.2 当前明确不做

- 不接后端真实接口；
- 不接 MQTT；
- 不接 ESP / 真实舵机；
- 不实现真实发丝模拟；
- 不实现真实剪刀开合；
- 不追求高精度软体 / 毛发物理；
- 不强制实现逆运动学 IK；
- 不提前绑定具体机械臂型号与自由度；
- 不提前绑定最终竞赛规则、最终评分权重或 CAT 动态题库。

---

## 3. 技术架构

### 3.1 技术栈

| 层级 | 技术 | 当前职责 |
|---|---|---|
| Web 应用 | React | 页面、组件、状态与交互组织 |
| 类型系统 | TypeScript | 数据模型、Command、接口与模块边界 |
| 3D React 层 | React Three Fiber | 使用 React 方式组织 Three.js 场景 |
| 3D 渲染 | Three.js | Scene、Camera、Light、Robot、Voxel 等 |
| 可视化编程 | Blockly | 拼图式程序编辑与自定义舵机 Block |
| 构建工具 | Vite | 本地开发与前端构建 |

### 3.2 总体分层

```text
React UI
Challenge / Blockly / Controls / Score / Result
        ↓
Program / Command Layer
Blockly → Program IR → Robot Commands
        ↓
Robot Control Layer
Servo / Joint State → Command Executor
        ↓
Simulation Layer
Robot Kinematics / Animation / Contact Detection
        ↓
R3F / Three.js Rendering
        +
Voxel Hair State / Remove / Target
```

### 3.3 关键架构约束

- Blockly **禁止直接修改 Three.js Mesh**。
- 3D 组件 **禁止直接承担评分逻辑**。
- UI **禁止直接写死机械臂关节数量**。
- 剪发结果由 Voxel State 表示，而不是仅通过视觉效果表示。
- 当前 Demo 的业务计算全部可在浏览器本地完成，但必须通过接口 / service abstraction 组织，为未来后端替换留出入口。

---

## 4. Voxel 头部 / 头发模型

### 4.1 设计目标

头发采用离散 voxel 表示，而不是连续发丝或复杂 Mesh Deformation。每一个 Hair Voxel 是一个可独立存在或删除的最小“头发单元”。

概念模型：

```text
Voxel Space
[x][y][z]

0 = 空
1 = 存在头发 voxel
```

头部本体与头发建议在逻辑上分离：

```text
Head
├── Solid Head / Face  （不可剪）
└── Hair Voxels        （可剪）
```

### 4.2 推荐数据结构

```ts
export type VoxelKey = string; // e.g. "12,8,4"

export interface VoxelCoord {
  x: number;
  y: number;
  z: number;
}

export interface HairVoxel {
  key: VoxelKey;
  coord: VoxelCoord;
}

export interface HairState {
  voxels: Set<VoxelKey>;
}
```

实际渲染层可以根据性能选择：

- 少量 voxel：普通 Mesh；
- 较多 voxel：`THREE.InstancedMesh`；
- 更大规模：后续再考虑 chunk / greedy meshing 等优化。

当前 Demo 不要求一开始就实现复杂 voxel engine。

### 4.3 Target Hairstyle

目标发型同样使用 HairState / voxel 集合表达：

```ts
export interface HairstyleTarget {
  id: string;
  name: string;
  voxels: Set<VoxelKey>;
}
```

因此“用户当前发型”与“目标发型”可以直接做集合比较。

---

## 5. 机械臂与舵机控制模型

### 5.1 当前确定原则

玩家主要控制对象是机械臂的 **Servo / Joint**，而不是直接给末端执行器下达 Cartesian `Move X/Y/Z` 指令。

当前大方向：

```text
Blockly Command
      ↓
Servo / Joint Angle
      ↓
Robot Kinematic Chain
      ↓
Robot Pose
      ↓
End-Effector Position
```

### 5.2 当前不确定项（TBD）

以下内容后续再确定，当前实现必须可配置：

- 舵机数量；
- 机械臂自由度；
- 每个舵机名称；
- 每个舵机角度上下限；
- 舵机速度；
- 是否允许“绝对角度”与“相对角度”两种 Block；
- 是否支持多个舵机同时运动；
- 是否加入 wait / delay；
- 是否加入传感器、if、while 等高级 Block；
- 是否严格模拟真实舵机运动速度和误差。

### 5.3 通用数据结构

```ts
export type JointId = string;

export interface JointConfig {
  id: JointId;
  name: string;
  minAngleDeg: number;
  maxAngleDeg: number;
  initialAngleDeg: number;
  speedDegPerSec?: number;
}

export interface JointState {
  id: JointId;
  angleDeg: number;
}

export interface RobotState {
  joints: Record<JointId, JointState>;
}
```

### 5.4 第一版 Robot Command

第一版只需要保留最小可扩展语义：

```ts
export type RobotCommand =
  | {
      type: 'set-joint-angle';
      jointId: JointId;
      angleDeg: number;
    }
  | {
      type: 'change-joint-angle';
      jointId: JointId;
      deltaDeg: number;
    }
  | {
      type: 'wait';
      durationMs: number;
    };
```

注意：**不存在 `open-scissors` / `close-scissors` 指令。**

---

## 6. 剪发 / 碰撞判定

### 6.1 当前规则

当前 Demo 中，机械臂末端执行器本身视为“剪发工具”。

> 当末端执行器的有效碰撞体与一个 Hair Voxel 发生有效接触时，该 Hair Voxel 立即视为被剪掉，并从当前 HairState 中删除。

流程：

```text
End Effector Move
      ↓
Contact / Overlap Detection
      ↓
Hit Hair Voxel?
   ├─ No → Nothing
   └─ Yes
        ↓
   Remove Voxel
        ↓
   Update HairState
        ↓
   Update Rendering
```

### 6.2 实现边界

当前只需要稳定、可重复的几何接触判定，不需要真实剪刀物理。

可采用：

- 包围盒 AABB；
- Sphere / Box overlap；
- 基于 voxel 网格坐标的空间判断；
- 简单 Ray / Sweep（如后续有需要）。

当前 **不要求引入完整物理引擎**。如果后续玩法证明需要真实碰撞，再单独评估 Rapier 等方案。

---

## 7. Blockly 与 Program IR

### 7.1 Blockly 的职责

Blockly 只负责：

- 提供拼图式编程 UI；
- 生成结构化 Program IR；
- 表达舵机 / 关节控制和基本流程控制。

Blockly 不直接：

- 修改 Robot Mesh；
- 删除 Hair Voxel；
- 计算得分；
- 访问后端。

### 7.2 第一版候选 Block

```text
Servo / Joint
├── Set Joint [id] to [angle]°
├── Increase Joint [id] by [angle]°
└── Decrease Joint [id] by [angle]°

Control
├── Repeat [N]
└── Wait [time]
```

具体 Block 名称、舵机 ID、角度范围仍为 TBD。

### 7.3 Program IR

示例：

```ts
export type ProgramNode =
  | RobotCommand
  | {
      type: 'repeat';
      count: number;
      body: ProgramNode[];
    };

export interface Program {
  nodes: ProgramNode[];
}
```

执行时由 Program Executor 展开 / 解释为 RobotCommand 流。

---

## 8. 评分与 Difficulty 设计

### 8.1 三项评分维度

当前采用：

1. **目标发型完成度（Completion）**：第一标准，权重最高；
2. **程序效率（Program Efficiency）**：第二标准；
3. **完成时间（Completion Time）**：第三标准。

三项统一换算为 `0~100` 分，然后计算加权平均：

```text
FinalScore =
    wCompletion × CompletionScore
  + wEfficiency × EfficiencyScore
  + wTime × TimeScore

wCompletion + wEfficiency + wTime = 1
```

### 8.2 默认 Demo 权重

为便于 Codex 直接实现 Demo，当前提供**可配置默认值**：

```ts
export const DEFAULT_SCORE_WEIGHTS = {
  completion: 0.60,
  efficiency: 0.25,
  time: 0.15,
};
```

该权重不是最终产品规则，必须集中在配置中，不得散落硬编码。

### 8.3 完成度：Voxel IoU

推荐第一版直接使用 Hair Voxel 集合 IoU：

```text
CompletionScore = |Target ∩ Result| / |Target ∪ Result| × 100
```

它可以同时惩罚：

- 应该保留却被剪掉的 voxel；
- 应该剪掉却仍然存在的 voxel；
- 结果形状位置错误。

### 8.4 程序效率

程序效率应鼓励：

- 更少的无效操作；
- 更紧凑的程序；
- 合理使用循环；
- 更少的实际机械臂动作。

第一版建议把“程序成本”抽象为独立函数，不把规则绑定在 Blockly：

```ts
export interface ProgramMetrics {
  sourceBlockCount: number;
  executedCommandCount: number;
}

export interface EfficiencyConfig {
  referenceCost: number;
  commandWeight: number;
}
```

Demo 可采用：

```text
ProgramCost = SourceBlockCount
            + commandWeight × ExecutedCommandCount

EfficiencyScore = min(100, referenceCost / ProgramCost × 100)
```

`referenceCost` 由 Challenge 配置提供；以后可替换为最优解、动态基准或后端计算。

### 8.5 时间得分

Challenge 提供 `referenceTimeMs`：

```text
TimeScore = min(100, referenceTime / elapsedTime × 100)
```

因此：

- 小于等于参考时间：100 分；
- 超过参考时间：按比例下降。

未来可以改为分段函数、倒计时或竞技模式，不影响其他模块。

### 8.6 Difficulty 的当前含义

当前阶段 Difficulty 不单独绑定一个固定“难度公式”。它可以由 Challenge 配置综合体现，例如：

- Target Hairstyle 的复杂程度；
- 可用 Block 类型；
- 机械臂初始姿态；
- 评分参考成本；
- 评分参考时间；
- 后续可能加入的舵机限制。

---

## 9. Challenge 数据模型

当前 Demo 使用本地静态 Challenge 数据。

```ts
export interface Challenge {
  id: string;
  name: string;
  description?: string;

  robotConfig: {
    joints: JointConfig[];
  };

  initialHair: HairstyleTarget;
  targetHair: HairstyleTarget;

  allowedBlocks: string[];

  scoring: {
    weights: {
      completion: number;
      efficiency: number;
      time: number;
    };
    referenceProgramCost: number;
    referenceTimeMs: number;
    commandWeight: number;
  };
}
```

第一版 Challenge 可以直接存放在：

```text
src/data/challenges/*.ts
```

或 JSON 文件中。

---

## 10. 前端独立运行与后端接口预留

### 10.1 当前原则

**Demo 的全部核心流程必须在没有后端的情况下可运行。**

同时，业务模块不得直接依赖“本地静态数据”这一实现细节。

推荐使用 Provider / Gateway 抽象：

```ts
export interface ChallengeProvider {
  getChallenge(id: string): Promise<Challenge>;
  listChallenges(): Promise<Challenge[]>;
}

export interface ScoreProvider {
  score(input: ScoreInput): Promise<ScoreResult>;
}
```

当前实现：

```text
ChallengeProvider
    └── LocalChallengeProvider

ScoreProvider
    └── LocalScoreProvider
```

未来可替换：

```text
ChallengeProvider
    └── HttpChallengeProvider

ScoreProvider
    └── HttpScoreProvider
```

UI、Blockly、Robot Controller 不应该知道当前使用的是 Local 还是 HTTP。

### 10.2 建议预留 API 形状

未来可能需要：

```text
GET  /api/challenges
GET  /api/challenges/:id
POST /api/score
POST /api/simulations       （未来）
```

这里只定义接口边界，**当前 Demo 不实现网络调用**。

---

## 11. 当前不接真实 ESP

当前系统链路只有：

```text
Blockly
   ↓
Program IR
   ↓
Local Program Executor
   ↓
Virtual Robot Controller
   ↓
R3F / Three.js
```

暂时不存在：

```text
ESP
MQTT
PWM
Real Servo
Real Robot
```

但 Robot Command 应继续使用通用、与硬件语义接近的 `jointId + angle` 形式，避免未来硬件接入时完全重做编程层。

---

## 12. 推荐前端目录结构

```text
src/
├── app/
│   ├── App.tsx
│   └── routes.tsx                 # 如暂时单页可省略
│
├── components/
│   ├── challenge/
│   ├── controls/
│   ├── result/
│   └── layout/
│
├── features/
│   ├── blockly/
│   │   ├── BlocklyEditor.tsx
│   │   ├── blocks/
│   │   └── programCompiler.ts
│   │
│   ├── robot/
│   │   ├── RobotModel.tsx
│   │   ├── robotController.ts
│   │   ├── robotKinematics.ts
│   │   └── types.ts
│   │
│   ├── voxel/
│   │   ├── VoxelHair.tsx
│   │   ├── voxelState.ts
│   │   ├── contactDetection.ts
│   │   └── similarity.ts
│   │
│   ├── simulation/
│   │   ├── SimulatorCanvas.tsx
│   │   ├── programExecutor.ts
│   │   └── simulationStore.ts
│   │
│   └── scoring/
│       ├── scoring.ts
│       ├── efficiency.ts
│       └── types.ts
│
├── services/
│   ├── challengeProvider.ts
│   ├── scoreProvider.ts
│   └── local/
│       ├── localChallengeProvider.ts
│       └── localScoreProvider.ts
│
├── data/
│   └── challenges/
│
├── types/
│   └── shared.ts
│
└── main.tsx
```

目录名称可根据项目现状调整，但**模块边界应保留**。

---

## 13. 前端 Demo 最小验收标准（MVP）

### P0 - 必须完成

- [ ] React + TypeScript + Vite 项目可正常启动；
- [ ] R3F / Three.js 3D Canvas 正常渲染；
- [ ] 页面中存在一个可观察的虚拟机械臂；
- [ ] 存在方块化 Head + Hair Voxel；
- [ ] Hair Voxel 有独立逻辑状态；
- [ ] Blockly Workspace 可以编辑基础舵机/关节指令；
- [ ] Blockly 能转换为 Program IR；
- [ ] Program Executor 能按顺序执行 Command；
- [ ] 机械臂关节能根据 Command 运动；
- [ ] 末端碰到 Hair Voxel 时，该 voxel 被删除；
- [ ] Reset 能恢复初始机械臂和头发状态；
- [ ] Demo 可以统计运行时间；
- [ ] Demo 可以统计 source block 数与 executed command 数；
- [ ] Demo 可以计算 Voxel IoU 完成度；
- [ ] Demo 可以计算三项评分和最终加权分；
- [ ] Challenge 与 Score 通过 Provider abstraction 获取/计算，而不是直接写死在 UI。

### P1 - 可以随后补充

- [ ] 更好的 voxel 视觉；
- [ ] `InstancedMesh` 优化；
- [ ] 多个 Challenge；
- [ ] 更丰富 Blockly 控制结构；
- [ ] 相机视角 / OrbitControls；
- [ ] 程序单步执行、暂停、继续；
- [ ] 当前执行 Block 高亮；
- [ ] 轨迹 / 调试信息；
- [ ] 双人竞赛 UI。

---

## 14. Codex 实施约束

Codex 在实现时必须遵守以下规则：

### MUST

1. 使用 **React + TypeScript + Vite**。
2. 使用 **@react-three/fiber + three** 作为 3D 主方案。
3. 使用 **Blockly** 作为可视化编程方案。
4. 将 Hair 表示为**可删除 voxel state**，而不仅是一个不可变 GLB 模型。
5. 机械臂操作抽象为 **servo / joint command**。
6. 暂时不要加入剪刀开关；末端与 hair voxel 接触即删除 voxel。
7. Blockly → Program IR → Executor → Robot Controller，禁止 Blockly 直接控制 Mesh。
8. 评分逻辑必须独立于 UI 与 3D 渲染。
9. 后端暂时用 Local Provider / Mock 实现，但保留可替换接口。
10. 所有 TBD 参数应集中配置，不要散落 magic numbers。
11. 优先实现可执行 Demo，而不是过度封装或提前建设完整产品架构。

### MUST NOT

1. 不要引入 Unity。
2. 不要引入真实 ESP / MQTT / WebSerial / WebBluetooth。
3. 不要实现真实发丝模拟。
4. 不要实现真实剪刀开合。
5. 不要默认玩家使用 `Move X/Y/Z` 控制机械臂。
6. 不要把机械臂自由度、舵机角度限制写死在组件内部。
7. 不要让 React State 承担每帧 60 FPS 的高频动画更新；高频 3D 变化放在 R3F/Three.js 渲染逻辑中。
8. 不要让网络接口成为 Demo 运行前提。

---

## 15. 当前 TBD 清单

以下问题留待后续团队确认：

- 机械臂具体模型 / 自由度；
- 舵机具体数量与名称；
- 舵机真实角度与速度限制；
- Blockly 最终 Block 集合；
- 是否保留绝对角度和相对角度两种控制；
- 是否需要同时驱动多个舵机；
- 末端执行器碰撞体的精确形状 / 大小；
- voxel 分辨率与方块尺寸；
- 头部本体的 voxel / mesh 表达；
- Target Hairstyle 的正式制作流程；
- Program Efficiency 的最终成本函数；
- 三项评分最终权重；
- 双人竞赛具体展示与胜负规则；
- 后端 API 正式协议；
- CAT / Dynamic QBank 接入方式；
- 真实 ESP / MQTT 接入方式。

在上述内容确认之前，应使用**配置、接口和可替换策略**处理，而不是假设某个方案已经确定。

---

## 16. 一句话架构定义

> **HCR Simulator 当前是一个纯前端可运行的 Web 3D 编程 Demo：用户通过 Blockly 编排真实舵机语义的关节控制指令，驱动 R3F/Three.js 中的虚拟机械臂；机械臂末端触碰方块化 Hair Voxel 即完成剪除；系统基于目标 voxel 发型完成度、程序效率和完成时间计算加权得分，同时通过 Provider / Service 接口为未来后端与真实硬件接入保留扩展边界。**
