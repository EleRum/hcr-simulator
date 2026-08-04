//! hcr_app — HCR S4: Anime hair + PBD physics + cutting.

mod anime_hair;

use bevy::prelude::*;
use bevy::input::mouse::{MouseButtonInput, MouseMotion, MouseWheel};
use bevy::input::keyboard::KeyCode;

use hcr_core::config::SimConfig;
use hcr_core::clipper::RuntimeCommand;
use hcr_core::simulation::Simulation;

#[derive(Resource)]
struct SimRes {
    sim: Simulation,
    original_cards: hcr_core::hair::CardSet,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Startup, setup_hair.after(setup))
        .add_systems(Startup, setup_sim.after(setup_hair))
        .add_systems(Update, camera_orbit)
        .add_systems(Update, step_and_sync)
        .add_systems(Update, handle_input)
        .add_systems(Update, draw_gizmo)
        .run();
}

// ── Camera ──────────────────────────────────────────

#[derive(Resource)]
struct CamState { theta: f32, phi: f32, dist: f32, dragging: bool }

fn camera_orbit(
    mut q: Query<&mut Transform, With<Camera3d>>,
    mut st: ResMut<CamState>,
    mut btns: EventReader<MouseButtonInput>,
    mut motion: EventReader<MouseMotion>,
    mut scroll: EventReader<MouseWheel>,
) {
    for ev in btns.read() { if ev.button == MouseButton::Left { st.dragging = ev.state.is_pressed(); } }
    if st.dragging {
        for ev in motion.read() {
            st.theta -= ev.delta.x * 0.005;
            st.phi = (st.phi - ev.delta.y * 0.005).clamp(-1.5, 1.5);
        }
    }
    for ev in scroll.read() { st.dist = (st.dist - ev.y * 0.3).clamp(0.5, 5.0); }
    let Ok(mut cam) = q.single_mut() else { return };
    let c = Vec3::ZERO;
    cam.translation = c + Vec3::new(
        st.dist * st.phi.cos() * st.theta.cos(),
        st.dist * st.phi.cos() * st.theta.sin(),
        st.dist * st.phi.sin(),
    );
    cam.look_at(c, Vec3::Z);
}

// ── Head ────────────────────────────────────────────

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let (verts, tris) = load_obj("assets/head.obj");
    commands.spawn((
        Mesh3d(meshes.add(build_mesh(&verts, &tris))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.92, 0.78, 0.65), perceptual_roughness: 0.5,
            double_sided: true, ..default()
        })),
        Transform { scale: Vec3::splat(1.25), translation: Vec3::new(-0.10, 0.0, 0.0), ..default() },
    ));
    commands.spawn((PointLight { intensity: 3000.0, ..default() }, Transform::from_xyz(1.0, 2.0, 2.0)));
    commands.spawn((Camera3d::default(), Transform::default()));
    commands.insert_resource(CamState { theta: 1.5, phi: 0.0, dist: 2.0, dragging: false });
}

// ── Hair ────────────────────────────────────────────

#[derive(Resource)]
struct HairSetRes {
    card_set: hcr_core::hair::CardSet,
    handles: Vec<Handle<Mesh>>,
    entities: Vec<Entity>,
    original_indices: Vec<Vec<u32>>,
    original_uvs: Vec<Vec<[f32; 2]>>,
}

fn setup_hair(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let path = r"D:\download\chrome\test_head.obj";
    let hair_set = anime_hair::load_anime_hair(path, &mut meshes);
    let card_set = anime_hair::build_card_set(&hair_set);
    info!("{} pieces, {} verts", hair_set.pieces.len(), hair_set.total_vertices());

    let mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.22, 0.18, 0.28), perceptual_roughness: 0.6,
        double_sided: true, ..default()
    });

    let mut entities = Vec::new();
    let mut idx_save = Vec::new();
    let mut uv_save = Vec::new();
    for (i, (piece, handle)) in hair_set.pieces.iter().zip(hair_set.mesh_handles.iter()).enumerate() {
        let e = commands.spawn((
            Mesh3d(handle.clone()), MeshMaterial3d(mat.clone()),
            Transform::default(),
        )).id();
        entities.push(e);
        idx_save.push(piece.indices.clone());
        uv_save.push(piece.positions.iter().map(|p| [0.0f32, 0.0f32]).collect()); // placeholder UVs
    }

    commands.insert_resource(HairSetRes {
        card_set,
        handles: hair_set.mesh_handles.clone(),
        entities,
        original_indices: idx_save,
        original_uvs: uv_save,
    });
}

