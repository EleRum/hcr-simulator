# HCR Simulator

纯前端 Web 3D 编程 Demo。用户使用 **Blockly** 可视化编排五关节机械臂的舵机角度指令，机械臂末端接触 **Hair Voxel** 时完成剪除；头部由确定性几何约束防穿模。系统按目标发型完成度（Voxel IoU）、程序效率和估算执行时间给出加权评分。

> **当前状态：Phase 1–6 已完成，纯前端 Demo 主闭环可运行；Phase 7 跨浏览器人工验收尚未执行。**

---

## 架构总览

```
┌──────────────────────────────────────────────────────────┐
│                     浏览器 (React)                        │
│  ┌─────────┐  ┌──────────────────┐  ┌────────────────┐  │
│  │ Blockly │  │  R3F 3D Canvas   │  │ Inspector/Log  │  │
│  │ Editor  │  │  (Three.js)      │  │ Panel          │  │
│  └────┬────┘  └────────┬─────────┘  └───────┬────────┘  │
│       │                │                     │           │
│       ▼                ▼                     ▼           │
│  ┌────────────────────────────────────────────────────┐  │
│  │              SimulationEngine (核心)                 │  │
│  │  RobotController  │  ProgramExecutor  │  Scoring   │  │
│  │  HeadCollision    │  ContactDetection │  Voxel IoU │  │
│  └────────────────────────────────────────────────────┘  │
│                         │                                │
│                         ▼                                │
│  ┌────────────────────────────────────────────────────┐  │
│  │         Service Providers (可替换接口)               │  │
│  │  LocalChallengeProvider  │  LocalScoreProvider     │  │
│  └────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────┘
```

**核心闭环：**

```text
LocalChallengeProvider  →  加载 Challenge（初始/目标发型）
        ↓
Blockly Workspace       →  用户编排积木程序
        ↓
Program Compiler        →  IR → RobotCommand[] 原子命令序列
        ↓
SimulationEngine.tick() →  每帧推进命令执行 & 关节插值
        ↓
RobotController         →  正运动学计算末端位置
HeadCollision           →  逐段检测 SDF 椭球体穿透
ContactDetection        →  末端运动轨迹扫掠 Voxel 剪除
        ↓
R3F Rendering           →  Three.js 实时渲染场景
        ↓
LocalScoreProvider      →  IoU × 效率 × 时间 → 加权总分
```

---

## 技术栈

| 层 | 技术 | 版本 |
|---|---|---|
| 语言 | TypeScript | 6.0 |
| UI 框架 | React | 19.2 |
| 构建 | Vite | 8.1 |
| 3D 渲染 | Three.js / React Three Fiber | 0.185 / 9.6 |
| 3D 工具 | @react-three/drei | 10.7 |
| 积木编程 | Blockly | 13.2 |
| 状态管理 | Zustand | 5.0 |
| 图标 | Lucide React | 1.27 |
| 单元测试 | Vitest + Testing Library | 4.1 / 16.3 |
| E2E 测试 | Playwright | 1.62 |
| Lint | ESLint 9 | 9.39 |

---

## 目录结构

