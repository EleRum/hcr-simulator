//! Card hair — loads Anime Hair Studio OBJ, renders as flat card meshes.
//! Ported exactly from hcr_s4 anime_hair.rs + main.rs.

import { useEffect, useMemo, useRef, useState } from 'react';
import * as THREE from 'three';
import { cardStore, setPieces, type CardPiece } from './cardStore';

const OBJ_URL = '/hair.obj';
// Match hcr_s4 main.rs line 105: Color::srgb(0.22, 0.18, 0.28)
const HAIR_COLOR = '#382a40';

// ═══════════════════════════════════════════════════════════
// OBJ Parser — matches hcr_s4 anime_hair.rs load_anime_hair
// ═══════════════════════════════════════════════════════════

interface RawObj {
  name: string;
  posIndices: number[];
  uvIndices: number[];
}

interface ParsedPiece {
  name: string;
  positions: number[];   // flat [x,y,z, ...]
  uvs: number[];         // flat [u,v, ...]
  indices: number[];     // flat [i, ...]
}

async function parseAnimeHairObj(url: string): Promise<{ pieces: ParsedPiece[]; center: [number, number, number] }> {
  const text = await fetch(url).then(r => r.text());
  const lines = text.split('\n');

  const allPositions: [number, number, number][] = [];
  const allUvs: [number, number][] = [];
  const objects: RawObj[] = [];
  let current: RawObj | null = null;

  for (const line of lines) {
    const l = line.trim();
    if (l.startsWith('o ')) {
      if (current && !current.name.endsWith('_curve') && current.posIndices.length > 0) {
        objects.push(current);
      }
      current = { name: l.slice(2), posIndices: [], uvIndices: [] };
    } else if (l.startsWith('vt ')) {
      const p = l.slice(3).split(/\s+/).map(Number);
      if (p.length >= 2) allUvs.push([p[0], p[1]]);
    } else if (l.startsWith('v ')) {
      const p = l.slice(2).split(/\s+/).map(Number);
      if (p.length >= 3) {
        // Y-up OBJ → Y-up Three.js: (x, y, z) stays (x, y, z) — both are Y-up
        allPositions.push([p[0], p[1], p[2]]);
      }
    } else if (l.startsWith('f ') && current) {
      const parts = l.slice(2).split(/\s+/);
      const pi: number[] = [], ui: number[] = [];
      for (const part of parts) {
        const s = part.split('/');
        const pIdx = parseInt(s[0]) || 0;
        const tIdx = parseInt(s[1]) || 0;
        pi.push(pIdx > 0 ? pIdx - 1 : 0);
        ui.push(tIdx > 0 ? tIdx - 1 : 0);
      }
      if (pi.length === 3) {
        current.posIndices.push(pi[0], pi[1], pi[2]);
        current.uvIndices.push(ui[0], ui[1], ui[2]);
      } else if (pi.length >= 4) {
        current.posIndices.push(pi[0], pi[1], pi[2], pi[0], pi[2], pi[3]);
        current.uvIndices.push(ui[0], ui[1], ui[2], ui[0], ui[2], ui[3]);
      }
    }
  }
  if (current && !current.name.endsWith('_curve') && current.posIndices.length > 0) {
    objects.push(current);
  }

  // Compute centroid (matching anime_hair.rs centering)
  const n = allPositions.length;
  let cx = 0, cy = 0, cz = 0;
  for (const p of allPositions) { cx += p[0]; cy += p[1]; cz += p[2]; }
  cx /= n; cy /= n; cz /= n;

  // Build per-object deduplicated vertex buffers (matching anime_hair.rs lines 192-208)
  const pieces: ParsedPiece[] = [];
  for (const obj of objects) {
    const map = new Map<string, number>();
    const positions: number[] = [];
    const uvs: number[] = [];
    const indices: number[] = [];

    for (let k = 0; k < obj.posIndices.length; k++) {
      const pi = obj.posIndices[k];
      const ui = obj.uvIndices[k];
      const key = `${pi},${ui}`;
      let localIdx = map.get(key);
      if (localIdx === undefined) {
        localIdx = positions.length / 3;
        map.set(key, localIdx);
        positions.push(allPositions[pi][0] - cx, allPositions[pi][1] - cy, allPositions[pi][2] - cz);
        const uv = allUvs[ui] || [0, 0];
        uvs.push(uv[0], uv[1]);
      }
      indices.push(localIdx);
    }

    pieces.push({ name: obj.name, positions, uvs, indices });
  }

  console.log(`[CardHair] Loaded ${pieces.length} pieces, ${allPositions.length} verts, center=(${cx.toFixed(3)},${cy.toFixed(3)},${cz.toFixed(3)})`);
  return { pieces, center: [cx, cy, cz] };
}

