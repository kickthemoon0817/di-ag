# di-ag

**Diagram-as-code for people who'd rather edit text than drag boxes.**

di-ag is a small Rust workspace that compiles a compact `.diag` DSL into
positioned, inspected, themed SVG/PNG diagrams. It has a local web editor
with live preview, a built-in inspection pass that scores your layout for
overlaps / crossings / symmetry, and a drawio-style `.diag.png` format that
embeds the original source into the PNG so a single file is both a shareable
image and an editable source.

```diag
@preset flowchart
@layout direction=TB spacing=60

node start "Start" { shape: rounded_rect, style: { fill: "#4CAF50" } }
node fetch "Fetch Data"
node ok    "Valid?" { shape: diamond }
node save  "Save"
node retry "Retry"

edge start -> fetch
edge fetch -> ok
edge ok -> save  { label: "yes" }
edge ok -> retry { label: "no" }
edge retry -> fetch
```

```bash
di-ag render flow.diag -o flow.diag.png -f png   # self-contained PNG
di-ag extract flow.diag.png                      # reprints the DSL
di-ag serve --open                               # live editor
```

---

## Why

Most diagram tools fall into one of two camps: GUI editors that don't fit in
a git diff (Figma, drawio, Lucidchart), or code-first tools whose layout
engines are famously fragile (mermaid, graphviz). di-ag is the middle path:

- **Text-first** — `.diag` files are human-readable, version-controllable,
  and take 30 seconds to learn.
- **Honest scoring** — every render produces a 0–100 quality score based on
  real defects (overlapping nodes, crossing edges, balance). The score goes
  down when your diagram is bad.
- **Drawio-style sharing** — `.diag.png` is just a PNG with the DSL source
  embedded in an iTXt chunk. Paste the image into a doc, and anyone with the
  CLI can edit it again. No re-export cycle.
- **Multi-strategy layout** — flowcharts use a layered Sugiyama-style pass,
  ER/class diagrams use an orthogonal grid, freeform uses force-directed.

## Install

```bash
git clone https://github.com/kickthemoon0817/di-ag
cd di-ag
cargo build --release
# binary at ./target/release/di-ag
```

## Quick tour

```bash
# Start a new diagram from the built-in template
di-ag init
$EDITOR diagram.diag

# Render to SVG, PNG, or a shareable .diag.png
di-ag render diagram.diag -o diagram.svg
di-ag render diagram.diag -o diagram.png -f png
di-ag render diagram.diag -o diagram.diag.png -f png   # PNG with source embedded

# Inspect layout quality alongside the render
di-ag render diagram.diag --inspect
di-ag render diagram.diag --score-threshold 80        # exit 2 if score < 80

# Validate structural rules (cycles, self-loops, dangling edges, duplicates)
di-ag validate diagram.diag --json

# Round-trip through other formats
di-ag convert diagram.diag --to json
di-ag convert diagram.diag --to yaml
di-ag convert diagram.json --to dsl

# Edit interactively in a browser
di-ag serve --open                                    # http://localhost:3000

# Extract the DSL back out of a shared .diag.png
di-ag extract diagram.diag.png
```

Any command that takes an input also accepts a `.diag.png` file — the CLI
transparently extracts the embedded source.

## The DSL

di-ag's DSL is deliberately small. Everything is one of: a directive, a
node, an edge, a container, or a `let`/`repeat` helper.

```diag
@preset flowchart               # layout strategy (flowchart|sequence|er|class|tree|freeform)
@theme dark                     # built-in theme (light|dark|blueprint|monochrome)
@layout direction=LR spacing=80 # TB|BT|LR|RL and inter-layer spacing

# Simple node — label defaults to id
node hello

# Node with label, shape, size, style
node start "Start" {
    shape: rounded_rect
    size: 120x40
    style: { fill: "#4CAF50", stroke: "#388E3C" }
}

# Edges
edge start -> hello
edge hello -> start { label: "retry" }

# Containers group nodes into a bounded sub-layout
container backend "Backend Services" {
    node api "API Gateway"
    node auth "Auth Service"
    edge api -> auth
}

# String escapes: \n for newline, \" for literal quote, \\ for backslash
node multiline "line one\nline two"
```

Shapes: `rect`, `rounded_rect`, `diamond`, `circle`, `ellipse`, `cylinder`,
`parallelogram`, `hexagon`, `triangle`.

## Commands

| Command | What it does |
|---|---|
| `render` | DSL/JSON/YAML → SVG/PNG. Supports `--theme`, `--layout`, `--inspect`, `--score-threshold`, `--json`. |
| `validate` | Tier-1 structural checks: duplicate ids, dangling edges, cycles, self-loops, labels, orphans. Each violation has a `fix_hint`. |
| `convert` | Between `dsl`, `json`, `yaml`, `svg`, `png`. PNG output auto-embeds source when input was DSL. |
| `fmt` | Formats a `.diag` file in place. `--check` for CI. |
| `init` | Writes a starter `diagram.diag`. |
| `serve` | Launches the local web editor with live preview, theme picker, and inspection panel. |
| `extract` | Reads the embedded DSL back out of a `.diag.png`. |

## Layout quality score

`di-ag render --inspect` returns a report like:

```json
{
  "score": 85.3,
  "metrics": {
    "edge_crossings": 1,
    "node_overlaps": 0,
    "whitespace_efficiency": 0.85,
    "label_readability": 1.0,
    "symmetry": 1.0
  },
  "issues": [
    { "type": "edge_crossing", "edges": ["e3", "e4"], "at": [0, 340],
      "fix_hint": "Reorder nodes to reduce edge crossings" }
  ]
}
```

The score is multiplicative — every overlap halves it, every crossing costs
~8%. Clean layouts hit the high 80s; diagrams with real defects drop fast.
Wire it into CI with `--score-threshold 75` to fail any PR that regresses
layout quality.

## Layout strategies

| Preset | Strategy | Good for |
|---|---|---|
| `flowchart`, `sequence`, `tree` | Layered (Sugiyama-style) with FIFO Kahn + cycle detection | Directed acyclic flows, org charts, call graphs |
| `er`, `class` | Orthogonal grid | Entity-relationship, UML class diagrams |
| `freeform` | Force-directed | Concept maps, network topologies |

All strategies respect `@layout direction=TB/BT/LR/RL`, handle containers
transparently, and share the same inspection/scoring pass.

## Architecture

```
crates/
├── di-ag-ir        # Typed IR: Document, Node, Edge, Container, Preset
├── di-ag-dsl       # Pest grammar and parser for .diag files
├── di-ag-layout    # Layered / force / orthogonal strategies + scoring
├── di-ag-render    # SVG + PNG rendering with themes and .diag.png embed
├── di-ag-validate  # Tier-1 structural lint rules
├── di-ag-inspect   # Aesthetic scoring: crossings, overlaps, symmetry
├── di-ag-cli       # `di-ag` binary (clap-based)
└── di-ag-wasm      # wasm-bindgen glue for browser use
```

The compilation pipeline is a pure function: `DSL -> IR -> layout -> SVG`,
with inspection and validation as independent passes over the IR. Every
stage is a separate crate you can use on its own.

## Status

Pre-1.0 and moving fast. The IR and DSL shapes are stable but the layout
engine is still getting better — expect quality scores for the same input
to go up over time, not down. Open issues and diffs welcome.

## License

MIT.
