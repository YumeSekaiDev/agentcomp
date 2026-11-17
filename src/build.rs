//! Turn a [`Scene`] into an EffectCraft composition and render an MP4.

use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use crate::effects::{
    effect_params, is_particle_effect, is_warp_stabilizer, merge_params, particle_params,
    resolve_effect_id,
};
use crate::scene::{EffectSpec, KeyFrame, LayerSpec, Scene, parse_key_list};

pub fn render_scene(scene: &Scene, out: &Path) -> Result<PathBuf, String> {
    let mut session = effectcraft_host::session();
    let layers = scene.resolved_layers();

    exec(
        &mut session,
        "comp.new",
        json!({
            "name": scene.name,
            "width": scene.width,
            "height": scene.height,
            "frameRate": scene.frame_rate,
            "duration": scene.duration,
            "background": scene.background,
        }),
    )?;

    for (index, layer) in layers.iter().enumerate() {
        create_layer(&mut session, scene, layer, index)?;
    }

    exec(
        &mut session,
        "renderQueue.add",
        json!({
            "comp": scene.name,
            "format": "h264",
            "output": out.to_string_lossy(),
        }),
    )?;

    let done = exec(&mut session, "renderQueue.render", json!({ "wait": true }))?;
    if let Some(items) = done.get("items").and_then(Value::as_array) {
        for item in items {
            let status = item.get("status").and_then(Value::as_str).unwrap_or("?");
            if status != "Done" {
                return Err(format!("render queue item not Done: {item}"));
            }
        }
    }

    if !out.is_file() {
        return Err(format!("expected output file missing: {}", out.display()));
    }
    Ok(out.to_path_buf())
}