```
hcr-simulator/
├── index.html                          # SPA 入口
├── package.json                        # 依赖 & 脚本
├── vite.config.ts                      # Vite + Vitest 配置
├── playwright.config.ts                # Playwright E2E 配置
├── tsconfig.json / tsconfig.*.json     # TypeScript 配置
├── eslint.config.js                    # ESLint Flat Config
│
├── docs/
│   ├── HCR_Simulator_SPEC_v0.3.md      # 🔴 当前生效产品规格（独立可读）
│   ├── HCR_Simulator_SPEC_v0.2.md      # 历史规格（仅追溯）
│   ├── IMPLEMENTATION_PLAN.md          # 实施计划 & 阶段依赖
│   └── ACCEPTANCE.md                   # 自动化 + 人工验收清单
│
├── src/
│   ├── main.tsx                        # ReactDOM 挂载
│   ├── styles.css                      # 全局样式
│   │
│   ├── types/
│   │   └── domain.ts                   # 🔴 所有领域类型定义（Joint/Voxel/Challenge/Score）
│   │
│   ├── app/
│   │   ├── App.tsx                     # 根组件
│   │   ├── providers.tsx               # Services Context Provider
│   │   ├── servicesContext.ts          # React Context 类型
│   │   └── WorkbenchBootstrap.tsx      # Challenge 加载 → 启动工作台
│   │
│   ├── services/
│   │   ├── contracts.ts                # ChallengeProvider / ScoreProvider 接口
│   │   ├── validation.ts              # 输入校验
│   │   └── local/
│   │       ├── LocalChallengeProvider.ts  # 内置 Challenge 提供者
│   │       └── LocalScoreProvider.ts      # 内置评分提供者
│   │
│   ├── data/
│   │   └── challenges/
│   │       ├── defaultChallenge.ts     # 默认 Challenge 参数（关节/几何/Voxel/评分权重）
│   │       └── starterWorkspace.ts     # 预置 Blockly 安全示例程序
│   │
│   ├── features/
│   │   ├── blockly/
│   │   │   ├── BlocklyEditor.tsx       # Blockly 编辑器 React 组件
│   │   │   ├── blockDefinitions.ts     # 自定义积木定义（set-joint-angle/wait/repeat）
│   │   │   ├── blockConstants.ts       # 积木颜色 & 尺寸常量
│   │   │   ├── workspaceFactory.ts     # Blockly Workspace 工厂
│   │   │   ├── programCompiler.ts      # Blockly → Program IR → RobotCommand[]
│   │   │   └── programTypes.ts         # Program / CompiledProgram / ProgramNode 类型
│   │   │
│   │   ├── simulation/
│   │   │   ├── SimulationEngine.ts     # 🔴 仿真核心：状态机 + 命令执行 + 碰撞 + 评分
│   │   │   ├── programExecutor.ts      # 原子命令队列推进 & 关节运动插值
│   │   │   ├── simulationStore.ts      # Zustand UI 状态（面板开关/目标预览）
│   │   │   ├── SimulationTicker.tsx    # requestAnimationFrame 驱动 tick()
│   │   │   ├── SimulatorCanvas.tsx     # R3F Canvas + 场景组合
│   │   │   ├── useSimulationSnapshot.ts # 订阅引擎快照的 Hook
│   │   │   ├── frameTiming.ts          # 帧时序 & delta 计算
│   │   │   └── webglSupport.ts         # WebGL 可用性检测
│   │   │
│   │   ├── robot/
│   │   │   ├── kinematics.ts           # 🔴 正运动学：5关节旋转链 → 各连杆世界坐标
│   │   │   ├── RobotController.ts      # 关节角度插值驱动 & 命令执行
│   │   │   ├── RobotModel.tsx          # R3F 机械臂 3D 可视化组件
│   │   │   └── headCollision.ts        # 🔴 头部防穿模：线段-椭球体 SDF 碰撞检测
│   │   │
│   │   ├── voxel/
│   │   │   ├── hairGenerator.ts        # 初始/目标发型生成（椭球壳层采样 + trim band）
│   │   │   ├── VoxelHair.tsx           # R3F Voxel 渲染组件
│   │   │   ├── contactDetection.ts     # 末端运动扫掠 → Voxel 剪除
│   │   │   ├── similarity.ts           # Voxel IoU 相似度计算
│   │   │   └── voxelKey.ts             # VoxelKey 编解码工具
│   │   │
│   │   └── scoring/
│   │       └── scoring.ts              # 评分算法：完成度 × 效率 × 时间 → 加权总分
│   │
│   ├── components/
│   │   ├── controls/
│   │   │   └── SimulationControls.tsx  # Run / Pause / Step / Stop / Reset 按钮
│   │   ├── inspector/
│   │   │   └── InspectorPanel.tsx      # 关节角度/末端位置/Voxel/评分面板
│   │   └── layout/
│   │       ├── SimulationWorkbench.tsx # 主工作台布局（Blockly | 3D | Inspector）
│   │       └── LogDrawer.tsx           # 底部日志抽屉
│   │
│   └── test/
│       └── setup.ts                    # Vitest 全局 setup
│
└── tests/
    └── e2e/                            # Playwright E2E 测试
```

---

## 核心模块详解

### 1. 领域类型 (`types/domain.ts`)

所有核心类型的唯一来源。关键类型：

| 类型 | 说明 |
|---|---|
| `ChallengeDefinition` | 完整的 Challenge 定义：机器人配置 + Voxel 配置 + 初始/目标发型 + 积木白名单 + 评分参数 |
| `RobotGeometryConfig` | 机械臂几何：底座位置、连杆长度、工具半径、碰撞参数 |
| `JointConfig` | 单个关节：ID、旋转轴、角度范围、初始角度、速度 |
| `HairstyleDefinition` | 发型：ID + 名称 + Voxel 坐标数组 |
| `ScoreInput` / `ScoreResult` | 评分输入输出 |
| `ProgramMetrics` | 程序指标：源积木数、已执行命令数、估算执行时间 |

