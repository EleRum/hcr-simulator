"""
HCR S4 — Blender Hair Card Generator
=====================================
Fully automated: imports head.obj, detects scalp region, generates
hair card quad-strip meshes, UV-unwraps, assigns layers, and exports
per-layer glTF 2.0 (.glb) files.

Usage:
  1. Open Blender
  2. Switch to the "Scripting" workspace
  3. Open this file (Text > Open)
  4. Click "Run Script" (▶) or press Alt+P

  Or from command line:
    blender --background --python generate_hair_cards.py

Output (in the same directory as this script):
  hair_L0.glb   — scalp layer (short fuzz)
  hair_L1.glb   — inner layer
  hair_L2.glb   — middle layer
  hair_L3.glb   — outer layer (longest)
  hair_L4.glb   — fringe (front-facing cards)
  hair.blend    — editable source file

Naming convention:  CARD_L{layer}_{idx:03d}
  Example: CARD_L3_042  →  layer 3 (outer), card index 42
"""

import bpy
import bmesh
import math
import os
import random
from mathutils import Vector, Matrix
from pathlib import Path

# ═══════════════════════════════════════════════════════════
# Configuration
# ═══════════════════════════════════════════════════════════


class Config:
    """Tunable parameters. Edit these to change the hairstyle."""

    # --- Layer definitions ---
    # theta = polar angle from top of head (0 = crown, π/2 = equator)
    LAYERS = [
        # (name, card_count, cols, rows, len_min_m, len_max_m, theta_min, theta_max)
        ("scalp",  48,  1, 4, 0.010, 0.025,  0.00, 0.20),
        ("inner",  64,  1, 5, 0.030, 0.070,  0.15, 0.50),
        ("middle", 96,  2, 6, 0.060, 0.160,  0.40, 0.90),
        ("outer",  128, 2, 8, 0.120, 0.320,  0.80, 1.40),
        ("fringe", 48,  1, 5, 0.040, 0.110,  0.10, 0.60),
    ]

    # --- Scalp detection ---
    SCALP_NORMAL_MAX_ANGLE = math.radians(75)  # max angle from "up"
    SCALP_HEIGHT_MIN_RATIO = 0.55  # ignore bottom 55% of head

    # --- Card geometry ---
    CARD_WIDTH_MM = 6.0       # base card width at root (mm → 0.006 m)
    CARD_WIDTH_TIP_RATIO = 0.35  # tip width / root width
    CARD_CURVE_FACTOR = 0.3   # how much card bends to follow head (0=straight)

    # --- Fringe detection ---
    FRINGE_PHI_RANGE = math.radians(55)  # ±55° around front direction

    # --- UV ---
    UV_ISLAND_MARGIN = 0.002

    # --- Random seed (for reproducible jitter) ---
    SEED = 42


# ═══════════════════════════════════════════════════════════
# Helpers
# ═══════════════════════════════════════════════════════════


def _info(msg):
    print(f"[HCR] {msg}")


def _warn(msg):
    print(f"[HCR] ⚠ {msg}")


def _fatal(msg):
    raise RuntimeError(f"[HCR] FATAL: {msg}")


def _script_dir():
    return Path(__file__).parent.resolve()


def _assets_dir():
    return _script_dir().parent.resolve()


# ═══════════════════════════════════════════════════════════
# Step 1 — Import head.obj
# ═══════════════════════════════════════════════════════════


