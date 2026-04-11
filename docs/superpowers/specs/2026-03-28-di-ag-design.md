# di-ag — Diagram Agent Framework

**Date:** 2026-03-28
**Status:** Draft

## Overview

di-ag is a Rust-based diagram-as-code framework that solves the fundamental problem of AI agents producing ugly, unusable diagrams. It combines a powerful DSL, custom layout engine, and a structured feedback loop so agents can iteratively improve diagram quality — while also serving as a full-featured diagramming tool for humans with an interactive visual editor.

**Core goals:**
1. A diagram-as-code framework for humans (CLI-first, expressive DSL)
2. An agent-friendly tool with a render-feedback loop (structured quality reports, not screenshots)
3. A natural-language-to-diagram agent pipeline (DSL as the generation target)

## 1. Core IR (Intermediate Representation)

The IR is the single source of truth. All inputs (DSL, JSON, YAML, web editor) compile to it. All outputs (SVG, PNG, feedback) read from it.

**Format:** JSON canonical, YAML as alias (parsed to same model).

**Data model:**

```
Document
  metadata
    title: string
    version: string
    theme: string (optional)
  nodes[]
    id: string
    label: string
    type: string (shape preset or custom)
    position: {x, y} (optional — layout engine fills if absent)
    size: {w, h} (optional — auto-calculated from content)
    style: {fill, stroke, font, border-radius, ...}
    ports: string[] (named connection points: top, bottom, left, right, custom)
    children: node[] (recursive — for containers/groups)
  edges[]
    id: string
    source: string (node id, optionally with port: "node_id.port")
    target: string
    label: string (optional)
    waypoints: [{x, y}] (optional — layout engine fills if absent)
    type: straight | curved | orthogonal
    style: {stroke, arrow_head, dash, color, ...}
  presets
    diagram_type: string (flowchart, sequence, er, class, freeform, ...)
    theme_defaults: style map
    layout_defaults: layout config
```

**Key design choices:**
- **Position is optional.** If omitted, the layout engine auto-places. If provided, it's respected. This is how agent-generated and human-edited diagrams coexist.
- **Containers are recursive.** A node can contain child nodes — enables swimlanes, groups, sub-diagrams.
- **Presets are hints, not constraints.** `preset: "flowchart"` applies default styles and suggests a layout strategy, but everything is overridable.
- **Ports enable precise edge routing.** Edges connect to named ports on nodes, not just node centers.

## 2. DSL (Full-Featured Diagram Language)

The DSL is the primary authoring interface for both agents and humans. Expressive enough to handle complex diagrams without dropping to JSON.

**Design principles:**
- Any diagram expressible in the IR is expressible in DSL
- Readable by humans, writable by agents with minimal tokens
- Full access to styling, layout hints, positioning, and containers
- Specialized blocks for diagram types (sequence, ER, class)

**Syntax:**

```
@preset flowchart
@theme dark
@layout direction=TB spacing=40

# Nodes with rich properties
node api_server "API Server" {
  shape: rect
  style: { fill: "#2196F3", stroke: "#1565C0", radius: 8 }
  port: [top, bottom, left, right]
  size: 200x80
}

node db "PostgreSQL" {
  shape: cylinder
  icon: database
}

# Edges with full control
edge api_server -> db {
  label: "queries"
  style: { stroke: dashed, color: gray }
  route: orthogonal
  waypoints: [(300, 200), (300, 400)]
}

# Containers with arbitrary nesting
container backend "Backend Services" {
  style: { fill: "#f5f5f5", border: dashed }
  layout: horizontal

  node auth "Auth"
  node cache "Redis Cache"

  container data_layer "Data Layer" {
    node db
    node queue "Message Queue"
  }

  edge auth -> cache
  edge auth -> data_layer.db
}

# Repeat blocks for dynamic/parameterized structures
repeat 3 as i {
  node worker_${i} "Worker ${i}"
  edge lb -> worker_${i}
}

# Sequence diagram block
sequence {
  actor user "User"
  participant api "API"
  participant db "DB"

  user -> api: "POST /login"
  api -> db: "SELECT user"
  db --> api: "user row"
  api --> user: "200 OK"

  alt "valid credentials" {
    api -> api: "create session"
  } else {
    api --> user: "401"
  }
}

# Variables and reuse
let primary = "#2196F3"
let standard_node = { shape: rect, style: { fill: $primary } }

node svc1 "Service 1" { ...$standard_node }
node svc2 "Service 2" { ...$standard_node }

# Layout directives
align horizontal [svc1, svc2, svc3]
distribute vertical [layer1, layer2, layer3] gap=60
```

