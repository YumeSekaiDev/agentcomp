# agent-compositor

Standalone **headless** motion-graphics tool for AI agents. It drives EffectCraft’s engine and H.264 exporter (no egui UI). An agent writes a small JSON scene (or a prompt); the binary builds the composition and writes an MP4.

## Run

```bash
cd "/run/media/yume/KINGSTON/Effect Craft/agent-compositor"

# Warp Stabilizer + Particle Emitter 2D
cargo run --release -- --input examples/stabilizer-particles.json -o stabilizer-particles.mp4

# Particles + glow + stroked shapes + camera move
cargo run --release -- --input examples/next-test.json -o next-test.mp4

# Rich demo / Particle Emitter 3D / prompt title card
cargo run --release -- --input examples/full-test.json -o full-test.mp4
cargo run --release -- --input examples/particles-test.json -o particles-test.mp4
cargo run --release -- --prompt "Hello | Agent Motion" -o hello.mp4
```

First build pulls EffectCraft’s git deps and can take several minutes.

## Dependencies

| Direct crate | Role |
|---|---|
| `effectcraft-host` | Wired headless `Session` + exporter |
| `effectcraft-engine` | Commands (`comp.new`, `layer.*`, `prop.*`, `effect.apply`, `renderQueue.*`) |
| `serde` / `serde_json` | Scene JSON |

Path deps point at the sibling `../effectcraft` checkout. Keep the `[patch.crates-io]` hayro entries in `Cargo.toml`.

## Scene schema

Top-level (all optional unless noted):

| Field | Default | Meaning |
|---|---|---|
| `name` | `"Agent Comp"` | Comp / queue name |
| `width` / `height` | `1280` / `720` | Frame size |
| `frameRate` / `duration` | `30` / `5` | FPS / length (s) |
| `background` | `"#0a0a12"` | Comp background |
| `layers` | `[]` | Layer list (see below) |
| `output` | — | Default MP4 if `-o` omitted |
| `title` / `subtitle` / `complex` | … | Used when `layers` is empty |

### Layer types

| `type` | Fields |
|---|---|
| `text` | `text`, `size` (pt), `fill` |
| `solid` | `color`, `width`, `height` |
| `shape` | `kind` (`rect`/`ellipse`/…), `size` `[w,h]`, `fill`, `stroke`, `strokeWidth` |
| `footage` | `path`, optional `sequence`, `footageFrameRate` |
| `particle` | Particle Emitter 3D (or 2D with `"emitter": "2d"`) |
| `particle2d` | Particle Emitter 2D host solid + controls |
| `camera` | `cameraType`, `preset`, `zoom`, `poi`, `move` (position keyframes) |
| `light` | `lightKind` (`Point`/`Spot`/`Parallel`/`Ambient`), `color`, `intensity`, `poi` |
| `null` | null object |

Common layer fields: `name`, `position` `[x,y]` or `[x,y,z]`, `opacity`, `scale`, `rotation`, `threeD`, `keys`, `effect` / `effects`.

**Shapes** accept `fill` and optional `stroke` (`#hex`) with `strokeWidth` (px). Example:

```json
{ "type": "shape", "kind": "ellipse", "size": [160, 160],
  "fill": "#38bdf8", "stroke": "#e0f2fe", "strokeWidth": 5 }
```

**Camera move** (dolly / push-in):

```json
{ "type": "camera", "position": [640, 360, -1500], "poi": [640, 360, 0],
  "move": [[0, [640, 360, -1500]], [4, [640, 340, -1280]]] }
```

### Keyframes

```json
"keys": {
  "position": [[0, [100, 200]], [2, [400, 200, -50]]],
  "scale": [[0, 50], [1, 100]],
  "rotation": [[0, 0], [5, 180]],
  "opacity": [[0, 0], [1, 100]],
  "effects": { "birthRate": [[0, 1], [2, 5]] }
}
```

Effect-parameter keys under `keys.effects` target `effects/#1/...`. Per-effect keys also work:

