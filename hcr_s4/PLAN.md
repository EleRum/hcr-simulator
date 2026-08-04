# HCR S4: Blender 发片 (Hair Cards) 方案

## Context

HCR 项目是一个 K-12 教育游戏：学生用 Blockly 编程控制机械臂剪头发，目标是逼近目标发型。

**`hcr_s4` 是一个全新的 workspace**，专门验证"Blender 制作的发片 + Bevy 渲染 + PBD 模拟"这条技术路线。它不修改 `hcr_s2`/`hcr_s3`，从 hcr_s3 的 Bevy app 骨架起步，替换其 cloth demo 为卡片系统。

### 为什么新开 S4 而非改 S3

- S3 的设计文档和架构有内部矛盾（docs 06 说 "发片不需要贴图"，docs 01/03/08 说 "卡片+纹理"）—— 与其在原地争论，不如用 S4 实际验证
- S4 专注 **"Blender 发片到底能不能跑起来"**，成功后可以合并回 S3 或替代它
- 风险隔离：S4 实验失败不影响 S3 的 Bevy + bevy_silk 主线

### 现有代码参考

**当前 Gen 2 (`hcr_s2`)** 的"发片"是**程序化生成的同心网格**（`hcr_s2\src\sheet.rs` — `HairSheet::generate()` 从头中心射线投射到头模 OBJ，生成 4 层连续三角网格，~1435 顶点）。
**Gen 3 (`hcr_s3`)** 是 Bevy 0.17 + bevy_silk 骨架（cloth demo over head），hcr_core 模块声明但空白。

S4 从 S3 拷贝 Bevy app 骨架，但把 cloth demo 替换为 Blender 卡片系统。

### 为什么值得做

| 当前 S2 (程序化网格) | S4 目标 (Blender 发片) |
|---|---|
| 生成方式粗暴 — 等距投射，无发型感 | 美术可控 — 每张卡片的位置、形状、长度都是设计的 |
| 无纹理 — 纯顶点着色 | 带 alpha 贴图 — 视觉接近真实头发 |
| 网格是连续的 — 切"一个顶点"影响相邻列 | 卡片独立 — 切一张卡片="剪短这一缕" |
| 无法表达发型（刘海、鬓角等由参数模拟） | 自然表达任何发型（刘海=额前的卡片组） |

---

## 推荐方案概述

**卡片 = 模拟几何 + 渲染几何，一个数据源。** 不引入"指南曲线→卡片"中间步骤（Gen 3 docs 的矛盾），卡片自己就是模拟和渲染的最小单元。

每张卡片是 4-8 行的 quad strip（宽度 1-2 个 quad），根部 pinned (`inv_mass = 0`)，通过 Verlet + PBD 约束模拟，同一个顶点 buffer 上传 wgpu 渲染。带 UV 和 alpha 贴图。

## 1. Blender 资产管线（自动化优先）

### 前提：用户 Blender 熟练度 = 入门

用户反馈"不太会用 Blender"，因此方案优先用 **Blender Python 脚本自动化** — 最终用户只需：打开 Blender → 运行脚本 → 导出。大部分工作由脚本和免费插件完成。

### 1.1 推荐自动化方案：Blender Python 脚本

