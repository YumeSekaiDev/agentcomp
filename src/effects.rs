//! Effect id aliases and parameter maps for the JSON scene format.

use serde_json::{Map, Value, json};

use crate::scene::{EffectSpec, LayerSpec};

pub fn resolve_effect_id(id: &str) -> &str {
    match id.trim().to_ascii_lowercase().as_str() {
        "glow" | "stylize.glow" => "ec.stylize.glow",
        "blur" | "gaussian" | "gaussianblur" | "gaussian blur" => "ec.blur.gaussian",
        "hue" | "huesaturation" | "hue/saturation" | "color" | "colorcorrect"
        | "color correction" => "ec.color.huesaturation",
        "brightness" | "contrast" | "brightnesscontrast" | "brightness & contrast" => {
            "ec.color.brightnesscontrast"
        }
        "levels" => "ec.color.levels",
        "exposure" => "ec.color.exposure",
        "chroma" | "chromakey" | "chroma key" | "key" | "colorkey" | "linearcolor"
        | "linear color key" => "ec.key.linearcolor",
        "colorkeyobsolete" | "color key" => "ec.key.colorkey",
        // EffectCraft's Keylight-style keyer (Screen Key).
        "screenkey" | "keylight" | "key light" | "key_light" | "screen" => "ec.key.screen",
        "particles" | "particles3d" | "particle" | "particleworld" | "particle emitter 3d"
        | "ccparticleworld" => "ec.sim.ccparticleworld",
        "particles2d" | "particle2d" | "particlesystems2" | "particle emitter 2d"
        | "ccparticlesystems2" => "ec.sim.ccparticlesystems2",
        "stabilizer" | "warpstabilizer" | "warp stabilizer" | "warp" => {
            "ec.distort.warpstabilizer"
        }
        _ => id,
    }
}

pub fn effect_params(effect: &EffectSpec) -> Map<String, Value> {
    let mut params = Map::new();
    let id = resolve_effect_id(&effect.id);

    if let Some(v) = effect.blurriness {
        params.insert("blurriness".into(), json!(v));
    }
    if let Some(v) = effect.radius {
        params.insert("radius".into(), json!(v));
    }
    if let Some(v) = effect.intensity {
        params.insert("intensity".into(), json!(v));
    }
    if let Some(v) = effect.threshold {
        params.insert("threshold".into(), json!(v));
    }
    if let Some(v) = effect.brightness {
        params.insert("brightness".into(), json!(v));
    }
    if let Some(v) = effect.contrast {
        params.insert("contrast".into(), json!(v));
    }
    if let Some(v) = effect.hue {
        params.insert("hue".into(), json!(v));
    }
    if let Some(v) = effect.saturation {
        params.insert("saturation".into(), json!(v));
    }
    if let Some(v) = effect.lightness {
        params.insert("lightness".into(), json!(v));
    }
    if let Some(c) = &effect.key_color {
        let key = if id == "ec.key.screen" {
            "screenColor"
        } else {
            "keyColor"
        };
        params.insert(key.into(), json!(c));
    }
    if let Some(v) = effect.tolerance {
        if id == "ec.key.colorkey" {
            params.insert("colorTolerance".into(), json!(v));
        } else if id == "ec.key.screen" {
            params.insert("screenGain".into(), json!(v));
        } else {
            params.insert("tolerance".into(), json!(v));
        }
    }
    if let Some(v) = effect.softness {
        if id == "ec.key.screen" {
            params.insert("screenSoftness".into(), json!(v));
        } else {
            params.insert("softness".into(), json!(v));
        }
    }

    if id == "ec.distort.warpstabilizer" {
        if let Some(v) = effect.smoothness {
            params.insert("stabilization/smoothness".into(), json!(v));
        }
        if let Some(v) = effect.strength {
            // Crop Less ↔ Smooth More: higher values favour smoother motion.
            params.insert("advanced/cropLessSmoothMore".into(), json!(v));
        }
        if let Some(method) = &effect.method {
            let idx = match method.trim().to_ascii_lowercase().as_str() {
                "position" | "0" => 0,
                "positionscalerotation" | "position, scale, rotation" | "psr" | "1" => 1,
                "perspective" | "2" => 2,
                "subspace" | "subspace warp" | "3" => 3,
                _ => 3,
            };
            params.insert("stabilization/method".into(), json!(idx));
        }
    }

    for (k, v) in &effect.params {
        params.insert(k.clone(), v.clone());
    }
    params
}