### 2. 仿真引擎 (`features/simulation/SimulationEngine.ts`)

7 状态状态机：

```
loading → idle ⇄ running → completed
              ↕         ↘ error
           paused ←       ↓
              ↕         stopped
           (step)
```

**核心方法：**
- `run(compiled)` — 从 idle/stopped 进入 running
- `pause()` / `resume()` — 暂停/恢复
- `step(compiled?)` — 执行单条命令后自动暂停
- `stop()` — 保留现场停止
- `reset()` — 恢复 Challenge 初始状态
- `tick(deltaMs)` — 每帧调用，推进命令执行、关节插值、碰撞检测、Voxel 剪除

**每 tick 处理链：**
1. `ProgramExecutor.advance()` — 推进命令队列 & 关节角度插值
2. 每次关节运动后调用 `findRobotHeadCollision()` — 防穿模检测
3. 运动段间调用 `findSweptVoxelHits()` — 末端轨迹扫掠 Voxel
4. 程序完成时异步调用 `ScoreProvider.score()` — 计算成绩
5. 每 100ms 发布 `SimulationSnapshot` 给 UI 订阅者

### 3. 正运动学 (`features/robot/kinematics.ts`)

五关节旋转链，绕各自旋转轴：

```
baseYaw (绕Y) → shoulderRoll (绕X) → shoulder (绕Z)
    → elbow (绕Z) → wrist (绕Z) → endEffector
```

每个关节的旋转矩阵级联传递到子连杆。末端执行器世界坐标由底座位置逐级叠加变换后的连杆方向得到。

**关节配置：**

| 关节 ID | 名称 | 旋转轴 | 范围 | 初始角度 | 速度 |
|---|---|---|---|---|---|
| `baseYaw` | 底座旋转 | Y | -90°~90° | 0° | 45°/s |
| `shoulderRoll` | 肩部侧倾 | X | -60°~60° | 0° | 60°/s |
| `shoulder` | 肩部俯仰 | Z | -90°~45° | -30° | 60°/s |
| `elbow` | 肘部 | Z | 0°~135° | 60° | 90°/s |
| `wrist` | 腕部 | Z | -90°~90° | 0° | 120°/s |

### 4. 头部防穿模 (`features/robot/headCollision.ts`)

头部建模为**轴对齐椭球体** SDF。将机械臂拆分为 8 个碰撞基元（base / shoulder-joint / upper-arm / elbow-joint / forearm / wrist-joint / tool-shaft / end-effector），每个基元为线段 + 半径的胶囊体。

碰撞检测流程：
1. 线段两端点归一化到椭球体空间（各轴除以 `scale + expansion`）
2. 计算线段上到原点最近的点（clamped t ∈ [0,1]）
3. 最近点距离² ≤ 1 → 碰撞

膨胀参数 `headClearance` 提供额外安全余量。碰撞时：关节停在最后安全角度 → 定位源积木 → 进入 `error` 状态（可在 UI 恢复）。

### 5. Voxel 系统 (`features/voxel/`)

**发型生成 (`hairGenerator.ts`)：**
- 在椭球壳层 `innerBound(0.68) ~ outerBound(1.24)` 内采样整数坐标
- 覆盖 X∈[-6,6] / Y∈[-2,7] / Z∈[-6,6]
- 初始发型 = 全壳层（"厚帽型"）
- 目标发型 = 初始发型去掉 trim band 区域（"对称整齐短发"）
- Trim band 规则定义顶部和侧面需修剪的 Voxel

**接触检测 (`contactDetection.ts`)：**
- 机械臂末端从 `start → end` 的运动段
- 以 `toolRadius` 为半径扫掠线段
- 与当前发型 VoxelSet 求交
- 命中的 Voxel 从 active set 中删除

**相似度 (`similarity.ts`)：**
- Voxel IoU = |result ∩ target| / |result ∪ target|
- 用作评分中的完成度分数

### 6. Blockly 系统 (`features/blockly/`)