**Capabilities:**
- Block syntax `{}` for any element — nodes, edges, containers
- Containers nest arbitrarily — swimlanes, sub-diagrams, grouped components
- Variables and spread `...$var` — reuse styles, reduce repetition
- `repeat` blocks — agents generate parameterized structures
- Diagram-type blocks — `sequence {}`, `er {}`, `class {}` for specialized syntax
- Layout directives — `align`, `distribute`, `layout` for positioning without pixel coords
- Ports — named connection points for precise edge routing
- Cross-container references — `data_layer.db` addresses nested nodes
- Shorthand syntax — `[rect]`, `{diamond}`, `(rounded)` for quick sketches

**Compilation direction:** DSL -> IR is the primary path. The IR is the source of truth. The web editor works on IR, not DSL. `di-ag convert --to dsl` can *generate* DSL from IR for readability (pretty-print), but this is a lossy reconstruction — the generated DSL may not match the original authoring style. The IR remains authoritative.

## 3. Layout Engine

Custom layout algorithms — the core differentiator. Full control over visual quality.

**Pipeline:**

```
IR (nodes + edges + hints)
  -> Strategy selection (based on preset/hints/overrides)
  -> Layout algorithm
  -> Post-passes (edge routing, label placement, spacing, container sizing)
  -> IR (with positions, sizes, waypoints filled)
```

**Algorithms (priority order for v1):**

1. **Layered (Sugiyama-style)** — directed graphs, flowcharts. Minimizes edge crossings, assigns layers.
2. **Orthogonal** — grid-based, architecture diagrams. Clean right-angle edges.
3. **Force-directed** — freeform, organic layouts. Stable via deterministic initialization (no randomness).
4. **Sequence (timeline-based)** — sequence diagrams. Vertical timeline with participant columns.
5. **Tree (hierarchical)** — org charts, class hierarchies.
6. **Constraint solver** — respects explicit `align`/`distribute`/`gap` directives, resolves conflicts.

**Post-passes (always applied):**
- **Edge routing** — avoid crossing through nodes, minimize crossings between edges
- **Label placement** — no occlusion, prefer outside-edge positioning
- **Spacing normalization** — enforce minimum gaps between elements
- **Container sizing** — fit children with padding, respect min-size constraints

**Key design choices:**
- **Preset selects default algorithm.** Flowchart -> layered, sequence -> timeline, freeform -> force-directed. User/agent can override via `@layout`.
- **Post-passes fix manual layouts too.** Even with user-provided positions, the engine prevents overlaps and routes edges cleanly.
- **Incremental layout.** Moving a node in the web editor only re-routes affected edges and adjusts neighbors — no full re-layout.
- **Deterministic output.** Same IR always produces the same layout. Agents can predict the effect of IR changes.
- **Layout scoring.** Internal quality score (edge crossings, overlaps, whitespace, readability) exposed to the feedback system.

## 4. Rendering (SVG + PNG)

**SVG renderer:**
- Renders positioned IR to clean, semantic SVG
- Each node/edge gets `data-id` attribute matching IR id — enables inspection and CSS targeting
- Text uses `<text>` elements with proper font measurement
- Containers render as nested `<g>` groups
- Standalone output (no external dependencies), human-readable markup
- Themes via embedded CSS variables — one SVG, switchable themes