```json
"effect": {
  "id": "blur",
  "keys": { "blurriness": [[0, 0], [2, 20]] }
}
```

### Effects

Shortcuts → EffectCraft ids:

| Shortcut | Id |
|---|---|
| `glow` | `ec.stylize.glow` (`radius`, `intensity`, `threshold`) |
| `blur` | `ec.blur.gaussian` (`blurriness`) |
| `chroma` / `key` | `ec.key.linearcolor` (`keyColor`, `tolerance`, `softness`) |
| `keylight` / `key light` | `ec.key.screen` — Keylight-style (`keyColor` → screen colour) |
| `hue` / `color` | `ec.color.huesaturation` |
| `brightness` | `ec.color.brightnesscontrast` |
| `particles3d` | `ec.sim.ccparticleworld` |
| `particles2d` | `ec.sim.ccparticlesystems2` |
| `stabilizer` | `ec.distort.warpstabilizer` (`smoothness`, `strength`, `method`, `analyze`) |

Glow and blur apply to **any** layer via `effect` or `effects[]`. Key Light / chroma:

```json
{ "id": "keylight", "keyColor": "#00dc28", "tolerance": 100, "softness": 8 }
{ "id": "chroma", "keyColor": "#00ff00", "tolerance": 35, "softness": 18 }
{ "id": "glow", "radius": 24, "intensity": 1.2 }
{ "id": "blur", "blurriness": 6 }
```

Convenience fields: `radius`, `intensity`, `threshold`, `blurriness`, `hue`, `saturation`, `brightness`, `contrast`, `keyColor`, `tolerance`, `softness`. Any other parameter goes in `params`.

### Particle layers (`particle` / `particle2d`)

- **3D** — `type: "particle"` → Particle Emitter 3D (`ec.sim.ccparticleworld`)
- **2D** — `type: "particle2d"` or `"emitter": "2d"` → Particle Emitter 2D (`ec.sim.ccparticlesystems2`)

```json
{
  "type": "particle2d",
  "birthRate": 5,
  "life": 1.4,
  "velocity": 3.5,
  "gravity": 1.2,
  "direction": -90,
  "color": "#fde047",
  "producer": [0.5, 0.85],
  "particle": { "transferMode": "add" }
}
```

| Friendly field | 3D param | 2D param |
|---|---|---|
| `birthRate` | `birthRate` | same |
| `life` | `longevity` | same |
| `velocity` / `gravity` | `physics/velocity` / `gravity` | same |
| `direction` | — | `physics/direction` (degrees) |
| `color` / `deathColor` | `particle/birthColor` / `deathColor` | same |
| `size` / `deathSize` | `particle/birthSize` / `deathSize` | same |
| `producer` | `producer/producerX|Y|Z` (world) | `producer/position` (0…1) |

### Warp Stabilizer

Apply on a **footage** layer (still or sequence). The tool runs `warp.analyze` when `analyze` is true (default).

```json
{
  "type": "footage",
  "path": "examples/media/shake/f_000.png",
  "sequence": true,
  "footageFrameRate": 12,
  "effect": {
    "id": "stabilizer",
    "smoothness": 80,
    "strength": 70,
    "method": "subspace",
    "analyze": true
  }
}
```

| Field | Maps to |
|---|---|
| `smoothness` | `stabilization/smoothness` |
| `strength` | `advanced/cropLessSmoothMore` |
| `method` | `stabilization/method` (`position` / `positionScaleRotation` / `perspective` / `subspace`) |

## What the binary does

1. Parse `--prompt` / `--json` / `--input` into a scene  
2. `effectcraft_host::session()`  
3. `comp.new` → create each layer (`layer.newText` / `newSolid` / `newShape` / `addItem` / `newCamera` / `newLight` / particle solid)  
4. `prop.set` / `prop.addKey`, `layer.setSwitch` (`threeD`), `effect.apply`  
5. `renderQueue.add` + `renderQueue.render` → H.264 MP4  