写一个 `generate_hair_cards.py`，放在 `hcr_s4\assets\hair\` 下，自动完成：

```
加载 head.obj → 在头皮上 emit 曲线 → 曲线转卡片 → UV → 分层 → 导出 glTF
```

**脚本工作流**：

1. `bpy.ops.import_scene.obj(filepath="head.obj")` — 导入头模
2. **自动选头皮顶点** — 按法线方向 + 高度范围过滤（theta: 0 到 PI*0.75，法线朝外的顶点），选中 → 创建顶点组 `Scalp`
3. **生成引导曲线** — 在头皮顶点上按 phi 间隔采样，对每条曲线生成贝塞尔路径（NURBS → 转换），根部贴头皮，末端下垂
4. **曲线转卡片** — 对每条曲线：(a) 创建 1-wide quad strip（4-8 行）沿曲线方向，(b) 根部 shrinkwrap 到头模，(c) 尖端渐细
5. **自动 UV** — 所有卡片 unfold 到一个 512×512 UV tile
6. **分层** — 按 theta 范围自动分配卡片到 Collection：头皮层(theta < 0.15) / 内层(0.15-0.5) / 中层(0.5-1.0) / 外层(1.0-1.4) / 刘海（前方卡片）
7. `bpy.ops.export_scene.gltf(filepath="hair_L0.glb", export_selected=True)` — 按层导出

**备选**：如果 Python 自动化曲线→卡片太复杂，改用 **"Hair Cards Maker" 免费插件** 做曲线→卡片这步，脚本只做剩下的（导入头模、UV、分层、导出）。

### 1.2 手动操作备选（如果脚本方案被跳过）

如果只用 Blender UI 手动操作：

1. 安装 "Hair Cards Maker" 插件（免费，Blender Market / GitHub）
2. 导入 `head.obj` → 在 Edit Mode 选头皮面 → 插件 "Generate Hair Cards" → 调整参数
3. Shrinkwrap modifier 吸附根部到头模
4. 手动分组到 Collection，命名 `CARD_L{layer}_{idx:03d}`
5. UV → Smart UV Project → 打包到 atlas
6. 导出 glTF 2.0

### 1.3 建模约束清单

| 约束 | 原因 |
|---|---|
| Quad 拓扑，焊接顶点 | PBD 需要干净边；loader 用 index buffer |
| 根部行 first，贴头皮 | `inv_mass = 0` pinning（同 `sheet.rs:50`）|
| 至少 4 段（5 行）/ 卡 | Bend constraint 需要 triple（`solve_bend`）|
| 卡片 rest pose 不穿插 | 防止初始约束爆炸 |
| 对象命名 `CARD_L{layer}_{idx:03d}` | Loader 解析层和索引 |
| Rest pose = 重力下自然垂坠形状 | Verlet 收敛到 rest，减少 spawn jitter |

### 预算：~384 张卡片

| 层 | 卡片数 | 每卡顶点 | 总顶点 |
|---|---|---|---|
| 头皮 | 48 | 1×3 | ~200 |
| 内层 | 64 | 1×4 | ~320 |
| 中内层 | 96 | 2×6 | ~1300 |
| 外层 | 128 | 2×8 | ~2300 |
| 刘海 | 48 | 1×5 | ~300 |
| **总计** | **384** | | **~4400** |

约 4400 顶点 — 是当前 1435 的 3 倍，但 PBD O(n) 和 wgpu 60 FPS 完全无压力。

## 2. 导出格式：glTF 2.0 (.glb)

- **为什么 glTF 而非 OBJ**：需要 UV、材质、命名网格。OBJ 的 `vt`/`o` 支持碎片化
- **为什么不是 FBX**：Rust/Bevy 无一等支持
- **导出设置**：glTF 2.0，Apply Modifiers ON，贴图嵌入 .glb，每层一个文件（`hair_L0.glb` … `hair_L4.glb`）
- 保留 `hair.blend` 在 repo 作为可编辑源文件

## 3. Loader：glTF → CardSet

新增 `hcr_s4\crates\hcr_app\src\card_loader.rs`：

```
hair_L{i}.glb ──bevy_gltf──▶ Handle<Mesh> per card ──extract CPU data──▶ hcr_core::hair::CardSet
```

- `AssetServer::load("assets/hair/hair_L0.glb")` 在 startup system
- 当 `Assets<Gltf>` resolve，遍历 `gltf.named_meshes`，解析 `CARD_L3_042` → `(layer, index)`
- 从每个 `Mesh` 提取 positions/indices/UVs/顶点色
- **坐标系**：bevy_gltf 自动转 glTF Y-up → Bevy Z-up（需验证，否则在 loader 加 `(x,y,z)→(x,z,-y)` 修正）
- 构建 `CardSet` 交给模拟初始化

新增 `hcr_s4\crates\hcr_core\src\hair.rs` — 纯数据类型（无 I/O）：

```rust
pub struct CardVertex {
    pub pos: Vec3, pub prev_pos: Vec3,
    pub inv_mass: f32, pub active: bool, pub uv: [f32; 2],
}
pub struct HairCard {
    pub layer: u8, pub rows: Vec<Vec<CardVertex>>, pub is_cut: bool,
}
pub struct CardSet {
    pub cards: Vec<HairCard>,
    pub render_data: CardRenderData, // merged buffers
}
```

### 后备方案

如果 glTF 导入有问题，从 hcr_s3 拷贝 OBJ loader（`hcr_s3\crates\hcr_app\src\main.rs:96-121`）加载带 `vt` 和 `o` 的 OBJ — 慢但可行。

## 4. 物理集成

`hcr_s2\src\sheet.rs` 的求解器几乎原样搬到卡片：

- **同结构，更小的网格**：`HairCard.rows[ri][ci]`，`cols ∈ {1, 2}`
- **Verlet 预测** (`step_verlet`) — 不变
- **Stretch 约束** (`solve_stretch`) — 垂直边 = 沿发长，水平边（2-wide 卡片才有）— 复用
- **Bend 约束** (`solve_bend`) — 垂直 bend（沿长度）保留，水平 bend 只在 2-wide 卡片上
- **头部碰撞** (`solve_head_collision`) — `head.contains` + push-out，不变
- **根部 pinning**：rows `ri == 0` → `inv_mass = 0`（同 `sheet.rs:50`）
- **层碰撞**：内层顶点不能突出外层（同 `simulation.rs:99-114`）

### hcr_core PBD vs bevy_silk（两个可选路径）

- **(a) 快速方案**：合并所有卡片为一个 mesh，喂给 `bevy_silk::ClothBuilder` + `StickGeneration::Quads`。卡片岛自动独立。**快速看到效果**
- **(b) 推荐终态**：`hcr_core\src\pbd.rs` 作为唯一物理源，app 把结果写入 mesh position buffer。**无双重求解器漂移，支持 headless 确定性模拟**

建议：先用 (a) 快速验证，`pbd.rs` 落地后切换到 (b)。

## 5. 渲染：带 Alpha 的纹理卡片

- **材质**：`StandardMaterial` + `base_color_texture`（atlas PNG），`alpha_mode: AlphaMode::Mask`（cutoff ~0.4）——零排序成本，鲁棒
- **绘制**：每层一个 merged mesh → 5 次 Draw Call（符合 doc 07 预算）
- **每帧更新**：求解器写 position → mesh attribute → `compute_normals()`
- **剪切效果**：卡片被剪后 tip 行标 `cut` flag → 顶点色加深（断面效果）
- **碎发**：被切顶点 → debris 路径（重力 + 地板），灰色小 mesh
- **LOD**：远距离掉内层，只显外层

## 6. 剪切：从逐个顶点切 → 逐卡片段切

代替当前的"切顶点 → 整列消失"：

```
for layer in outer → inner:
    if 推剪距离头皮 < safety_margin: blocked; return
    for card in layer (active):
        找出所有在推剪球体内的顶点行
        cut_row = 末端最近的行
        停用 cut_row..tip 的行 → 送去 debris
        标记 (cut_row - 1) 行为断面 → 渲染加深
    if 当前层还有卡片与推剪相交: break  // 外层遮挡内层