def import_head(config):
    """Import head.obj. Returns (head_object, head_center_world)."""
    head_path = _assets_dir() / "head.obj"
    if not head_path.exists():
        _fatal(f"head.obj not found at {head_path}")

    _info(f"Importing {head_path} …")

    # Clean scene
    bpy.ops.object.select_all(action="SELECT")
    bpy.ops.object.delete(use_global=False)
    # Purge orphan data
    for block in bpy.data.meshes:
        if block.users == 0:
            bpy.data.meshes.remove(block)

    bpy.ops.wm.obj_import(filepath=str(head_path))

    # Find imported mesh
    head_obj = None
    for obj in bpy.context.selected_objects:
        if obj.type == "MESH":
            head_obj = obj
            break
    if head_obj is None:
        for obj in bpy.data.objects:
            if obj.type == "MESH":
                head_obj = obj
                break
    if head_obj is None:
        _fatal("No mesh found after OBJ import")

    head_obj.name = "Head"

    # Apply any transform so vertex coords are in world space for querying
    bpy.context.view_layer.objects.active = head_obj
    bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)

    mesh = head_obj.data
    n = len(mesh.vertices)
    # Compute center
    cx = sum(v.co.x for v in mesh.vertices) / n
    cy = sum(v.co.y for v in mesh.vertices) / n
    cz = sum(v.co.z for v in mesh.vertices) / n
    center = Vector((cx, cy, cz))

    _info(f"  {n} vertices, {len(mesh.polygons)} faces, center=({cx:.3f},{cy:.3f},{cz:.3f})")

    return head_obj, center


# ═══════════════════════════════════════════════════════════
# Step 2 — Scalp detection
# ═══════════════════════════════════════════════════════════


def detect_scalp(head_obj, head_center, config):
    """Build a vertex group of scalp vertices. Returns (group_index, scalp_vert_indices)."""
    _info("Detecting scalp region …")

    mesh = head_obj.data

    # Use BMesh for accurate normal computation
    bm = bmesh.new()
    bm.from_mesh(mesh)
    bm.verts.ensure_lookup_table()
    bm.faces.ensure_lookup_table()
    bm.normal_update()

    verts = list(bm.verts)
    z_vals = [v.co.z for v in verts]
    z_min, z_max = min(z_vals), max(z_vals)
    z_range = z_max - z_min
    _info(f"  Head Z: [{z_min:.3f}, {z_max:.3f}]")

    up = Vector((0.0, 0.0, 1.0))

    scalp_indices = []
    for v in verts:
        # Average adjacent face normals → vertex normal
        normal = Vector((0.0, 0.0, 0.0))
        for f in v.link_faces:
            normal += f.normal
        if normal.length_squared < 1e-12:
            normal = (v.co - head_center).normalized()
        else:
            normal.normalize()

        angle = math.acos(max(-1.0, min(1.0, normal.dot(up))))
        h = (v.co.z - z_min) / z_range if z_range > 0 else 0.0

        if angle < config.SCALP_NORMAL_MAX_ANGLE and h >= config.SCALP_HEIGHT_MIN_RATIO:
            scalp_indices.append(v.index)

    bm.free()

    # Create vertex group
    vg = head_obj.vertex_groups.new(name="Scalp")
    vg.add(scalp_indices, 1.0, "ADD")

    _info(f"  {len(scalp_indices)} scalp vertices / {len(mesh.vertices)} total")
    return vg, scalp_indices


# ═══════════════════════════════════════════════════════════
# Step 3 — Sample seed points
# ═══════════════════════════════════════════════════════════