fn create_layer(
    session: &mut effectcraft_engine::Session,
    scene: &Scene,
    layer: &LayerSpec,
    index: usize,
) -> Result<(), String> {
    let default_name = format!("Layer {}", index + 1);
    let name = layer
        .name
        .as_deref()
        .filter(|s| !s.is_empty())
        .unwrap_or(&default_name);
    let cx = scene.width as f64 / 2.0;
    let cy = scene.height as f64 / 2.0;
    let pos2 = layer.position2(cx, cy);
    let pos3 = layer.position3(cx, cy);

    let created = match layer.layer_type.to_ascii_lowercase().as_str() {
        "text" => {
            let text = layer
                .text
                .as_deref()
                .filter(|s| !s.is_empty())
                .unwrap_or(scene.title.as_str());
            exec(
                session,
                "layer.newText",
                json!({
                    "name": name,
                    "text": text,
                    "size": layer.font_size(scene.font_size),
                    "fill": layer.fill.as_deref().unwrap_or(scene.fill.as_str()),
                    "position": pos2,
                }),
            )?
        }
        "solid" | "particle" | "particle2d" => {
            let w = layer.width.unwrap_or(scene.width);
            let h = layer.height.unwrap_or(scene.height);
            let color = if layer.is_particle_host() {
                layer.color.as_deref().unwrap_or("#000000")
            } else {
                layer
                    .color
                    .as_deref()
                    .unwrap_or(scene.background.as_str())
            };
            exec(
                session,
                "layer.newSolid",
                json!({
                    "name": name,
                    "color": color,
                    "width": w,
                    "height": h,
                }),
            )?
        }
        "shape" => {
            let kind = layer.kind.as_deref().unwrap_or("rect");
            let size = layer.geom_size();
            let mut params = json!({
                "name": name,
                "kind": kind,
                "size": size,
                "fill": layer.fill.as_deref().unwrap_or("#3d8bff"),
                "position": pos2,
            });
            if let Some(stroke) = &layer.stroke {
                // `false` / null → no stroke; otherwise a colour.
                if stroke.is_null() || stroke == &Value::Bool(false) {
                    params["stroke"] = Value::Bool(false);
                    params["strokeWidth"] = json!(0.0);
                } else {
                    params["stroke"] = stroke.clone();
                    params["strokeWidth"] =
                        json!(layer.stroke_width.unwrap_or(4.0).max(0.0));
                }
            } else if let Some(w) = layer.stroke_width.filter(|w| *w > 0.0) {
                params["stroke"] = json!(layer.fill.as_deref().unwrap_or("#ffffff"));
                params["strokeWidth"] = json!(w);
            }
            exec(session, "layer.newShape", params)?
        }
        "footage" => {
            let path = resolve_media_path(layer.path.as_deref().unwrap_or(""))?;
            let imported = exec(
                session,
                "file.import",
                json!({
                    "paths": [path.to_string_lossy()],
                    "sequence": layer.sequence,
                }),
            )?;
            let item = imported
                .get("items")
                .and_then(Value::as_array)
                .and_then(|a| a.first())
                .cloned()
                .ok_or_else(|| format!("layers[{index}]: file.import returned no items"))?;
            if let Some(fps) = layer.footage_frame_rate {
                let _ = exec(
                    session,
                    "file.interpretFootage",
                    json!({ "items": [item.clone()], "frameRate": fps }),
                );
            }
            let added = exec(
                session,
                "layer.addItem",
                json!({
                    "item": item,
                    "position": pos2,
                }),
            )?;
            if let Some(lid) = added.get("layer") {
                let _ = exec(
                    session,
                    "layer.rename",
                    json!({ "layer": lid, "name": name }),
                );
            }
            added
        }
        "camera" => {
            let mut params = json!({
                "name": name,
                "type": layer.camera_type.as_deref().unwrap_or("twoNode"),
                "position": pos3,
            });
            if let Some(poi) = layer.poi {
                params["poi"] = json!(poi);
            }
            if let Some(zoom) = layer.zoom {
                params["zoom"] = json!(zoom);
            }
            if let Some(preset) = &layer.preset {
                params["preset"] = json!(preset);
            }
            exec(session, "layer.newCamera", params)?
        }
        "light" => {
            let mut params = json!({
                "name": name,
                "kind": layer.light_kind.as_deref().unwrap_or("Point"),
                "position": pos3,
            });
            if let Some(c) = &layer.color {
                params["color"] = json!(c);
            }
            if let Some(i) = layer.intensity {
                params["intensity"] = json!(i);
            }
            if let Some(poi) = layer.poi {
                params["poi"] = json!(poi);
            }
            exec(session, "layer.newLight", params)?
        }
        "null" => exec(session, "layer.newNull", json!({ "name": name }))?,
        other => return Err(format!("layers[{index}]: unknown type `{other}`")),
    };

    let id = created
        .get("layer")
        .cloned()
        .ok_or_else(|| format!("layers[{index}]: create returned no layer id"))?;

    // Solids / particles / nulls are created without a position; place them explicitly.
    if matches!(
        layer.layer_type.to_ascii_lowercase().as_str(),
        "solid" | "particle" | "particle2d" | "null"
    ) {
        exec(
            session,
            "prop.set",
            json!({
                "layer": id,
                "path": "transform/position",
                "value": pos3,
            }),
        )?;
    }

    if let Some(opacity) = layer.opacity {
        exec(
            session,
            "prop.set",
            json!({
                "layer": id,
                "path": "transform/opacity",
                "value": opacity,
            }),
        )?;
    }

    if let Some(scale) = &layer.scale {
        exec(
            session,
            "prop.set",
            json!({
                "layer": id,
                "path": "transform/scale",
                "value": scale_value(scale),
            }),
        )?;
    }

    if let Some(rotation) = layer.rotation {
        exec(
            session,
            "prop.set",
            json!({
                "layer": id,
                "path": "transform/rotation",
                "value": rotation,
            }),
        )?;
    }

    if layer.three_d
        && !matches!(
            layer.layer_type.to_ascii_lowercase().as_str(),
            "camera" | "light"
        )
    {
        exec(
            session,
            "layer.setSwitch",
            json!({
                "layers": [id],
                "switch": "threeD",
                "value": true,
            }),
        )?;
    }

    // Camera `move` is a shorthand for position keys (appended after `keys.position`).
    let mut position_keys = layer.keys.position.clone();
    if !layer.camera_move.is_empty() {
        position_keys.extend(layer.camera_move.clone());
    }
    apply_keys(
        session,
        &id,
        "transform/position",
        &position_keys,
        KeyKind::Position,
    )?;
    apply_keys(
        session,
        &id,
        "transform/scale",
        &layer.keys.scale,
        KeyKind::Scale,
    )?;
    apply_keys(
        session,
        &id,
        "transform/rotation",
        &layer.keys.rotation,
        KeyKind::Scalar,
    )?;
    apply_keys(
        session,
        &id,
        "transform/opacity",
        &layer.keys.opacity,
        KeyKind::Scalar,
    )?;

    let effects = layer.all_effects();
    for (effect_index, effect) in effects.iter().enumerate() {
        apply_effect(session, &id, layer, effect, effect_index)?;
    }

    // Layer-level effect keyframes target effects/#1/...
    for (path, raw) in &layer.keys.effects {
        let keys = parse_key_list(raw);
        apply_keys(
            session,
            &id,
            &format!("effects/#1/{path}"),
            &keys,
            KeyKind::Raw,
        )?;
    }

    Ok(())
}