/// Map friendly particle fields onto Particle Emitter 2D or 3D parameter paths.
pub fn particle_params(layer: &LayerSpec) -> Map<String, Value> {
    let nested = layer.particle.clone().unwrap_or_default();
    let is_2d = layer.wants_particle_2d();
    let mut params = Map::new();

    set_num(
        &mut params,
        "birthRate",
        layer.birth_rate.or(nested.birth_rate),
    );
    set_num(
        &mut params,
        "longevity",
        layer
            .life
            .or(layer.longevity)
            .or(nested.life)
            .or(nested.longevity),
    );
    set_num(
        &mut params,
        "physics/velocity",
        layer.velocity.or(nested.velocity),
    );
    set_num(
        &mut params,
        "physics/gravity",
        layer.gravity.or(nested.gravity),
    );

    let birth_color = layer
        .color
        .as_deref()
        .or(layer.fill.as_deref())
        .or(nested.color.as_deref());
    if let Some(c) = birth_color {
        params.insert("particle/birthColor".into(), json!(c));
    }
    if let Some(c) = layer.death_color.as_deref().or(nested.death_color.as_deref()) {
        params.insert("particle/deathColor".into(), json!(c));
    }

    let birth_size = nested.size.or_else(|| {
        layer
            .size
            .as_ref()
            .and_then(Value::as_f64)
            .filter(|n| *n > 0.0 && *n <= 20.0)
    });
    set_num(&mut params, "particle/birthSize", birth_size);
    set_num(
        &mut params,
        "particle/deathSize",
        layer.death_size.or(nested.death_size),
    );
    set_num(
        &mut params,
        "particle/maxOpacity",
        layer.max_opacity.or(nested.max_opacity),
    );

    if let Some([x, y, z]) = layer.producer.or(nested.producer) {
        if is_2d {
            // Particle Emitter 2D Point params are stored in layer pixels
            // (EffectCraft scales default 0…1 points by layer size on apply).
            let w = layer.width.unwrap_or(1280) as f64;
            let h = layer.height.unwrap_or(720) as f64;
            let (px, py) = if x.abs() <= 1.5 && y.abs() <= 1.5 {
                (x * w, y * h)
            } else {
                (x, y)
            };
            params.insert("producer/position".into(), json!([px, py]));
        } else {
            params.insert("producer/producerX".into(), json!(x));
            params.insert("producer/producerY".into(), json!(y));
            params.insert("producer/producerZ".into(), json!(z));
        }
    }

    let direction = layer.direction.or(nested.direction);
    if is_2d {
        set_num(&mut params, "physics/direction", direction);
    }

    if let Some(mode) = nested.transfer_mode.as_deref() {
        let idx = match mode.to_ascii_lowercase().as_str() {
            "screen" => 1,
            "add" => 2,
            "blackmatte" | "black matte" | "black_matte" => 3,
            _ => 0,
        };
        params.insert("particle/transferMode".into(), json!(idx));
    }

    if !params.contains_key("birthRate") {
        params.insert("birthRate".into(), json!(if is_2d { 4.0 } else { 3.0 }));
    }
    if !params.contains_key("longevity") {
        params.insert("longevity".into(), json!(2.0));
    }
    if !params.contains_key("physics/velocity") {
        params.insert("physics/velocity".into(), json!(if is_2d { 3.0 } else { 2.0 }));
    }
    if !params.contains_key("particle/transferMode") {
        params.insert("particle/transferMode".into(), json!(2)); // Add
    }
    if is_2d && !params.contains_key("producer/position") {
        let w = layer.width.unwrap_or(1280) as f64;
        let h = layer.height.unwrap_or(720) as f64;
        params.insert("producer/position".into(), json!([0.5 * w, 0.75 * h]));
    }

    params
}

fn set_num(params: &mut Map<String, Value>, key: &str, value: Option<f64>) {
    if let Some(v) = value {
        params.insert(key.into(), json!(v));
    }
}

pub fn merge_params(base: Map<String, Value>, over: Map<String, Value>) -> Map<String, Value> {
    let mut out = base;
    for (k, v) in over {
        out.insert(k, v);
    }
    out
}

pub fn is_particle_effect(id: &str) -> bool {
    matches!(
        resolve_effect_id(id),
        "ec.sim.ccparticleworld" | "ec.sim.ccparticlesystems2"
    )
}

pub fn is_warp_stabilizer(id: &str) -> bool {
    resolve_effect_id(id) == "ec.distort.warpstabilizer"
}
