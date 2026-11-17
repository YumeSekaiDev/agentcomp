//! Built-in layer stacks when a scene omits `layers`.

use serde_json::json;

use crate::scene::{
    EffectSpec, KeyFrame, LayerKeys, LayerSpec, Scene, fade_opacity_keys,
};

pub fn simple_title_layers(scene: &Scene) -> Vec<LayerSpec> {
    let w = scene.width as f64;
    let h = scene.height as f64;
    let mut layers = vec![LayerSpec {
        layer_type: "solid".into(),
        name: Some("Backdrop".into()),
        color: Some(scene.background.clone()),
        width: Some(scene.width),
        height: Some(scene.height),
        position: Some([w / 2.0, h / 2.0, 0.0]),
        ..Default::default()
    }];

    let mut title_keys = LayerKeys::default();
    if scene.fade {
        title_keys.opacity = fade_opacity_keys(scene.duration, scene.fade_in, scene.fade_out);
    }
    layers.push(LayerSpec {
        layer_type: "text".into(),
        name: Some("Title".into()),
        text: Some(scene.title.clone()),
        size: Some(json!(scene.font_size)),
        fill: Some(scene.fill.clone()),
        position: Some([w / 2.0, h / 2.0, 0.0]),
        keys: title_keys,
        ..Default::default()
    });

    if let Some(sub) = &scene.subtitle {
        let mut sub_keys = LayerKeys::default();
        if scene.fade {
            sub_keys.opacity = fade_opacity_keys(
                scene.duration,
                (scene.fade_in + 0.2).min(scene.duration / 2.0),
                scene.fade_out,
            );
        }
        layers.push(LayerSpec {
            layer_type: "text".into(),
            name: Some("Subtitle".into()),
            text: Some(sub.clone()),
            size: Some(json!((scene.font_size * 0.4).max(18.0))),
            fill: Some("#c8c8d8".into()),
            position: Some([w / 2.0, h / 2.0 + scene.font_size * 0.85, 0.0]),
            keys: sub_keys,
            ..Default::default()
        });
    }
    layers
}

pub fn title_card_layers(scene: &Scene) -> Vec<LayerSpec> {
    let w = scene.width as f64;
    let h = scene.height as f64;
    let d = scene.duration;
    let subtitle = scene
        .subtitle
        .clone()
        .unwrap_or_else(|| "Text · shape · keyframes · glow".into());

    vec![
        LayerSpec {
            layer_type: "solid".into(),
            name: Some("Plate".into()),
            color: Some("#140a22".into()),
            width: Some(scene.width),
            height: Some(scene.height),
            position: Some([w / 2.0, h / 2.0, 0.0]),
            keys: LayerKeys {
                opacity: vec![
                    KeyFrame {
                        time: 0.0,
                        value: json!(100.0),
                    },
                    KeyFrame {
                        time: d,
                        value: json!(85.0),
                    },
                ],
                ..Default::default()
            },
            ..Default::default()
        },
        LayerSpec {
            layer_type: "shape".into(),
            name: Some("Accent".into()),
            size: Some(json!([w * 0.42, 14.0])),
            fill: Some("#c084fc".into()),
            kind: Some("ellipse".into()),
            shape_size: Some([w * 0.42, 14.0]),
            position: Some([w / 2.0, h * 0.58, 0.0]),
            keys: LayerKeys {
                scale: vec![
                    KeyFrame {
                        time: 0.0,
                        value: json!(20.0),
                    },
                    KeyFrame {
                        time: 1.0,
                        value: json!(110.0),
                    },
                    KeyFrame {
                        time: 1.3,
                        value: json!(100.0),
                    },
                ],
                opacity: fade_opacity_keys(d, 0.5, 0.8),
                ..Default::default()
            },
            ..Default::default()
        },
        LayerSpec {
            layer_type: "text".into(),
            name: Some("Title".into()),
            text: Some(scene.title.clone()),
            size: Some(json!(scene.font_size)),
            fill: Some(scene.fill.clone()),
            position: Some([w / 2.0, h * 0.42, 0.0]),
            keys: LayerKeys {
                scale: vec![
                    KeyFrame {
                        time: 0.0,
                        value: json!(70.0),
                    },
                    KeyFrame {
                        time: 1.2,
                        value: json!(100.0),
                    },
                    KeyFrame {
                        time: d,
                        value: json!(106.0),
                    },
                ],
                opacity: fade_opacity_keys(d, 0.6, 0.7),
                ..Default::default()
            },
            effect: Some(EffectSpec {
                id: "glow".into(),
                radius: Some(28.0),
                intensity: Some(1.15),
                threshold: Some(35.0),
                ..Default::default()
            }),
            ..Default::default()
        },
        LayerSpec {
            layer_type: "text".into(),
            name: Some("Subtitle".into()),
            text: Some(subtitle),
            size: Some(json!((scene.font_size * 0.38).max(22.0))),
            fill: Some("#e9d8ff".into()),
            position: Some([w / 2.0, h * 0.58 + 36.0, 0.0]),
            keys: LayerKeys {
                position: vec![
                    KeyFrame {
                        time: 0.6,
                        value: json!([w / 2.0, h * 0.58 + 56.0]),
                    },
                    KeyFrame {
                        time: 1.4,
                        value: json!([w / 2.0, h * 0.58 + 36.0]),
                    },
                ],
                opacity: fade_opacity_keys(d, 0.9, 0.8),
                ..Default::default()
            },
            ..Default::default()
        },
    ]
}