// ── Simulation ──────────────────────────────────────

fn setup_sim(mut commands: Commands, hair: Res<HairSetRes>) {
    let init_pos = hcr_core::types::Vec3::new(0.0, 0.4, 0.7);
    let mut cfg = SimConfig::default();
    cfg.clipper.initial_pos = init_pos;
    cfg.clipper.radius = 0.10;
    let mut sim = Simulation::new(cfg);
    sim.clipper.position = init_pos;
    let original = hair.card_set.clone();
    sim.set_cards(original.clone());
    info!("Sim ready: {} cards, clipper at ({:.1},{:.1},{:.1})",
        sim.cards.total_cards(), init_pos.x, init_pos.y, init_pos.z);
    commands.insert_resource(SimRes { sim, original_cards: original });
}

// ── Cutting ─────────────────────────────────────────

fn step_and_sync(
    mut sim_res: ResMut<SimRes>,
    hair: Res<HairSetRes>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
) {
    let sim = &mut sim_res.sim;
    if !sim.clipper.is_cutting { return; }

    let cp = sim.clipper.position;
    let r2 = sim.clipper.radius * sim.clipper.radius;

    for layer in &mut sim.cards.layers {
        for card in layer {
            // hcr_s4 original logic: find first vertex in sphere → record Z
            // → deactivate all vertices with Z <= hit_Z (tip = lower Z)
            // Plus: snap the hit vertex to sphere surface for smooth cut
            let r = sim.clipper.radius;

            // 1. Find highest Z vertex inside sphere (closest to scalp = root)
            let mut hit_z: Option<f32> = None;
            for row in &card.rows {
                for v in row {
                    if v.active && (v.pos - cp).length_squared() < r2 {
                        hit_z = Some(v.pos.z);
                        break;
                    }
                }
                if hit_z.is_some() { break; }
            }

            if let Some(hz) = hit_z {
                // 2. Snap the hit vertex (and others at same Z) to sphere surface
                for row in &mut card.rows {
                    for v in row {
                        if v.active && (v.pos.z - hz).abs() < 0.001 {
                            let dir = v.pos - cp;
                            let d = dir.length();
                            if d > 1e-10 {
                                v.pos = cp + dir * (r / d);
                                v.prev_pos = v.pos;
                            }
                        }
                    }
                }

                // 3. Deactivate all vertices with Z <= hit_Z (tipward)
                for row in &mut card.rows {
                    for v in row {
                        if v.pos.z < hz - 0.0001 { v.active = false; }
                    }
                }
                card.is_cut = true;
            }
        }
    }

    // Rebuild meshes for cut cards (trim inactive vertices)
    let cut_count = sim.cards.layers.iter().flat_map(|l| l.iter()).filter(|c| c.is_cut).count();
    if cut_count > 0 { info!("rebuilding {cut_count} cut cards"); }
    let mut commands = commands;
    rebuild_cut_meshes(sim, &hair, &mut meshes, &mut commands);
    sim.tick += 1;
}

/// Closest point on segment AB to cp. Returns Z at intersection if distance² < r2.
fn segment_sphere_z(a: hcr_core::types::Vec3, b: hcr_core::types::Vec3, cp: hcr_core::types::Vec3, r2: f32) -> Option<f32> {
    let ab = b - a;
    let len2 = ab.length_squared();
    if len2 < 1e-12 { return None; }
    let t = ((cp - a).dot(ab) / len2).clamp(0.0, 1.0);
    let closest = a + ab * t;
    if (closest - cp).length_squared() < r2 { Some(closest.z) } else { None }
}

fn min_opt(a: Option<f32>, b: Option<f32>) -> Option<f32> {
    match (a, b) {
        (Some(x), Some(y)) => Some(x.min(y)),
        (Some(x), None) => Some(x),
        (None, Some(y)) => Some(y),
        (None, None) => None,
    }
}