def sample_seed_points(head_obj, head_center, vg, config):
    """Build a pool of (position, normal, theta, phi) tuples for each layer.

    Returns list-of-lists: [layer_idx][(pos, normal, theta, phi), …]
    """
    _info("Sampling seed points …")

    mesh = head_obj.data
    up = Vector((0.0, 0.0, 1.0))
    forward = Vector((1.0, 0.0, 0.0))  # Blender +X is typically "front" for a head

    # Gather scalp vert positions & normals
    scalp_data = []
    for v in mesh.vertices:
        for g in v.groups:
            if g.group == vg.index and g.weight > 0.5:
                rel = v.co - head_center
                r = rel.length
                theta = math.acos(max(-1.0, min(1.0, rel.z / r))) if r > 1e-8 else 0.0
                phi = math.atan2(rel.y, rel.x)
                # Approximate normal = direction from center (smoothed)
                normal = rel.normalized()
                scalp_data.append((v.co.copy(), normal, theta, phi))
                break

    _info(f"  Pool: {len(scalp_data)} candidates")

    # For each layer, sample randomly within its theta band
    rng = random.Random(config.SEED)
    layer_pools = []

    for li, (name, count, cols, rows, len_min, len_max, t_min, t_max) in enumerate(config.LAYERS):
        if li == 4:  # fringe — filter by phi (front-facing)
            candidates = [(pos, nrm, theta, phi)
                          for (pos, nrm, theta, phi) in scalp_data
                          if t_min <= theta <= t_max
                          and abs(phi) < config.FRINGE_PHI_RANGE]
        else:
            candidates = [(pos, nrm, theta, phi)
                          for (pos, nrm, theta, phi) in scalp_data
                          if t_min <= theta <= t_max]

        if len(candidates) < count:
            _warn(f"  Layer {li} ({name}): only {len(candidates)} candidates for {count} slots — duplicating")
            # Pad by sampling with replacement
            while len(candidates) < count:
                candidates.append(rng.choice(candidates) if candidates else (
                    Vector((0, 0, 0)), Vector((0, 0, 1)), 0.5, 0.0))

        sampled = rng.sample(candidates, count)
        layer_pools.append(sampled)
        _info(f"  Layer {li} ({name}): {len(sampled)} seeds (theta {t_min:.1f}–{t_max:.1f})")

    return layer_pools


# ═══════════════════════════════════════════════════════════
# Step 4 — Create hair card meshes
# ═══════════════════════════════════════════════════════════


def _create_card_mesh(name, seed_pos, seed_normal, cols, rows,
                      card_length, card_width, config):
    """Create a single hair card as a quad-strip mesh.

    Geometry (side view for 1-wide card, rows=4):

        root (row 0)    tip (row 3)
        ●───●───●───●
        │  /│  /│   │
        ●───●───●───●

    The card starts at seed_pos on the scalp, extends outward along
    seed_normal for card_length, with card_width at the root tapering
    toward the tip.
    """
    rng = random.Random(config.SEED + hash(name) % 10000)

    # Direction vectors
    outward = seed_normal.normalized()

    # Build a local tangent frame
    up = Vector((0.0, 0.0, 1.0))
    if abs(outward.dot(up)) > 0.99:
        up = Vector((1.0, 0.0, 0.0))
    tangent = outward.cross(up).normalized()   # left-right
    bitangent = outward.cross(tangent).normalized()  # up-along-card

    # --- Build vertex grid: rows × (cols+1) vertices ---
    n_vert_rows = rows + 1          # e.g. 4 segments → 5 vert rows
    n_vert_cols = cols + 1          # 1-wide → 2 vert cols, 2-wide → 3

    verts = []
    for ri in range(n_vert_rows):
        t = ri / max(n_vert_rows - 1, 1)  # 0 = root, 1 = tip

        # Row position: start at seed, go outward
        # Add slight forward tilt for gravity look
        tilt = t * 0.3
        row_origin = seed_pos + outward * (card_length * t) - bitangent * (card_length * t * tilt)

        # Optional: curve card back toward head surface near root
        if t < 0.3 and config.CARD_CURVE_FACTOR > 0:
            bend = config.CARD_CURVE_FACTOR * (1.0 - t / 0.3)
            row_origin = row_origin.lerp(seed_pos, bend)

        # Width tapers from root to tip
        half_w = (card_width / 2.0) * (1.0 - t * (1.0 - config.CARD_WIDTH_TIP_RATIO))

        # Add slight random jitter for natural look (reproducible via seed)
        jitter = (rng.random() - 0.5) * card_width * 0.15

        for ci in range(n_vert_cols):
            if n_vert_cols == 1:
                u = 0.0
            else:
                u = (ci / (n_vert_cols - 1)) - 0.5  # -0.5 to +0.5
            pos = row_origin + tangent * (u * card_width + jitter)
            verts.append(pos)

    # --- Build faces (quads) ---
    faces = []
    for ri in range(rows):
        for ci in range(cols):
            # Vert indices for this quad
            v00 = ri * n_vert_cols + ci
            v10 = ri * n_vert_cols + ci + 1
            v01 = (ri + 1) * n_vert_cols + ci
            v11 = (ri + 1) * n_vert_cols + ci + 1
            faces.append((v00, v10, v11, v01))

    # --- Create mesh via BMesh ---
    bm = bmesh.new()
    bm_verts = [bm.verts.new(v) for v in verts]
    bm.verts.ensure_lookup_table()

    for f_verts in faces:
        try:
            bm.faces.new([bm_verts[i] for i in f_verts])
        except ValueError:
            # Duplicate face, skip
            pass

    # Create Blender mesh
    mesh_data = bpy.data.meshes.new(name)
    bm.to_mesh(mesh_data)
    bm.free()

    # Create UV layer
    uv_layer = mesh_data.uv_layers.new(name="UVMap")
    _assign_card_uv(mesh_data, uv_layer, rows, cols)

    return mesh_data


