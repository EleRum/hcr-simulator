"""Analyze Anime Hair Studio OBJ to understand card topology."""
from collections import Counter

path = r'D:\download\chrome\test_head.obj'
verts_all = []
faces_all = []
current_obj = None
objects = {}

with open(path) as f:
    for line in f:
        l = line.strip()
        if l.startswith('o '):
            name = l[2:]
            current_obj = name
            if not name.endswith('_curve'):
                objects[name] = {'verts': [], 'faces': []}
        elif l.startswith('v ') and current_obj and not current_obj.endswith('_curve'):
            p = [float(x) for x in l[2:].split()[:3]]
            objects[current_obj]['verts'].append(p)
        elif l.startswith('f ') and current_obj and not current_obj.endswith('_curve'):
            idx = [int(x.split('/')[0])-1 for x in l[2:].split()]
            objects[current_obj]['faces'].append(idx)

# Analyze first 3 meshes
for name in list(objects.keys())[:3]:
    obj = objects[name]
    vs = obj['verts']
    fs = obj['faces']

    print(f'\n=== {name} ===')
    print(f'Vertices: {len(vs)}, Faces: {len(fs)}')

    zs = [v[2] for v in vs]
    xs = [v[0] for v in vs]
    ys = [v[1] for v in vs]

    # Check if sorted by Z
    z_diffs = [zs[i+1]-zs[i] for i in range(len(zs)-1)]
    decreasing = sum(1 for d in z_diffs if d < 0)
    increasing = sum(1 for d in z_diffs if d > 0)
    print(f'Z: [{min(zs):.3f}, {max(zs):.3f}]  decreases:{decreasing} increases:{increasing}')

    # Check: is this a strip (2 columns alternating) or chain (1 column)?
    # Find faces and check vertex relationships
    quads = [f for f in fs if len(f)==4]
    tris = [f for f in fs if len(f)==3]

    # For a 2-column strip, vertices should alternate: 0=left_edge, 1=right_edge, 2=left_edge, ...
    # Check if consecutive vertices are on opposite sides by checking X spread
    x_changes = [abs(xs[i+1]-xs[i]) for i in range(len(xs)-1)]
    avg_x_change = sum(x_changes) / len(x_changes)
    print(f'Avg |dX| between consecutive verts: {avg_x_change:.4f}')

    # Check face structure: do faces connect 4 consecutive vertices?
    # For a Nx2 strip with quad faces: face i connects (i, i+1, i+3, i+2)
    consecutive_faces = 0
    for f in quads:
        if len(f) == 4:
            s = sorted(f)
            if s[1]-s[0] == 1 and s[3]-s[2] == 1 and s[2]-s[1] >= 1:
                consecutive_faces += 1
    print(f'Faces using 4 consecutive-ish vertices: {consecutive_faces}/{len(quads)}')

    # Show first 3 faces
    print(f'First 3 faces: {fs[:3]}')

    # Build edge list
    edges = set()
    for f in fs:
        n = len(f)
        for i in range(n):
            a, b = f[i], f[(i+1)%n]
            edges.add((min(a,b), max(a,b)))

    # Check vertex degree distribution
    deg = Counter()
    for a, b in edges:
        deg[a] += 1
        deg[b] += 1
    deg_dist = Counter(deg.values())
    print(f'Vertex degree distribution: {dict(sorted(deg_dist.items()))}')

    # For each vertex, how far apart are its connected neighbors?
    # If vertices are in a chain, connected vertices should be close in index
    edge_gaps = []
    for a, b in edges:
        edge_gaps.append(abs(a-b))
    gap_dist = Counter(edge_gaps)
    print(f'Edge index gap distribution: {dict(sorted(gap_dist.items())[:10])}')

# Also check: do ALL cards have similar vertex counts?
counts = [len(o['verts']) for o in objects.values()]
print(f'\nVertex counts across all {len(objects)} cards: min={min(counts)} max={max(counts)} avg={sum(counts)/len(counts):.0f}')
print(f'Count distribution: {dict(sorted(Counter(counts).items()))}')

# Check Z ranges for each card - do they all decrease from root to tip?
print('\nZ decrease analysis (first 20 vs last 20):')
for name in list(objects.keys())[:5]:
    obj = objects[name]
    zs = [v[2] for v in obj['verts']]
    first_z = sum(zs[:20])/20 if len(zs)>=20 else sum(zs)/len(zs)
    last_z = sum(zs[-20:])/20 if len(zs)>=20 else 0
    print(f'  {name}: first20_avgZ={first_z:.3f} last20_avgZ={last_z:.3f}  decreasing={first_z > last_z}')