fn rebuild_cut_meshes(
    sim: &Simulation,
    hair: &HairSetRes,
    meshes: &mut Assets<Mesh>,
    commands: &mut Commands,
) {
    use bevy::mesh::{Indices, PrimitiveTopology};
    use bevy::asset::RenderAssetUsages;

    for (i, entity) in hair.entities.iter().enumerate() {
        if i >= sim.cards.layers[0].len() { continue; }
        let card = &sim.cards.layers[0][i];
        if !card.is_cut { continue; }

        let orig_idx = &hair.original_indices[i];
        let total_verts = card.rows.len() * card.rows[0].len();

        // Collect active vertices and build old→new index map
        let mut new_positions: Vec<[f32; 3]> = Vec::new();
        let mut old_to_new: Vec<Option<u32>> = vec![None; total_verts];
        let mut vi = 0usize;
        for row in &card.rows {
            for v in row {
                if v.active {
                    old_to_new[vi] = Some(new_positions.len() as u32);
                    new_positions.push([v.pos.x, v.pos.y, v.pos.z]);
                }
                vi += 1;
            }
        }

        if new_positions.len() < 3 {
            info!("card {}: skip rebuild (<3 active)", card.name);
            continue;
        }

        // Remap indices: keep only triangles where all 3 vertices are active
        let mut new_indices: Vec<u32> = Vec::new();
        for tri in orig_idx.chunks(3) {
            if tri.len() < 3 { continue; }
            if let (Some(a), Some(b), Some(c)) = (
                old_to_new[tri[0] as usize],
                old_to_new[tri[1] as usize],
                old_to_new[tri[2] as usize],
            ) {
                new_indices.push(a);
                new_indices.push(b);
                new_indices.push(c);
            }
        }
        info!("card {}: {} active / {} total, {} → {} tris",
            card.name, new_positions.len(), total_verts, orig_idx.len()/3, new_indices.len()/3);

        // Compute normals
        let mut normals = vec![[0.0f32; 3]; new_positions.len()];
        for tri in new_indices.chunks(3) {
            if tri.len() < 3 { continue; }
            let a = Vec3::from_array(new_positions[tri[0] as usize]);
            let b = Vec3::from_array(new_positions[tri[1] as usize]);
            let c = Vec3::from_array(new_positions[tri[2] as usize]);
            let n = (b - a).cross(c - a).normalize().to_array();
            for &t in tri { normals[t as usize][0] += n[0]; normals[t as usize][1] += n[1]; normals[t as usize][2] += n[2]; }
        }
        for n in &mut normals { let l = (n[0]*n[0]+n[1]*n[1]+n[2]*n[2]).sqrt(); if l > 0.0 { n[0]/=l; n[1]/=l; n[2]/=l; } }

        let new_mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD)
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, new_positions)
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
            .with_inserted_indices(Indices::U32(new_indices));

        let new_handle = meshes.add(new_mesh);
        commands.entity(*entity).insert(Mesh3d(new_handle));
    }
}

// ── Input ───────────────────────────────────────────

fn handle_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut sim_res: ResMut<SimRes>,
    hair: Res<HairSetRes>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
) {
    let need_regenerate = keys.just_pressed(KeyCode::KeyR);
    let orig = if need_regenerate { Some(sim_res.original_cards.clone()) } else { None };

    let sim = &mut sim_res.sim;
    if keys.just_pressed(KeyCode::Space) {
        sim.apply_command(&RuntimeCommand::ToggleCutting);
        info!("Cut: {}", if sim.clipper.is_cutting {"ON"} else {"OFF"});
    }
    if let Some(cards) = orig {
        sim.cards = cards;
        sim.debris.clear();
        sim.tick = 0;
        // Restore all cards to active state and swap back original meshes
        for layer in &mut sim.cards.layers {
            for card in layer {
                card.is_cut = false;
                for row in &mut card.rows {
                    for v in row { v.active = true; }
                }
            }
        }
        for (i, entity) in hair.entities.iter().enumerate() {
            commands.entity(*entity).insert(Mesh3d(hair.handles[i].clone()));
        }
        info!("Hair regenerated");
    }
    if keys.just_pressed(KeyCode::Digit1) { sim.apply_command(&RuntimeCommand::SetGuard(1)); }
    if keys.just_pressed(KeyCode::Digit3) { sim.apply_command(&RuntimeCommand::SetGuard(3)); }
    if keys.just_pressed(KeyCode::Digit5) { sim.apply_command(&RuntimeCommand::SetGuard(5)); }
    if keys.just_pressed(KeyCode::Digit8) { sim.apply_command(&RuntimeCommand::SetGuard(8)); }

    let s = if keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) { 0.04 } else { 0.02 };
    if keys.any_pressed([KeyCode::KeyW])     { sim.clipper.position.y -= s; }
    if keys.any_pressed([KeyCode::KeyS])     { sim.clipper.position.y += s; }
    if keys.any_pressed([KeyCode::KeyA])     { sim.clipper.position.x += s; }
    if keys.any_pressed([KeyCode::KeyD])     { sim.clipper.position.x -= s; }
    if keys.any_pressed([KeyCode::KeyQ])     { sim.clipper.position.z += s; }
    if keys.any_pressed([KeyCode::KeyE])     { sim.clipper.position.z -= s; }
}

