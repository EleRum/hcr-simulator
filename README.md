# HCR Simulator

基于 Web 的 3D 理发模拟器。使用 Blockly 可视化编程控制五关节机械臂剪除头发，
支持体素方块和 Anime Hair Studio 发片两种头发模型，真人头模可选。

## 快速开始

```bash
npm install
npm run dev
```

浏览器打开 http://localhost:5173。

## 目录结构

```
hcr-simulator2/
├── index.html                     # SPA 入口
├── package.json                   # 依赖与脚本
│
├── public/
│   ├── head.obj                   # 真人头模 (159K 顶点)
│   └── hair.obj                   # Anime Hair Studio 发片 (20 片, 4K 顶点)
│
├── src/
│   ├── types/domain.ts            # 核心类型定义
│   └── features/
│       ├── blockly/               # Blockly 编辑器、程序编译
│       ├── simulation/            # 仿真引擎 (7 状态状态机)、R3F 渲染
│       ├── robot/                 # 五关节运动学、控制器、头部碰撞检测
│       ├── voxel/                 # 体素头发生成、接触检测、IoU 评分
│       ├── card/                  # 发片加载、接触检测、剪除逻辑
│       └── head/                  # 真人头模 OBJ 渲染
│
├── hcr_s4/                        # Rust workspace (Bevy 3D + PBD 物理)
│   ├── assets/
│   │   ├── head.obj               # 原始头模
│   │   └── hair/                  # hair.blend、生成脚本、hair_L0..L4.glb
│   └── crates/
│       ├── hcr_core/              # 纯 Rust PBD 物理引擎
│       ├── hcr_app/               # Bevy 0.17 桌面应用
│       └── hcr_server/            # Hotaru HTTP/MQTT 服务
│
├── docs/                          # 产品规格与验收文档
└── tests/e2e/                     # Playwright E2E 测试
```

## 显示模式

点击顶栏 **ScanFace** 按钮切换两种模式：

| 模式 | 头模 | 头发 | 剪除方式 |
|------|------|------|---------|
| 默认 | 简笔椭球 | 体素方块 | 体素 AABB 扫掠 |
| hcr_s4 | 真人 OBJ 头模 | Anime Hair Studio 发片 | 顶点-球体检测 |

## 机械臂

| 关节 | 轴 | 角度范围 | 速度 |
|------|-----|---------|------|
| baseYaw | Y | -60°~60° | 60°/s |
| shoulderRoll | X | -45°~45° | 45°/s |
| shoulder | Z | -20°~100° | 45°/s |
| elbow | Z | -135°~10° | 60°/s |
| wrist | Z | -100°~100° | 75°/s |

## 技术栈

| 层 | 技术 |
|----|------|
| 语言 | TypeScript 6 |
| 界面 | React 19 + R3F 9 + drei 10 |
| 3D 引擎 | Three.js 0.185 |
| 可视化编程 | Blockly 13 |
| 状态管理 | Zustand 5 |
| 构建 | Vite 8 |
| Rust 物理 | Bevy 0.17 + hcr_core PBD |
