//! Headless motion-graphics compositor for AI agents, driven by EffectCraft.
//!
//! Accepts a prompt or a JSON scene (CLI / file), builds the composition, and
//! writes H.264 MP4 via the render queue.

mod build;
mod effects;
mod presets;
mod scene;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use scene::Scene;

fn main() -> ExitCode {
    let cli = match parse_args(std::env::args()) {
        Ok(cli) => cli,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::FAILURE;
        }
    };
    if cli.help {
        println!("{}", usage());
        return ExitCode::SUCCESS;
    }

    match run(&cli) {
        Ok(path) => {
            eprintln!("wrote {}", path.display());
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

#[derive(Debug, Default)]
struct Cli {
    help: bool,
    prompt: Option<String>,
    json: Option<String>,
    input: Option<PathBuf>,
    output: Option<PathBuf>,
    positionals: Vec<String>,
}

fn parse_args(args: impl IntoIterator<Item = String>) -> Result<Cli, String> {
    let mut cli = Cli::default();
    let mut iter = args.into_iter().skip(1).peekable();

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "-h" | "--help" => cli.help = true,
            "-p" | "--prompt" => {
                cli.prompt = Some(require_value("--prompt", &mut iter)?);
            }
            "-j" | "--json" => {
                cli.json = Some(require_value("--json", &mut iter)?);
            }
            "-i" | "--input" => {
                cli.input = Some(PathBuf::from(require_value("--input", &mut iter)?));
            }
            "-o" | "--output" => {
                cli.output = Some(PathBuf::from(require_value("--output", &mut iter)?));
            }
            s if s.starts_with('-') => {
                return Err(format!("unknown option: {s}\n{}", usage()));
            }
            other => cli.positionals.push(other.to_string()),
        }
    }
    Ok(cli)
}

fn require_value(
    flag: &str,
    iter: &mut impl Iterator<Item = String>,
) -> Result<String, String> {
    iter.next()
        .ok_or_else(|| format!("missing value for {flag}\n{}", usage()))
}

fn usage() -> String {
    r##"agent-compositor — headless EffectCraft compositor for AI agents

USAGE:
  agent-compositor [OPTIONS] [INPUT.json] [OUTPUT.mp4]
  agent-compositor --prompt "Hello | Subtitle" -o out.mp4
  agent-compositor --input examples/stabilizer-particles.json -o stabilizer-particles.mp4
  agent-compositor --input examples/next-test.json -o next-test.mp4

OPTIONS:
  -p, --prompt TEXT   Title (optional "Title | Subtitle"); builds a title card
  -j, --json JSON     Inline scene JSON
  -i, --input PATH    Read scene JSON from a file
  -o, --output PATH   Output MP4 path (default: test.mp4, or scene.output)
  -h, --help          Show this help

LAYER TYPES: text | solid | shape | footage | particle | particle2d | camera | light | null

PARTICLE 3D:  { "type": "particle", ... }
PARTICLE 2D:  { "type": "particle2d", "direction": -90, "producer": [0.5, 0.8], ... }
              or { "type": "particle", "emitter": "2d", ... }

WARP STABILIZER (on footage):
  "effect": { "id": "stabilizer", "smoothness": 80, "strength": 70, "analyze": true }

EFFECTS: glow, blur, chroma, keylight, particles3d, particles2d, stabilizer

See README.md for the full scene schema."##
        .to_string()
}

fn resolve_scene(cli: &Cli) -> Result<Scene, String> {
    let sources = [
        cli.prompt.is_some(),
        cli.json.is_some(),
        cli.input.is_some(),
    ]
    .into_iter()
    .filter(|b| *b)
    .count();
    if sources > 1 {
        return Err("use only one of --prompt, --json, or --input".into());
    }

    if let Some(prompt) = &cli.prompt {
        return Ok(Scene::from_prompt(prompt));
    }
    if let Some(raw) = &cli.json {
        return Scene::from_json_str(raw);
    }
    if let Some(path) = &cli.input {
        let raw = std::fs::read_to_string(path)
            .map_err(|e| format!("read {}: {e}", path.display()))?;
        return Scene::from_json_str(&raw);
    }

    for p in &cli.positionals {
        let path = Path::new(p);
        if is_json_path(path) {
            let raw = std::fs::read_to_string(path)
                .map_err(|e| format!("read {}: {e}", path.display()))?;
            return Scene::from_json_str(&raw);
        }
    }

    Ok(Scene::default())
}

fn is_json_path(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("json"))
}

fn resolve_output(cli: &Cli, scene: &Scene) -> Result<PathBuf, String> {
    let from_positional = cli
        .positionals
        .iter()
        .map(Path::new)
        .find(|p| !is_json_path(p))
        .map(Path::to_path_buf);

    let chosen = cli
        .output
        .clone()
        .or(from_positional)
        .or_else(|| scene.output.as_ref().map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from("test.mp4"));

    if chosen.is_absolute() {
        Ok(chosen)
    } else {
        Ok(std::env::current_dir()
            .map_err(|e| e.to_string())?
            .join(chosen))
    }
}

fn run(cli: &Cli) -> Result<PathBuf, String> {
    let scene = resolve_scene(cli)?;
    scene.validate()?;
    let out = resolve_output(cli, &scene)?;
    build::render_scene(&scene, &out)
}
