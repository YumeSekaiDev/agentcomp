//! JSON scene format an AI (or human) can write.

use serde::Deserialize;
use serde_json::{Map, Value, json};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Scene {
    pub name: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub background: String,
    pub fill: String,
    pub font_size: f64,
    pub width: u32,
    pub height: u32,
    pub frame_rate: f64,
    pub duration: f64,
    pub fade: bool,
    pub fade_in: f64,
    pub fade_out: f64,
    /// When `layers` is empty, synthesize a title-card stack.
    pub complex: bool,
    pub output: Option<String>,
    pub layers: Vec<LayerSpec>,
}

impl Default for Scene {
    fn default() -> Self {
        Self {
            name: "Agent Comp".into(),
            title: "Hello, Agent".into(),
            subtitle: None,
            background: "#0a0a12".into(),
            fill: "#ffffff".into(),
            font_size: 96.0,
            width: 1280,
            height: 720,
            frame_rate: 30.0,
            duration: 5.0,
            fade: true,
            fade_in: 1.0,
            fade_out: 1.0,
            complex: false,
            output: None,
            layers: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct LayerSpec {
    /// `text` | `solid` | `shape` | `footage` | `particle` | `camera` | `light` | `null`
    #[serde(rename = "type")]
    pub layer_type: String,
    pub name: Option<String>,
    pub text: Option<String>,
    /// Text point size, shape size `[w,h]`, or particle birth size.
    pub size: Option<Value>,
    pub fill: Option<String>,
    /// Shape stroke colour (`#hex` or `[r,g,b]`); omit or `false` for no stroke.
    pub stroke: Option<Value>,
    /// Shape stroke width in pixels (0 = none).
    pub stroke_width: Option<f64>,
    pub color: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    /// Shape kind: `rect`, `rounded`, `ellipse`, `polygon`, `star`.
    pub kind: Option<String>,
    pub shape_size: Option<[f64; 2]>,
    /// Footage file path (relative to CWD or absolute).
    pub path: Option<String>,
    pub position: Option<[f64; 3]>,
    /// Point of interest for cameras / lights `[x,y,z]`.
    pub poi: Option<[f64; 3]>,
    pub opacity: Option<f64>,
    pub scale: Option<Value>,
    pub rotation: Option<f64>,
    /// Enable the layer's 3D switch.
    pub three_d: bool,
    /// Camera: `oneNode` | `twoNode`. Light: `Parallel` | `Spot` | `Point` | `Ambient`.
    #[serde(rename = "cameraType")]
    pub camera_type: Option<String>,
    pub light_kind: Option<String>,
    pub intensity: Option<f64>,
    pub zoom: Option<f64>,
    pub preset: Option<String>,
    /// Camera-only shorthand: position keyframes `[[t, [x,y,z]], …]` (merged into `keys.position`).
    /// JSON field name is `move`.
    #[serde(rename = "move")]
    pub camera_move: Vec<KeyFrame>,
    /// Particle Emitter 3D controls (also accepted under `particle { ... }`).
    pub birth_rate: Option<f64>,
    pub life: Option<f64>,
    pub longevity: Option<f64>,
    pub velocity: Option<f64>,
    pub gravity: Option<f64>,
    pub death_color: Option<String>,
    pub death_size: Option<f64>,
    pub max_opacity: Option<f64>,
    /// Producer offset in particle-world units `[x,y,z]`.
    pub producer: Option<[f64; 3]>,
    /// Nested particle block (same fields as the shortcuts above).
    pub particle: Option<ParticleSpec>,
    pub keys: LayerKeys,
    /// Single effect (shorthand).
    pub effect: Option<EffectSpec>,
    /// Multiple effects, applied in order.
    pub effects: Vec<EffectSpec>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct ParticleSpec {
    pub birth_rate: Option<f64>,
    pub life: Option<f64>,
    pub longevity: Option<f64>,
    pub velocity: Option<f64>,
    pub gravity: Option<f64>,
    pub color: Option<String>,
    pub death_color: Option<String>,
    pub size: Option<f64>,
    pub death_size: Option<f64>,
    pub max_opacity: Option<f64>,
    pub producer: Option<[f64; 3]>,
    /// Transfer mode: `composite` | `screen` | `add` | `blackMatte`.
    pub transfer_mode: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct LayerKeys {
    pub position: Vec<KeyFrame>,
    pub scale: Vec<KeyFrame>,
    pub rotation: Vec<KeyFrame>,
    pub opacity: Vec<KeyFrame>,
    /// Effect-parameter keys for the first effect / particle system.
    /// Paths are relative to `effects/#1/` (e.g. `birthRate`, `physics/velocity`).
    #[serde(default)]
    pub effects: Map<String, Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct KeyFrame {
    pub time: f64,
    pub value: Value,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct EffectSpec {
    /// Short name (`glow`, `blur`, `hue`, `chroma`, `particles3d`, …) or full `ec.*` id.
    pub id: String,
    pub blurriness: Option<f64>,
    pub radius: Option<f64>,
    pub intensity: Option<f64>,
    pub threshold: Option<f64>,
    pub brightness: Option<f64>,
    pub contrast: Option<f64>,
    pub hue: Option<f64>,
    pub saturation: Option<f64>,
    pub lightness: Option<f64>,
    pub key_color: Option<String>,
    pub tolerance: Option<f64>,
    pub softness: Option<f64>,
    /// Free-form EffectCraft parameter map (overrides convenience fields on conflict).
    #[serde(default)]
    pub params: Map<String, Value>,
    /// Keyframes for this effect's parameters (`{ "blurriness": [[0,0],[2,12]] }`).
    #[serde(default)]
    pub keys: Map<String, Value>,
}

impl LayerSpec {
    pub fn font_size(&self, fallback: f64) -> f64 {
        self.size
            .as_ref()
            .and_then(Value::as_f64)
            .filter(|n| *n > 0.0)
            .unwrap_or(fallback)
    }

    pub fn geom_size(&self) -> [f64; 2] {
        if let Some(s) = self.shape_size {
            return s;
        }
        match &self.size {
            Some(Value::Array(a)) if a.len() >= 2 => [
                a[0].as_f64().unwrap_or(200.0),
                a[1].as_f64().unwrap_or(200.0),
            ],
            Some(Value::Number(n)) => {
                let v = n.as_f64().unwrap_or(200.0);
                [v, v]
            }
            _ => [200.0, 200.0],
        }
    }

    pub fn position2(&self, cx: f64, cy: f64) -> [f64; 2] {
        match self.position {
            Some([x, y, _]) => [x, y],
            None => [cx, cy],
        }
    }

    pub fn position3(&self, cx: f64, cy: f64) -> [f64; 3] {
        self.position.unwrap_or([cx, cy, 0.0])
    }

    pub fn all_effects(&self) -> Vec<EffectSpec> {
        let mut out = Vec::new();
        if let Some(e) = &self.effect {
            out.push(e.clone());
        }
        out.extend(self.effects.clone());
        if self.layer_type.eq_ignore_ascii_case("particle") && out.is_empty() {
            out.push(EffectSpec {
                id: "particles3d".into(),
                ..Default::default()
            });
        }
        out
    }
}

impl Scene {
    pub fn from_prompt(prompt: &str) -> Self {
        let mut scene = Self::default();
        scene.complex = true;
        let trimmed = prompt.trim();
        if trimmed.is_empty() {
            return scene;
        }
        if let Some((title, subtitle)) = trimmed.split_once('|') {
            scene.title = title.trim().to_string();
            let sub = subtitle.trim();
            if !sub.is_empty() {
                scene.subtitle = Some(sub.to_string());
            }
        } else if let Some((title, subtitle)) = trimmed.split_once('\n') {
            scene.title = title.trim().to_string();
            let sub = subtitle.trim();
            if !sub.is_empty() {
                scene.subtitle = Some(sub.to_string());
            }
        } else {
            scene.title = trimmed.to_string();
            scene.subtitle = Some("Agent Composition".into());
        }
        scene
    }

    pub fn from_json_str(raw: &str) -> Result<Self, String> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Ok(Self::default());
        }
        if !trimmed.starts_with('{') {
            if let Ok(Value::String(s)) = serde_json::from_str::<Value>(trimmed) {
                return Ok(Self::from_prompt(&s));
            }
            return Ok(Self::from_prompt(trimmed));
        }
        let mut value: Value =
            serde_json::from_str(trimmed).map_err(|e| format!("invalid scene JSON: {e}"))?;
        normalize_scene_json(&mut value);
        serde_json::from_value(value).map_err(|e| format!("invalid scene JSON: {e}"))
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.width == 0 || self.height == 0 {
            return Err("width and height must be > 0".into());
        }
        if self.frame_rate <= 0.0 {
            return Err("frameRate must be > 0".into());
        }
        if self.duration <= 0.0 {
            return Err("duration must be > 0".into());
        }
        if self.font_size <= 0.0 {
            return Err("fontSize must be > 0".into());
        }
        if self.fade_in < 0.0 || self.fade_out < 0.0 {
            return Err("fadeIn and fadeOut must be >= 0".into());
        }
        for (i, layer) in self.layers.iter().enumerate() {
            match layer.layer_type.to_ascii_lowercase().as_str() {
                "text" | "solid" | "shape" | "footage" | "particle" | "camera" | "light"
                | "null" => {}
                other => {
                    return Err(format!(
                        "layers[{i}]: unknown type `{other}` (expected text|solid|shape|footage|particle|camera|light|null)"
                    ));
                }
            }
            if layer.layer_type.eq_ignore_ascii_case("footage")
                && layer.path.as_deref().unwrap_or("").is_empty()
            {
                return Err(format!("layers[{i}]: footage requires `path`"));
            }
        }
        Ok(())
    }

    pub fn resolved_layers(&self) -> Vec<LayerSpec> {
        if !self.layers.is_empty() {
            return self.layers.clone();
        }
        if self.complex {
            return crate::presets::title_card_layers(self);
        }
        crate::presets::simple_title_layers(self)
    }
}

/// Rewrite shorthand keyframe arrays and 2D positions into the typed shape serde expects.
pub fn normalize_scene_json(value: &mut Value) {
    let Some(layers) = value.get_mut("layers").and_then(Value::as_array_mut) else {
        return;
    };
    for layer in layers {
        // Allow `"position": [x, y]` → `[x, y, 0]`.
        if let Some(Value::Array(pos)) = layer.get_mut("position")
            && pos.len() == 2
        {
            pos.push(json!(0.0));
        }
        if let Some(Value::Array(poi)) = layer.get_mut("poi")
            && poi.len() == 2
        {
            poi.push(json!(0.0));
        }
        if let Some(Value::Array(prod)) = layer.get_mut("producer")
            && prod.len() == 2
        {
            prod.push(json!(0.0));
        }
        if let Some(particle) = layer.get_mut("particle")
            && let Some(Value::Array(prod)) = particle.get_mut("producer")
            && prod.len() == 2
        {
            prod.push(json!(0.0));
        }

        // Camera `move: [[t, pos], …]` → same shape as keyframe lists.
        normalize_key_list(layer.get_mut("move"));

        normalize_key_map(layer.get_mut("keys"));

        for effect_key in ["effect", "effects"] {
            match layer.get_mut(effect_key) {
                Some(Value::Object(obj)) => {
                    normalize_effect_keys(obj.get_mut("keys"));
                }
                Some(Value::Array(arr)) => {
                    for item in arr {
                        if let Some(obj) = item.as_object_mut() {
                            normalize_effect_keys(obj.get_mut("keys"));
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

fn normalize_key_map(keys: Option<&mut Value>) {
    let Some(keys) = keys.and_then(Value::as_object_mut) else {
        return;
    };
    for prop in ["position", "scale", "rotation", "opacity"] {
        normalize_key_list(keys.get_mut(prop));
    }
    if let Some(effects) = keys.get_mut("effects").and_then(Value::as_object_mut) {
        let paths: Vec<String> = effects.keys().cloned().collect();
        for path in paths {
            normalize_key_list(effects.get_mut(&path));
        }
    }
}

fn normalize_effect_keys(keys: Option<&mut Value>) {
    let Some(keys) = keys.and_then(Value::as_object_mut) else {
        return;
    };
    let paths: Vec<String> = keys.keys().cloned().collect();
    for path in paths {
        normalize_key_list(keys.get_mut(&path));
    }
}

fn normalize_key_list(list: Option<&mut Value>) {
    let Some(list) = list.and_then(Value::as_array_mut) else {
        return;
    };
    for item in list.iter_mut() {
        if let Some(arr) = item.as_array()
            && arr.len() >= 2
            && item.get("time").is_none()
        {
            *item = json!({ "time": arr[0], "value": arr[1] });
        }
    }
}

pub fn fade_opacity_keys(duration: f64, fade_in: f64, fade_out: f64) -> Vec<KeyFrame> {
    let half = duration / 2.0;
    let fade_in = fade_in.min(half).max(0.0);
    let fade_out = fade_out.min(half).max(0.0);
    let hold_start = fade_in;
    let hold_end = (duration - fade_out).max(hold_start);

    let pairs: Vec<(f64, f64)> = if fade_in <= 0.0 && fade_out <= 0.0 {
        vec![(0.0, 100.0)]
    } else if fade_in <= 0.0 {
        vec![(0.0, 100.0), (hold_end, 100.0), (duration, 0.0)]
    } else if fade_out <= 0.0 {
        vec![(0.0, 0.0), (hold_start, 100.0)]
    } else {
        vec![
            (0.0, 0.0),
            (hold_start, 100.0),
            (hold_end, 100.0),
            (duration, 0.0),
        ]
    };
    pairs
        .into_iter()
        .map(|(time, value)| KeyFrame {
            time,
            value: json!(value),
        })
        .collect()
}

pub fn parse_key_list(raw: &Value) -> Vec<KeyFrame> {
    let Some(arr) = raw.as_array() else {
        return Vec::new();
    };
    arr.iter()
        .filter_map(|item| {
            if let (Some(time), Some(value)) = (item.get("time"), item.get("value")) {
                return Some(KeyFrame {
                    time: time.as_f64().unwrap_or(0.0),
                    value: value.clone(),
                });
            }
            let a = item.as_array()?;
            if a.len() < 2 {
                return None;
            }
            Some(KeyFrame {
                time: a[0].as_f64().unwrap_or(0.0),
                value: a[1].clone(),
            })
        })
        .collect()
}
