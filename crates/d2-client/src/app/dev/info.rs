//! The info overlay (`info`), the log ring the bug report reads, and the
//! bug report (`bugreport [note]`). Client-side only: they read the
//! client model and the window, and write files under the save folder.
//! The bug report is a `d2_report` bundle, the same format the launcher
//! shows the player and the report service accepts.

use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

use bevy::log::tracing_subscriber::layer::{Context, Layer};
use bevy::log::tracing_subscriber::registry::Registry;
use bevy::log::BoxedLayer;
use bevy::prelude::*;

use super::server::DevCatalog;
use crate::bridge::BridgeResource;
use crate::world_view::present::WorldViewState;

/// Log lines kept for a bug report.
pub const LOG_LINES: usize = 200;

fn ring() -> &'static Mutex<VecDeque<String>> {
    static RING: OnceLock<Mutex<VecDeque<String>>> = OnceLock::new();
    RING.get_or_init(|| Mutex::new(VecDeque::with_capacity(LOG_LINES)))
}

/// Adds one line to the ring (the oldest goes past [`LOG_LINES`]).
pub fn push_log(line: String) {
    if let Ok(mut r) = ring().lock() {
        if r.len() == LOG_LINES {
            r.pop_front();
        }
        r.push_back(line);
    }
}

/// The ring's lines, oldest first.
pub fn last_log() -> Vec<String> {
    ring()
        .lock()
        .map(|r| r.iter().cloned().collect())
        .unwrap_or_default()
}

struct Fields(String);

impl bevy::log::tracing::field::Visit for Fields {
    fn record_debug(
        &mut self,
        field: &bevy::log::tracing::field::Field,
        value: &dyn std::fmt::Debug,
    ) {
        use std::fmt::Write;
        if field.name() == "message" {
            let _ = write!(self.0, "{value:?}");
        } else {
            let _ = write!(self.0, " {}={value:?}", field.name());
        }
    }
}

/// The tracing layer that fills the ring.
struct RingLayer;

impl Layer<Registry> for RingLayer {
    fn on_event(&self, event: &bevy::log::tracing::Event<'_>, _: Context<'_, Registry>) {
        let m = event.metadata();
        let mut f = Fields(String::new());
        event.record(&mut f);
        push_log(format!("{} {}: {}", m.level(), m.target(), f.0));
    }
}

/// `LogPlugin::custom_layer` of the interactive game with the tools on.
pub fn log_layer(_: &mut App) -> Option<BoxedLayer> {
    Some(Box::new(RingLayer))
}

/// The build's commit (set by the CI build), else "local build".
pub fn build_commit() -> &'static str {
    option_env!("GITHUB_SHA").unwrap_or("local build")
}

/// What the tools know of the game they run in (set by `play::run`).
#[derive(Resource, Clone, Debug)]
pub struct DevContext {
    pub seed: u32,
    pub difficulty: u8,
    /// The character's save file (none: a save cannot be made).
    pub save_path: Option<PathBuf>,
    /// The folder the tools write into (slots, bug reports).
    pub dir: PathBuf,
    pub catalog: Arc<DevCatalog>,
}

/// The info overlay's switch.
#[derive(Resource, Default)]
pub struct InfoOverlay {
    pub on: bool,
}

#[derive(Component)]
pub(super) struct InfoText;

/// A level id with its name.
pub fn level_name(cat: &DevCatalog, level: u32) -> String {
    cat.levels.iter().find(|(id, _)| *id == level).map_or_else(
        || format!("level {level}"),
        |(_, n)| format!("{n} ({level})"),
    )
}

/// The state lines the info overlay and the bug report show.
pub fn state_lines(
    b: &BridgeResource,
    view: Option<&WorldViewState>,
    ctx: &DevContext,
    fps: Option<f64>,
) -> Vec<String> {
    let w = b.0.world();
    let mut out = Vec::new();
    match (w.local(), w.local_room()) {
        (Some(u), Some(r)) => {
            let (x, y) = u.cell();
            out.push(format!(
                "area {}",
                level_name(&ctx.catalog, u32::from(r.level))
            ));
            out.push(format!("position ({x}, {y})  mode {}", u.mode));
            if let (Some(l), Some(m)) = (u.stats.get(&6), u.stats.get(&7)) {
                out.push(format!("life {}/{}", l >> 8, m >> 8));
            }
        }
        _ => out.push("not in game".into()),
    }
    out.push(format!(
        "server tick {}  frame {}  seed {}  difficulty {}",
        w.server_ticks, w.frames, ctx.seed, ctx.difficulty
    ));
    if let Some(f) = fps {
        out.push(format!("fps {f:.0}"));
    }
    if let Some(k) = view.and_then(|v| v.prev_hover) {
        let line = match w.units.get(&k) {
            Some(u) => {
                let name = if k.unit_type == 1 {
                    ctx.catalog
                        .monsters
                        .iter()
                        .find(|(id, _)| *id == u.class)
                        .map_or("", |(_, n)| n.as_str())
                } else {
                    ""
                };
                let life = u
                    .stats
                    .get(&6)
                    .map_or(String::new(), |l| format!(" life {}", l >> 8));
                format!(
                    "hover: type {} guid {} class {} {name} mode {}{life}",
                    k.unit_type, k.guid, u.class, u.mode
                )
            }
            None => format!("hover: type {} guid {}", k.unit_type, k.guid),
        };
        out.push(line);
    }
    out
}

pub(super) fn spawn_info(mut commands: Commands) {
    commands.spawn((
        InfoText,
        crate::app::session::GameEntity,
        Text::new(""),
        TextFont {
            font_size: bevy::text::FontSize::Px(12.0),
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 0.6)),
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(2.0),
            right: Val::Px(2.0),
            padding: UiRect::all(Val::Px(3.0)),
            ..default()
        },
        GlobalZIndex(1990),
        Visibility::Hidden,
    ));
}

pub(super) fn draw_info(
    info: Res<InfoOverlay>,
    ctx: Option<Res<DevContext>>,
    bridge: Option<Res<BridgeResource>>,
    view: Option<Res<WorldViewState>>,
    time: Res<Time>,
    mut fps: Local<f64>,
    mut text: Query<(&mut Text, &mut Visibility), With<InfoText>>,
) {
    let dt = time.delta_secs_f64();
    if dt > 0.0 {
        *fps = if *fps == 0.0 {
            1.0 / dt
        } else {
            *fps * 0.95 + 0.05 / dt
        };
    }
    for (mut t, mut v) in &mut text {
        let want = if info.on {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *v != want {
            *v = want;
        }
        let (Some(ctx), Some(b), true) = (ctx.as_deref(), bridge.as_deref(), info.on) else {
            continue;
        };
        let s = state_lines(b, view.as_deref(), ctx, Some(*fps)).join("\n");
        if t.0 != s {
            t.0 = s;
        }
    }
}

/// A folder name that is free under `dir`: `<secs>`, else `<secs>-<n>`.
fn fresh_dir(dir: &Path, secs: u64) -> PathBuf {
    let mut p = dir.join(secs.to_string());
    let mut n = 1;
    while p.exists() {
        p = dir.join(format!("{secs}-{n}"));
        n += 1;
    }
    p
}
