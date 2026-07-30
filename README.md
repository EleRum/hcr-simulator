# HCR Simulator

HCR Simulator 计划实现为一个纯前端 Web 3D 编程 Demo。用户使用 Blockly 编排舵机角度指令，驱动虚拟四关节机械臂；机械臂末端接触 Hair Voxel 时完成剪除，系统随后按目标发型完成度、程序效率和估算执行时间给出成绩。

> **当前状态：Phase 1–6 已完成，纯前端 Demo 主闭环可运行；Phase 7 的最终跨浏览器人工验收尚未执行。**

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

## Demo 操作流程

1. 启动应用后等待本地 Challenge 与预置 Blockly 程序加载完成。
2. 可直接运行示例程序，也可在左侧调整关节绝对角度、Wait 与 Repeat 积木。
3. 点击“运行”从 Challenge 初始状态执行；运行中可暂停、继续或停止。
4. 在空闲或暂停状态点击“单步”，每次完整执行一条原子命令。
5. 右侧查看关节、末端位置、voxel、命令数和评分；底部日志记录关键事件。
6. “重置”恢复仿真现场并保留 Blockly 内容；目标发型预览可独立切换。

## 本地启动与质量命令

要求 Node.js 22 和 npm。首次使用先安装依赖：

```bash
npm install
npm run dev
```

工程提供以下质量命令：

```bash
npm run dev
npm run typecheck
npm run lint
npm test
npm run build
npm run test:e2e
```

生产构建输出位于 `dist/`；该目录和测试报告不应提交到仓库。

## 明确不做

- 后端、账户、工作区持久化或网络运行前提。
- 真实 ESP / 舵机、MQTT、WebSerial 或 WebBluetooth。
- IK、完整物理引擎、真实发丝或剪刀开合。
- 多人竞赛、移动端专项适配或生产部署。