def _assign_card_uv(mesh, uv_layer, rows, cols):
    """Assign simple UVs: u = across card width, v = along card length."""
    n_vert_cols = cols + 1
    n_vert_rows = rows + 1

    for poly in mesh.polygons:
        for loop_idx in poly.loop_indices:
            loop = mesh.loops[loop_idx]
            vi = loop.vertex_index
            ri = vi // n_vert_cols
            ci = vi % n_vert_cols

            u = ci / max(n_vert_cols - 1, 1)
            v = ri / max(n_vert_rows - 1, 1)

            uv_layer.data[loop_idx].uv = (u, v)


def create_all_cards(head_obj, head_center, layer_pools, config):
    """Generate all hair card objects. Returns list-of-lists of Blender objects."""
    _info("Creating hair card meshes …")

    rng = random.Random(config.SEED)
    all_objs = []

    for li, pool in enumerate(layer_pools):
        name, count, cols, rows, len_min, len_max, t_min, t_max = config.LAYERS[li]
        layer_objs = []

        for ci, (pos, normal, theta, phi) in enumerate(pool):
            # Card length: random within layer range
            card_len = len_min + rng.random() * (len_max - len_min)
            card_w = config.CARD_WIDTH_MM / 1000.0

            obj_name = f"CARD_L{li}_{ci:03d}"
            mesh = _create_card_mesh(
                obj_name, pos, normal, cols, rows,
                card_len, card_w, config
            )

            obj = bpy.data.objects.new(obj_name, mesh)
            bpy.context.collection.objects.link(obj)
            layer_objs.append(obj)

        all_objs.append(layer_objs)
        _info(f"  Layer {li} ({name}): {len(layer_objs)} cards")

    return all_objs


# ═══════════════════════════════════════════════════════════
# Step 5 — UV pack into atlas
# ═══════════════════════════════════════════════════════════


def pack_uv_islands(card_objects, config):
    """Pack all cards' UV islands into a shared 0-1 space (atlas)."""
    _info("Packing UV islands …")

    all_meshes = []
    for layer_objs in card_objects:
        for obj in layer_objs:
            all_meshes.append(obj)

    if not all_meshes:
        _warn("No card objects to UV-pack")
        return

    # Select all card objects
    bpy.ops.object.select_all(action="DESELECT")
    for obj in all_meshes:
        obj.select_set(True)
    bpy.context.view_layer.objects.active = all_meshes[0]

    # Smart UV Project — unwrap all together into shared space
    bpy.ops.object.mode_set(mode="EDIT")
    bpy.ops.mesh.select_all(action="SELECT")
    bpy.ops.uv.smart_project(
        angle_limit=math.radians(66),
        island_margin=config.UV_ISLAND_MARGIN,
        area_weight=0.0,
        correct_aspect=True,
        scale_to_bounds=True,
    )
    bpy.ops.object.mode_set(mode="OBJECT")

    _info(f"  UV-packed {len(all_meshes)} meshes")