// ═══════════════════════════════════════════════════════════
// CardPieceMesh — single hair piece with cutting support
// ═══════════════════════════════════════════════════════════

function CardPieceMesh({ piece }: { piece: CardPiece }) {
  const meshRef = useRef<THREE.Mesh>(null);
  const cutVerRef = useRef(0);

  // Build initial geometry
  const geo = useMemo(() => {
    const g = new THREE.BufferGeometry();
    g.setAttribute('position', new THREE.BufferAttribute(piece.positions.slice(), 3));
    g.setIndex(Array.from(piece.indices));
    g.computeVertexNormals();
    return g;
  }, [piece]);

  // Watch for cuts and rebuild geometry (matching hcr_s4 rebuild_cut_meshes)
  useEffect(() => {
    const interval = setInterval(() => {
      if (cardStore.cutVersion === cutVerRef.current) return;
      cutVerRef.current = cardStore.cutVersion;
      if (!piece.isCut || !meshRef.current) return;

      // Map old→new indices for active vertices
      const oldToNew: Map<number, number> = new Map();
      const newPositions: number[] = [];
      for (let vi = 0; vi < piece.positions.length / 3; vi++) {
        if (piece.active[vi]) {
          oldToNew.set(vi, newPositions.length / 3);
          newPositions.push(piece.positions[vi * 3], piece.positions[vi * 3 + 1], piece.positions[vi * 3 + 2]);
        }
      }
      if (newPositions.length < 9) return; // need ≥3 vertices

      // Remap indices: keep triangles where all 3 vertices active
      const newIndices: number[] = [];
      for (let i = 0; i + 2 < piece.indices.length; i += 3) {
        const a = oldToNew.get(piece.indices[i]);
        const b = oldToNew.get(piece.indices[i + 1]);
        const c = oldToNew.get(piece.indices[i + 2]);
        if (a !== undefined && b !== undefined && c !== undefined) {
          newIndices.push(a, b, c);
        }
      }
      if (newIndices.length < 3) return;

      const newGeo = new THREE.BufferGeometry();
      newGeo.setAttribute('position', new THREE.BufferAttribute(new Float32Array(newPositions), 3));
      newGeo.setIndex(newIndices);
      newGeo.computeVertexNormals();
      meshRef.current.geometry.dispose();
      meshRef.current.geometry = newGeo;
    }, 100);
    return () => clearInterval(interval);
  }, [piece]);

  return (
    <mesh ref={meshRef} geometry={geo}>
      <meshStandardMaterial
        color={HAIR_COLOR}
        roughness={0.6}
        side={THREE.DoubleSide}
      />
    </mesh>
  );
}

// ═══════════════════════════════════════════════════════════
// Main component
// ═══════════════════════════════════════════════════════════

export function CardHair({ center, scale = [1, 1, 1] }: { center?: readonly [number, number, number]; scale?: readonly [number, number, number] }) {
  const [pieces, setPiecesState] = useState<CardPiece[] | null>(null);

  useEffect(() => {
    parseAnimeHairObj(OBJ_URL).then(({ pieces: parsed }) => {
      const cardPieces: CardPiece[] = parsed.map(p => ({
        name: p.name,
        positions: new Float32Array(p.positions),
        originalPositions: new Float32Array(p.positions),
        indices: new Uint32Array(p.indices),
        active: new Array(p.positions.length / 3).fill(true),
        isCut: false,
      }));
      setPieces(cardPieces, center as [number, number, number]);
      setPiecesState(cardPieces);
      const totalVerts = cardPieces.reduce((s,p)=>s+p.positions.length/3,0);
      cardPieces.forEach(p => console.log(`  ${p.name}: ${p.positions.length/3} verts, ${p.indices.length/3} tris`));
      console.log(`[CardHair] Ready: ${cardPieces.length} pieces, ${totalVerts} verts, offset=(${center.map(v=>v.toFixed(2))})`);
    });
  }, []);

  if (!pieces) return null;

  return (
    <group position={center} scale={scale}>
      {pieces.map(p => <CardPieceMesh key={p.name} piece={p} />)}
    </group>
  );
}