// ── Gizmo ───────────────────────────────────────────

fn draw_gizmo(sim_res: Res<SimRes>, mut gizmos: Gizmos) {
    let s = &sim_res.sim;
    let p = Vec3::new(s.clipper.position.x, s.clipper.position.y, s.clipper.position.z);
    let c = if s.clipper.is_cutting { Color::srgb(0.2, 1.0, 0.2) }
        else { Color::srgb(0.6, 0.6, 0.8) };
    gizmos.sphere(p, s.clipper.radius, c);
    gizmos.sphere(p, s.clipper.radius + s.clipper.safety_margin, Color::srgba(1., 1., 1., 0.3));

    for frag in &s.debris {
        for pt in frag {
            gizmos.sphere(Vec3::new(pt.x, pt.y, pt.z), 0.003, Color::srgb(0.5, 0.3, 0.1));
        }
    }
}

// ── OBJ helpers ─────────────────────────────────────

fn load_obj(path: &str) -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
    let text = std::fs::read_to_string(path).unwrap();
    let mut verts: Vec<[f32; 3]> = Vec::new();
    let mut tris: Vec<[u32; 3]> = Vec::new();
    for line in text.lines() {
        let l = line.trim();
        if l.starts_with("v ") {
            let p: Vec<f32> = l[2..].split_whitespace().filter_map(|s| s.parse().ok()).collect();
            if p.len() >= 3 { verts.push([p[0], p[2], p[1]]); }
        } else if l.starts_with("f ") {
            let idx: Vec<u32> = l[2..].split_whitespace()
                .filter_map(|s| s.split('/').next()?.parse::<u32>().ok())
                .map(|i| if i > 0 { i - 1 } else { 0 }).collect();
            if idx.len() == 3 { tris.push([idx[0], idx[1], idx[2]]); }
            else if idx.len() >= 4 { tris.push([idx[0], idx[1], idx[2]]); tris.push([idx[0], idx[2], idx[3]]); }
        }
    }
    let n = verts.len() as f32;
    let cx = verts.iter().map(|v| v[0]).sum::<f32>() / n;
    let cy = verts.iter().map(|v| v[1]).sum::<f32>() / n;
    let cz = verts.iter().map(|v| v[2]).sum::<f32>() / n;
    for v in &mut verts { v[0] -= cx; v[1] -= cy; v[2] -= cz; }
    (verts, tris)
}

fn build_mesh(verts: &[[f32; 3]], tris: &[[u32; 3]]) -> Mesh {
    use bevy::mesh::{Indices, PrimitiveTopology};
    use bevy::asset::RenderAssetUsages;
    let positions = verts.to_vec();
    let mut normals = vec![[0.0f32; 3]; positions.len()];
    for tri in tris {
        let a = Vec3::from_array(positions[tri[0] as usize]);
        let b = Vec3::from_array(positions[tri[1] as usize]);
        let c = Vec3::from_array(positions[tri[2] as usize]);
        let n = (b - a).cross(c - a).normalize().to_array();
        for &vi in tri { normals[vi as usize][0] += n[0]; normals[vi as usize][1] += n[1]; normals[vi as usize][2] += n[2]; }
    }
    for n in &mut normals { let l=(n[0]*n[0]+n[1]*n[1]+n[2]*n[2]).sqrt(); if l>0.0{n[0]/=l;n[1]/=l;n[2]/=l;} }
    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::RENDER_WORLD)
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_indices(Indices::U32(tris.iter().flat_map(|t|[t[0],t[1],t[2]]).collect()))
}