fn apply_effect(
    session: &mut effectcraft_engine::Session,
    layer: &Value,
    layer_spec: &LayerSpec,
    effect: &EffectSpec,
    effect_index: usize,
) -> Result<(), String> {
    let effect_id = resolve_effect_id(&effect.id);
    exec(
        session,
        "effect.apply",
        json!({
            "layers": [layer],
            "effect": effect_id,
        }),
    )?;

    let mut params = effect_params(effect);
    if is_particle_effect(effect_id) || layer_spec.is_particle_host() {
        params = merge_params(particle_params(layer_spec), params);
    }

    let slot = effect_index + 1;
    for (key, value) in params {
        exec(
            session,
            "prop.set",
            json!({
                "layer": layer,
                "path": format!("effects/#{slot}/{key}"),
                "value": value,
            }),
        )?;
    }

    for (path, raw) in &effect.keys {
        let keys = parse_key_list(raw);
        apply_keys(
            session,
            layer,
            &format!("effects/#{slot}/{path}"),
            &keys,
            KeyKind::Raw,
        )?;
    }

    // Warp Stabilizer needs an analysis pass before it changes frames.
    if is_warp_stabilizer(effect_id) && effect.analyze.unwrap_or(true) {
        let status = exec(
            session,
            "warp.analyze",
            json!({
                "layer": layer,
                "effect": slot,
                "wait": true,
            }),
        )?;
        if status.get("analyzed") == Some(&json!(false)) {
            // Some builds only report analyzed via warp.status after wait.
            let check = exec(
                session,
                "warp.status",
                json!({ "layer": layer, "effect": slot }),
            )?;
            if check.get("analyzed") != Some(&json!(true)) {
                eprintln!("warning: Warp Stabilizer analysis may be incomplete: {check}");
            }
        }
    }

    Ok(())
}

#[derive(Clone, Copy)]
enum KeyKind {
    Position,
    Scale,
    Scalar,
    Raw,
}

fn apply_keys(
    session: &mut effectcraft_engine::Session,
    layer: &Value,
    path: &str,
    keys: &[KeyFrame],
    kind: KeyKind,
) -> Result<(), String> {
    for key in keys {
        let value = match kind {
            KeyKind::Position => position_value(&key.value),
            KeyKind::Scale => scale_value(&key.value),
            KeyKind::Scalar | KeyKind::Raw => key.value.clone(),
        };
        exec(
            session,
            "prop.addKey",
            json!({
                "layer": layer,
                "path": path,
                "time": key.time,
                "value": value,
            }),
        )?;
    }
    Ok(())
}

fn position_value(v: &Value) -> Value {
    match v {
        Value::Array(a) if a.len() >= 3 => json!([
            a[0].as_f64().unwrap_or(0.0),
            a[1].as_f64().unwrap_or(0.0),
            a[2].as_f64().unwrap_or(0.0)
        ]),
        Value::Array(a) if a.len() >= 2 => json!([
            a[0].as_f64().unwrap_or(0.0),
            a[1].as_f64().unwrap_or(0.0),
            0.0
        ]),
        _ => v.clone(),
    }
}

fn scale_value(v: &Value) -> Value {
    if let Some(n) = v.as_f64() {
        return json!([n, n, 100.0]);
    }
    match v {
        Value::Array(a) if a.len() == 1 => {
            let n = a[0].as_f64().unwrap_or(100.0);
            json!([n, n, 100.0])
        }
        Value::Array(a) if a.len() == 2 => json!([
            a[0].as_f64().unwrap_or(100.0),
            a[1].as_f64().unwrap_or(100.0),
            100.0
        ]),
        Value::Array(a) if a.len() >= 3 => json!([
            a[0].as_f64().unwrap_or(100.0),
            a[1].as_f64().unwrap_or(100.0),
            a[2].as_f64().unwrap_or(100.0)
        ]),
        _ => v.clone(),
    }
}

fn resolve_media_path(path: &str) -> Result<PathBuf, String> {
    let p = PathBuf::from(path);
    if p.is_absolute() {
        if p.is_file() {
            return Ok(p);
        }
        return Err(format!("footage path not found: {}", p.display()));
    }
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let candidate = cwd.join(&p);
    if candidate.is_file() {
        return Ok(candidate);
    }
    Err(format!(
        "footage path not found: {} (looked in {})",
        path,
        candidate.display()
    ))
}

fn exec(
    session: &mut effectcraft_engine::Session,
    command: &str,
    params: Value,
) -> Result<Value, String> {
    session
        .execute(command, params)
        .map_err(|e| format!("{command}: {e}"))
}