```

**语义更好**："一缕头发变短" 而非 "网格上戳一个洞"。

## 7. 迁移路径（4 个阶段）

> 用户选择：**跳过程序化卡片阶段，直接从 Blender 开始。**

| 阶段 | 内容 | 关键文件 | 验收标准 |
|---|---|---|---|
| **M0 — 创建 S4 骨架** | 从 hcr_s3 拷贝 Bevy app 骨架到 hcr_s4；复制 `head.obj`；清理 cloth demo 代码；初始化 Cargo workspace | `hcr_s4\Cargo.toml`, `hcr_s4\crates\` | `cargo build` 成功（空 app 窗口 + 加载头模） |
| **M1 — Blender 资产** | 写 `generate_hair_cards.py` 自动化脚本；生成 ~300-400 卡片；atlas PNG；导出 `hair_L0..4.glb`；写操作说明 | `hcr_s4\assets\hair\*`, `hcr_s4\assets\hair\generate_hair_cards.py` | 卡片贴合头模，rest pose 无穿插，glTF 查看器正确显示 |
| **M2 — Loader + 渲染** | 定义 `hcr_core\src\hair.rs` (CardSet/HairCard)；`card_loader.rs` (glTF→CardSet)；`hair_view.rs` (per-layer mesh + alpha 材质) | `hcr_s4\crates\hcr_core\src\hair.rs`, `hcr_s4\crates\hcr_app\src\card_loader.rs`, `hair_view.rs`, `main.rs` | App 以 60FPS 渲染 Blender 卡片在头模上，带 alpha 贴图 |
| **M3 — 物理 + 剪切** | `hcr_core\src\pbd.rs` 卡片拓扑求解器（Verlet + stretch/bend/head-collision）；实现 `hcr_core\src\simulation.rs` + `clipper.rs` 卡片级剪切；debris；断面染色 | `hcr_s4\crates\hcr_core\src\pbd.rs`, `simulation.rs`, `clipper.rs`，`hcr_s4\crates\hcr_app\src\hair_view.rs`（每帧 position 上传） | 一次完整手动理发端到端工作 |
| **M4 — 打磨** | Alpha 模式调优，LOD，可选 `tools\card_bake.rs`（glTF→紧凑 `cards.bin`）；MQTT/HTTP 服务层 | `hcr_s4\crates\hcr_server\*`, `hcr_s4\tools\card_bake.rs` | 验收：60FPS，完整游戏循环 |

### 排序逻辑

M0 先搭骨架（从 S3 拷贝 + 清理），M1 产出 Blender 资产，M2 立即能看到渲染结果（无物理也能转相机看效果），M3 加上物理和剪切，M4 打磨发布。每阶段独立可验收。

## 8. S4 项目结构（目标状态）

```
hcr_s4/
├── Cargo.toml                  # workspace: hcr_core, hcr_app, hcr_server
├── assets/
│   ├── head.obj                # 从 hcr_s3 拷贝的 Blender 头模
│   └── hair/
│       ├── hair.blend          # Blender 源文件
│       ├── generate_hair_cards.py  # 自动化脚本
│       ├── hair_atlas.png      # alpha atlas 贴图
│       ├── hair_L0.glb         # 头皮层卡片
│       ├── hair_L1.glb         # 内层卡片
│       ├── hair_L2.glb         # 中层卡片
│       ├── hair_L3.glb         # 外层卡片
│       └── hair_L4.glb         # 刘海卡片
├── crates/
│   ├── hcr_core/               # 纯 Rust 内核（无 I/O，无 Bevy）
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs          # 模块声明
│   │       ├── types.rs        # Vec3 等基础类型
│   │       ├── hair.rs         # CardSet, HairCard, CardVertex
│   │       ├── pbd.rs          # Verlet + PBD 求解器
│   │       ├── head.rs         # 头模加载 + 碰撞检测
│   │       ├── simulation.rs   # 顶层模拟（卡片 + 推剪 + 碎发）
│   │       ├── clipper.rs      # 推剪状态 + 命令
│   │       ├── config.rs       # 模拟配置
│   │       └── snapshot.rs     # 快照序列化
│   ├── hcr_app/                # Bevy 0.17 前端
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs         # Bevy app 入口
│   │       ├── card_loader.rs  # glTF → CardSet
│   │       ├── hair_view.rs    # 每层 merged mesh + alpha 材质
│   │       ├── head_view.rs    # 头模渲染
│   │       ├── camera.rs       # 轨道相机
│   │       └── clipper_view.rs # 推剪可视化
│   └── hcr_server/             # MQTT/HTTP 服务层（M4）
│       ├── Cargo.toml
│       └── src/
│           └── main.rs
└── tools/
    └── card_bake.rs            # glTF → 紧凑 .bin（M4 可选）