**PNG renderer:**
- SVG as intermediate, rasterized via `resvg` (pure Rust, no system dependencies)
- Configurable DPI (default 2x for retina)
- Transparent or solid background

**Pipeline:**

```
IR (positioned) -> SVG builder -> SVG string
                                      |
                                 +---------+
                                 |         |
                              SVG file   resvg -> PNG file
```

**Theme system:**
- Built-in themes: light, dark, blueprint, monochrome
- Themes are CSS variable maps — custom themes are JSON/YAML color token files
- Presets can declare a default theme

## 5. Agent Feedback System

Hybrid three-tier feedback loop.

### Tier 1 — Constraint Validation (pre-render, fast)

Runs on IR before layout/rendering. Returns structured violations.

```json
{
  "violations": [
    {"type": "overlap", "nodes": ["a", "b"], "severity": "error"},
    {"type": "label_too_long", "node": "c", "label_length": 84, "max": 40, "severity": "warn"},
    {"type": "orphan_node", "node": "d", "severity": "info"},
    {"type": "edge_cycle", "path": ["a", "b", "c", "a"], "severity": "info"}
  ],
  "valid": false
}
```

Checks: overlaps (when positions provided), orphan nodes, label lengths, missing required fields, duplicate IDs, unreachable nodes, container violations.

### Tier 2 — Render Inspection (post-render, detailed)

After layout + SVG render, analyzes visual output geometrically.

```json
{
  "score": 72,
  "issues": [
    {
      "type": "edge_crossing",
      "edges": ["e1", "e3"],
      "at": [240, 180],
      "fix_hint": "reorder nodes in layer 2"
    },
    {
      "type": "label_occlusion",
      "label": "queries",
      "occluded_by": "node_db",
      "fix_hint": "shorten label or increase edge length"
    },
    {
      "type": "dense_region",
      "bounds": [100, 100, 300, 250],
      "node_count": 6,
      "fix_hint": "increase spacing or use a container"
    }
  ],
  "metrics": {
    "edge_crossings": 3,
    "node_overlaps": 0,
    "whitespace_efficiency": 0.68,
    "label_readability": 0.85,
    "symmetry": 0.42
  }
}
```

Every issue includes a `fix_hint` — actionable text the agent can use directly.

### Tier 3 — Vision Review (optional)

CLI outputs PNG + prompt template for vision-capable models. Convenience layer — tiers 1+2 are sufficient for most agents.

### Agent Workflow

```
Agent writes DSL/IR
  -> di-ag validate (tier 1)
  -> fix violations
  -> di-ag render --inspect (tier 2)
  -> read score + issues + fix_hints
  -> adjust IR/DSL
  -> repeat until score > threshold
  -> di-ag render -o output.svg
```

Single command `di-ag render --inspect` runs layout + render + inspection in one pass.

## 6. CLI Interface

**Commands:**

```
di-ag render <input>            # DSL/JSON/YAML -> SVG (default) or PNG
  -o, --output <path>           # Output file (stdout for SVG if omitted)
  -f, --format svg|png          # Output format
  --theme <name>                # Override theme
  --layout <algorithm>          # Override layout algorithm
  --inspect                     # Return JSON feedback report alongside output
  --score-threshold <n>         # Fail if layout score < n (CI mode)

di-ag validate <input>          # Tier 1 validation only, JSON violations
  --strict                      # Treat warnings as errors

di-ag fmt <input>               # Format/prettify DSL files
  --check                       # Check without modifying (CI mode)

di-ag convert <input>           # Format conversion
  --to dsl|json|yaml|svg|png    # Target format
  --from dsl|json|yaml          # Source format (auto-detected if omitted)

di-ag inspect <svg>             # Tier 2 inspection on already-rendered SVG

di-ag serve                     # Launch web UI for interactive editing
  --port <n>                    # Default 3000
  --open                        # Open browser automatically

di-ag init                      # Create starter .diag file with examples

di-ag completions <shell>       # Shell completions (bash/zsh/fish)
```