# ═══════════════════════════════════════════════════════════
# Step 6 — Layer collections
# ═══════════════════════════════════════════════════════════


def organize_layers(card_objects, config):
    """Create Blender collections per layer, move cards into them."""
    _info("Organizing layer collections …")

    collections = []
    for li, (name, count, cols, rows, *_rest) in enumerate(config.LAYERS):
        coll_name = f"Hair_L{li}_{name}"
        coll = bpy.data.collections.new(coll_name)
        bpy.context.scene.collection.children.link(coll)

        for obj in card_objects[li]:
            # Unlink from root collection, link to layer collection
            for c in obj.users_collection:
                c.objects.unlink(obj)
            coll.objects.link(obj)

        collections.append(coll)
        _info(f"  {coll_name}: {len(card_objects[li])} objects")

    # Hide head in viewport (keep for reference)
    head_obj = bpy.data.objects.get("Head")
    if head_obj:
        head_obj.hide_viewport = True

    return collections


# ═══════════════════════════════════════════════════════════
# Step 7 — Export glTF
# ═══════════════════════════════════════════════════════════


def export_gltf_layers(card_objects, config):
    """Export each layer as a separate .glb file."""
    out_dir = _script_dir()
    _info(f"Exporting glTF to {out_dir} …")

    # Deselect all first
    bpy.ops.object.select_all(action="DESELECT")

    for li, (name, count, cols, rows, *_rest) in enumerate(config.LAYERS):
        # Select only cards of this layer
        for obj in card_objects[li]:
            obj.select_set(True)

        out_path = out_dir / f"hair_L{li}.glb"
        bpy.ops.export_scene.gltf(
            filepath=str(out_path),
            use_selection=True,
            export_apply=True,
            export_texcoords=True,
            export_normals=True,
            export_materials='EXPORT',
            export_extras=True,
        )
        _info(f"  {out_path.name}  ({count} cards)")

        # Deselect for next iteration
        for obj in card_objects[li]:
            obj.select_set(False)


# ═══════════════════════════════════════════════════════════
# Step 8 — Save Blender file
# ═══════════════════════════════════════════════════════════


def save_blend(config):
    """Save the working .blend file for later manual editing."""
    out_path = _script_dir() / "hair.blend"
    bpy.ops.wm.save_as_mainfile(filepath=str(out_path))
    _info(f"Saved {out_path}")


# ═══════════════════════════════════════════════════════════
# Main pipeline
# ═══════════════════════════════════════════════════════════


def main():
    _info("=" * 60)
    _info("HCR S4 — Hair Card Generator")
    _info("=" * 60)

    config = Config()

    # 1. Import head
    head_obj, head_center = import_head(config)

    # 2. Detect scalp
    vg, scalp_indices = detect_scalp(head_obj, head_center, config)
    if len(scalp_indices) < 50:
        _warn("Very few scalp vertices detected — check SCALP_ANGLE_THRESHOLD / HEIGHT_MIN")

    # 3. Sample seed points
    layer_pools = sample_seed_points(head_obj, head_center, vg, config)

    # 4. Create card meshes
    card_objects = create_all_cards(head_obj, head_center, layer_pools, config)

    # 5. UV pack
    pack_uv_islands(card_objects, config)

    # 6. Organize into layer collections
    collections = organize_layers(card_objects, config)

    # 7. Export .glb per layer
    export_gltf_layers(card_objects, config)

    # 8. Save .blend
    save_blend(config)

    # Summary
    total = sum(len(layer) for layer in card_objects)
    _info("=" * 60)
    _info(f"DONE — {total} cards across {len(card_objects)} layers")
    _info(f"Output: {_script_dir()}")
    _info("=" * 60)


# ═══════════════════════════════════════════════════════════
# Entry point
# ═══════════════════════════════════════════════════════════

if __name__ == "__main__":
    main()