**自定义积木（3 种）：**
1. `set-joint-angle` — 设置关节绝对角度（下拉选关节 + 数字输入）
2. `wait` — 等待毫秒数
3. `repeat` — 重复执行内部积木 N 次

**编译流程 (`programCompiler.ts`)：**
```
Blockly Workspace → Blockly JSON AST
    → 递归遍历 ProgramNode 树
    → 展开 repeat 循环
    → 展平为 RobotCommand[] 原子序列
    → CompiledProgram { program, runtimeCommands }
```

### 7. 评分系统 (`features/scoring/scoring.ts`)

三项指标加权求和（总分 0-100）：

| 指标 | 权重 | 计算方式 |
|---|---|---|
| **完成度** | 0.5 | Voxel IoU × 100 |
| **效率** | 0.3 | (参考程序成本 / 实际程序成本) × 100 |
| **时间** | 0.2 | (参考时间 / 估算执行时间) × 100 |

程序成本 = `sourceBlockCount + commandWeight × executedCommandCount`

默认参考值：`referenceProgramCost=34`, `referenceTimeMs=18000`, `commandWeight=0.5`

---

## 数据流

```
 1. WorkbenchBootstrap
    └→ LocalChallengeProvider.getChallenge("default")
       └→ Challenge { robotConfig, voxelConfig, initialHair, targetHair, scoring, ... }

 2. new SimulationEngine(challenge, scoreProvider)
    └→ RobotController(challenge.robotConfig, collisionChecker)
    └→ 加载 initialHair → hairVoxels: Set<VoxelKey>

 3. BlocklyEditor 加载 starterWorkspace

 4. 用户点击 Run
    └→ programCompiler.compile(workspace) → CompiledProgram
    └→ engine.run(compiled) → status = "running"

 5. SimulationTicker (rAF loop)
    └→ engine.tick(deltaMs)
       └→ executor.advance(...)
          ├→ handleCommandStart → log
          ├→ handleMovement → contactDetection → voxel 剪除
          └→ handleCommandComplete → metrics++
       └→ 每 100ms publish() → subscribers 更新 UI

 6. 程序完成
    └→ completeProgram()
       └→ scoreProvider.score({ targetVoxels, resultVoxels, ... })
          └→ calculateVoxelIoU() + calculateScore()
       └→ scoreResult 发布到 UI
```

---

## 本地启动

**要求：** Node.js ≥ 22 + npm

```bash
# 安装依赖
npm install

# 启动开发服务器
npm run dev
```

浏览器打开 `http://localhost:5173`。

### 质量命令

```bash
npm run typecheck    # TypeScript 类型检查
npm run lint         # ESLint 检查
npm test             # Vitest 单元测试
npm run build        # 生产构建（typecheck + vite build）
npm run test:e2e     # Playwright E2E 测试
```

---

## Demo 操作流程

1. 启动后等待 Challenge 与预置 Blockly 程序加载完成
2. 可直接运行含 `shoulderRoll` 的安全示例程序，亦可调整关节角度、Wait、Repeat 积木
3. 点击 **Run** 从初始状态执行；运行中可 **Pause / Resume / Stop**
4. 空闲或暂停状态点击 **Step** 单步执行一条原子命令
5. 右侧面板查看关节角度、末端位置、Voxel 统计和评分
6. **Reset** 恢复初始状态保留 Blockly；底部日志记录所有关键事件
7. 目标发型预览可独立切换显示

---

## 明确不做（首版）

- 后端、账户、工作区持久化、网络依赖
- 真实 ESP/舵机、MQTT、WebSerial、WebBluetooth
- IK 逆运动学、完整物理引擎、机械臂自碰撞、真实发丝或剪刀开合
- 相对角度移动、Cartesian Move、外部 GLB/FBX 资产
- 多人竞赛、移动端专项适配、生产部署

---

## 文档导航

| 文档 | 用途 |
|---|---|
| `AGENTS.md` | Codex / AI 编程代理必须遵守的仓库规则 |
| `docs/HCR_Simulator_SPEC_v0.3.md` | 当前生效、可独立阅读的产品与技术规格 |
| `docs/IMPLEMENTATION_PLAN.md` | 后续编码阶段的实施顺序、模块交付和质量门 |
| `docs/ACCEPTANCE.md` | 自动化与人工验收清单 |
| `docs/HCR_Simulator_SPEC_v0.2.md` | 历史规格，仅用于追溯早期讨论 |

---

## License

MIT（见仓库 LICENSE 文件）