**Agent-friendly design:**
- All commands output structured JSON when piped (non-TTY detection) or with `--json`
- Meaningful exit codes: 0 = success, 1 = validation errors, 2 = score below threshold
- `stdin` supported everywhere — agents pipe DSL directly, no temp files
- Single command `di-ag render --inspect --json` gives diagram + quality report in one JSON envelope

## 7. Web UI (Interactive Editor)

Optional visual editor served by `di-ag serve`. Same Rust engine compiled to WASM.

**Architecture:**

```
Browser
  TypeScript app (canvas rendering, event handling)
  WASM module (di-ag-ir, di-ag-layout, di-ag-render, di-ag-validate)
  Bidirectional sync: canvas edits <-> IR <-> DSL/JSON preview
```

**Interactions:**
- Drag nodes — repositions in IR, incremental edge re-routing
- Draw edges — click source port, drag to target, edge added to IR
- Resize containers — children reflow automatically
- Select + style — sidebar panel for visual properties
- Inline text editing — double-click node label
- Pan + zoom — infinite canvas

**Round-trip editing:**
- Left panel: live DSL or JSON/YAML editor (syntax highlighted)
- Center: visual canvas
- Edits in either direction sync instantly through shared WASM IR
- No lossy conversion — IR is the single source of truth

**File operations:**
- Open/save `.diag` (DSL), `.json`, `.yaml`
- Export to SVG, PNG
- Import from draw.io XML (best-effort, one-way, warnings for unmapped features)

**Served by `di-ag serve`** — no separate install. CLI bundles web assets.

## 8. Project Structure

```
di-ag/
  Cargo.toml                     # Workspace root
  crates/
    di-ag-ir/                    # Core IR model, JSON/YAML serde
    di-ag-dsl/                   # DSL parser (pest or winnow)
    di-ag-layout/                # Layout algorithms
    di-ag-render/                # SVG builder + resvg PNG rasterizer
    di-ag-validate/              # Tier 1 constraint validation
    di-ag-inspect/               # Tier 2 render inspection + scoring
    di-ag-cli/                   # CLI binary (clap)
    di-ag-wasm/                  # WASM bindings (wasm-bindgen)
  web/                           # TypeScript web UI
    src/
      canvas/                    # Canvas rendering + interaction
      panels/                    # Sidebar, DSL editor, properties
      wasm/                      # WASM bridge layer
    package.json
  docs/
  tests/                         # Integration tests (CLI end-to-end)
  examples/                      # Example .diag files
  LICENSE
  README.md
```

**Key Rust dependencies:**
- `serde` + `serde_json` + `serde_yaml` — IR serialization
- `clap` — CLI argument parsing
- `resvg` — SVG -> PNG rasterization (pure Rust)
- `pest` or `winnow` — DSL parser
- `wasm-bindgen` + `wasm-pack` — WASM compilation

**Web UI:** Vanilla TypeScript or lightweight framework (Lit, Solid, or plain DOM).

**Testing strategy:**
- Unit tests per crate (IR roundtrip, DSL parsing, layout correctness, render output)
- Integration tests: CLI end-to-end (input -> output -> inspection report)
- Visual regression tests: render known diagrams, compare SVG snapshots
- Layout quality tests: assert minimum scores on reference diagrams

## Non-Goals (v1)

- Real-time collaboration (multi-user editing)
- Plugin/extension system for custom shapes
- Animation or interactive diagrams
- Server-side rendering API (CLI only for now)
- draw.io export (import only)

## Success Criteria

1. An agent can generate a diagram via DSL, iterate using the feedback loop, and produce a visually clean result in <5 iterations
2. Layout quality scores consistently above 80 for standard diagram types
3. The DSL can express any diagram that draw.io can, for common diagram types
4. Web editor round-trips edits without data loss
5. CLI renders a 50-node diagram in <500ms
