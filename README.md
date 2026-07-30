# HCR Simulator

HCR Simulator 计划实现为一个纯前端 Web 3D 编程 Demo。用户使用 Blockly 编排舵机角度指令，驱动虚拟四关节机械臂；机械臂末端接触 Hair Voxel 时完成剪除，系统随后按目标发型完成度、程序效率和估算执行时间给出成绩。

> **当前状态：文档基线阶段。** 本目录目前只包含已确认的产品、技术、实施和验收文档，尚未创建前端工程或安装依赖。只有在用户明确开始实现阶段后，才应添加源码、构建配置和依赖。

## 文档导航

| 文档 | 用途 |
|---|---|
| `AGENTS.md` | Codex / AI 编程代理必须遵守的仓库规则 |
| `docs/HCR_Simulator_SPEC_v0.3.md` | 当前生效、可独立阅读的产品与技术规格 |
| `docs/IMPLEMENTATION_PLAN.md` | 后续编码阶段的实施顺序、模块交付和质量门 |
| `docs/ACCEPTANCE.md` | 自动化与人工验收清单 |
| `docs/HCR_Simulator_SPEC_v0.2.md` | 历史规格，仅用于追溯早期讨论 |

## 目标 Demo 闭环

```text
Local Challenge Provider
        ↓
Blockly Workspace
        ↓
Program IR → Runtime Commands
        ↓
Simulation Engine → Robot Controller
        ↓                    ↓
Swept Contact          R3F Rendering
        ↓
Hair Voxel State
        ↓
Local Score Provider
        ↓
Score Breakdown / Result
```

目标版本包含：

- React + TypeScript + Vite + React Three Fiber / Three.js + Blockly。
- 一个“厚帽型 → 对称整齐短发”的本地 Challenge。
- 程序化四关节机械臂、头部与 Hair Voxel。
- Run、Pause、Resume、Step、Stop、Reset 和当前积木高亮。
- Voxel IoU、程序效率、估算执行时间与加权总分。
- 面向桌面 Chrome / Edge、最低约 1280×720 的 3D 主视图工作台。

## 未来实现阶段的目标命令

前端工程创建后，应提供以下 npm 命令。当前文档阶段不可执行这些命令。

```bash
npm run dev
npm run typecheck
npm run lint
npm test
npm run build
npm run test:e2e
```

## 明确不做

- 后端、账户、工作区持久化或网络运行前提。
- 真实 ESP / 舵机、MQTT、WebSerial 或 WebBluetooth。
- IK、完整物理引擎、真实发丝或剪刀开合。
- 多人竞赛、移动端专项适配或生产部署。