```

### 与 S3 的关系

S4 是 S3 的**实验分支**，不是替代。关系如下：

| 方面 | S3 | S4 |
|---|---|---|
| 头发几何 | 程序化连续网格（1500 顶点） | Blender 制作的独立卡片（4400 顶点） |
| 贴图 | 无（纯顶点色） | alpha atlas |
| 物理 | bevy_silk（计划） | hcr_core PBD（Verlet） |
| 渲染 | wgpu mesh（无 alpha） | wgpu mesh + alpha mask |
| bevy_silk | 是 | 否（可选 M4 切换） |
| 状态 | 骨架、文档完整、代码空白 | 全新 — 从 S3 拷贝骨架起步 |

验证成功后，S4 的 `hcr_core` 和卡片管线可以合并回 S3（或 S4 成为主线）。

## 8. 需要更新的文档

- `docs\06_渲染与资产.md` L66 "发片不需要贴图" → 改为描述 UV + alpha atlas
- `docs\03` 数据流图 → 添加 `CardSet` 作为物理和渲染的共同数据源
- `docs\01\04` 发片网格描述 → 标注网格现在是独立卡片条集合

## 风险与缓解

| 风险 | 缓解 |
|---|---|
| glTF 坐标系（Y-up vs Z-up）| 先用小测试导出验证；loader 中加修正变换；OBJ 后备 |
| bevy_gltf named-mesh 名字冲突 | 强制 `CARD_L{i}_{idx}` 唯一命名；每层一个文件 |
| Alpha Blend 排序 artifact | 先用 `AlphaMode::Mask`（cutoff 0.4）；必要时 per-layer render pass |
| 双重求解器漂移 (bevy_silk vs hcr_core) | M4 收敛到 hcr_core 唯一求解器 |
| Blender 制作成本（K-12 项目）| Hair Cards Maker 插件自动 curve→card；先做 2 层 |

---

## 验证方式

1. **M0**：`cd hcr_s4 && cargo build` — workspace 编译成功；`cargo run -p hcr_app` — 窗口 + 头模渲染
2. **M1**：在 Blender 中运行 `generate_hair_cards.py`，检查生成的卡片 — 无穿插、贴头皮、分层正确；`.glb` 在 glTF 查看器中正确显示 UV 和贴图
3. **M2**：`cargo run -p hcr_app` — 60FPS 渲染 Blender 卡片，带 alpha 贴图，相机可旋转
4. **M3**：手动推剪 → 卡片变短 → 断面显示 → 碎发掉落 → 内层保护；`cargo test -p hcr_core`
5. **M4**：完整游戏循环（Blockly → 命令 → 模拟 → 渲染），profile 确认 60FPS
