//! The first-run install step.
//!
//! D2RUST ships no game data. On the first start the player points the game
//! at their own copy of the original game; the install step copies the
//! archives it needs into the player's D2RUST folder, converts them to
//! native assets, and bakes animation frames from those assets. Everything
//! happens locally. Later starts reuse the converted folder.

use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use d2_client::app::single_player;

/// Environment variable that names the player's own game folder.
pub const GAME_DIR_VAR: &str = "D2_GAME_DIR";

/// Environment variable that names a converted native asset folder.
pub const NATIVE_DIR_VAR: &str = "D2_NATIVE_DIR";

/// Environment variable that names a bake folder.
pub const ANIM_DIR_VAR: &str = "D2_ANIM_DIR";

/// Where this run's files live.
#[derive(Clone, Debug)]
pub struct Paths {
    /// The copied archives.
    pub game: PathBuf,
    /// The converted native assets.
    pub native: PathBuf,
    /// The baked animation frames.
    pub animations: PathBuf,
}

impl Paths {
    /// The default folders under the platform data directory.
    pub fn default_paths() -> Paths {
        let native = d2_client::assets::default_native_dir()
            .unwrap_or_else(|| d2_client::user_dir::root().join("native"));
        let game = native
            .parent()
            .map(|p| p.join("game"))
            .unwrap_or_else(|| d2_client::user_dir::root().join("game"));
        Paths {
            game,
            native,
            animations: d2_client::world_view::unit_rules::animation_baked::dir(),
        }
    }

    /// Whether the converted folder is ready.
    pub fn has_native(&self) -> bool {
        self.native.join("manifest.toml").is_file()
    }

    /// Makes the folders this run needs.
    pub fn create_dirs(&self) -> Result<()> {
        std::fs::create_dir_all(&self.game)
            .with_context(|| format!("creating {}", self.game.display()))?;
        std::fs::create_dir_all(&self.native)
            .with_context(|| format!("creating {}", self.native.display()))?;
        Ok(())
    }
}

/// The game folder a command line or the environment names, if any.
pub fn named_game_dir(arg: Option<&Path>) -> Option<PathBuf> {
    if let Some(p) = arg {
        return Some(p.to_path_buf());
    }
    std::env::var_os(GAME_DIR_VAR)
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
}

/// The folder of the running executable, then the current folder.
fn candidates() -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            out.push(dir.to_path_buf());
        }
    }
    if let Ok(cwd) = std::env::current_dir() {
        if !out.contains(&cwd) {
            out.push(cwd);
        }
    }
    out
}

/// Whether `dir` holds at least one archive of the original game.
pub fn looks_like_install(dir: &Path) -> bool {
    std::fs::read_dir(dir).is_ok_and(|entries| {
        entries.flatten().any(|e| {
            e.file_name()
                .to_string_lossy()
                .to_ascii_lowercase()
                .ends_with(".mpq")
        })
    })
}

/// Asks for the game folder: a typed path on a terminal, a native folder
/// picker when the game has no console (a release build on Windows).
fn ask_for_game_dir() -> Result<PathBuf> {
    use std::io::IsTerminal;
    if std::io::stdin().is_terminal() {
        println!("D2RUST needs the folder that holds your own copy of the original game.");
        println!("It will copy the files it needs into your D2RUST folder and convert them.");
        print!("Game folder: ");
        use std::io::Write;
        std::io::stdout().flush().ok();
        let mut line = String::new();
        std::io::stdin().read_line(&mut line)?;
        let path = PathBuf::from(line.trim());
        return Ok(path);
    }
    #[cfg(windows)]
    {
        if let Some(dir) = rfd::FileDialog::new()
            .set_title("Select the folder of your original game")
            .pick_folder()
        {
            return Ok(dir);
        }
    }
    bail!(
        "no game folder was chosen; pass --game-dir DIR or set {GAME_DIR_VAR}"
    )
}

/// Ensures a converted install exists.
///
/// `game_arg` is `--game-dir`. When a converted folder already exists it is
/// used as is. Otherwise the source folder is asked for (or taken from the
/// command line) and the install runs.
pub fn ensure(game_arg: Option<&Path>, force: bool) -> Result<Paths> {
    let paths = Paths::default_paths();
    if paths.has_native() && !force {
        return Ok(paths);
    }
    let source = named_game_dir(game_arg)
        .or_else(|| candidates().into_iter().find(|d| looks_like_install(d)))
        .map(Ok)
        .unwrap_or_else(ask_for_game_dir)?;
    install(&source, &paths)?;
    Ok(paths)
}

/// Runs the install from `source` into `paths`.
pub fn install(source: &Path, paths: &Paths) -> Result<()> {
    if !source.is_dir() {
        bail!("{} is not a folder", source.display());
    }
    if !looks_like_install(source) {
        bail!(
            "{} holds no game archives; choose the folder of your own copy of the original game",
            source.display()
        );
    }
    paths.create_dirs()?;

    println!("install: copying game files from {}", source.display());
    let copied = copy_archives(source, &paths.game)?;
    if copied == 0 {
        bail!("no archives were copied from {}", source.display());
    }
    println!("install: copied {copied} archive(s) to {}", paths.game.display());

    println!("install: converting assets (this can take a while)");
    let options = d2_install::Options::new(&paths.game, &paths.native);
    let kinds = d2_install::kinds::builtin();
    let summary = d2_install::convert(&options, &kinds)
        .map_err(|e| anyhow::anyhow!("converting assets: {e}"))?;
    for (path, check, first) in &summary.failures {
        eprintln!("install: {path}: {check}: {first}");
    }
    if !summary.failures.is_empty() {
        bail!(
            "conversion left {} file(s) unchecked; the install is not ready",
            summary.failures.len()
        );
    }
    if !paths.has_native() {
        bail!("conversion finished without a manifest");
    }
    println!(
        "install: converted {} file(s) to {}",
        summary.rows.len(),
        paths.native.display()
    );

    println!("install: baking animation frames (this can take a while)");
    let jobs = std::thread::available_parallelism()
        .map_or(1, |n| n.get().saturating_sub(1))
        .max(1);
    let bake = d2_client::world_view::unit_rules::animation_bake::BakeOptions {
        data: d2_client::world_view::unit_rules::animation_bake::BakeData::Native {
            dir: paths.native.clone(),
            mods: Vec::new(),
        },
        out: paths.animations.clone(),
        jobs,
        plan: false,
        limit: None,
        reuse: Vec::new(),
    };
    let baked = d2_client::world_view::unit_rules::animation_bake::bake(&bake)
        .map_err(|e| anyhow::anyhow!("baking animations: {e}"))?;
    println!(
        "install: baked {} frame(s) into {}",
        baked.frames,
        paths.animations.display()
    );
    println!("install: ready; starting the game");
    Ok(())
}

/// Copies every archive in `source` into `dest`; returns how many were copied.
fn copy_archives(source: &Path, dest: &Path) -> Result<usize> {
    let mut copied = 0;
    for entry in std::fs::read_dir(source)
        .with_context(|| format!("reading {}", source.display()))?
        .flatten()
    {
        let path = entry.path();
        if !path.is_file()
            || !path
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("mpq"))
        {
            continue;
        }
        let name = entry.file_name();
        let to = dest.join(&name);
        let same = std::fs::metadata(&path).ok().map(|m| m.len())
            == std::fs::metadata(&to).ok().map(|m| m.len());
        if !same {
            std::fs::copy(&path, &to)
                .with_context(|| format!("copying {} to {}", path.display(), to.display()))?;
            copied += 1;
        }
    }
    Ok(copied)
}

/// Sets the environment variables the rest of the game reads, after an
/// install.
pub fn apply(paths: &Paths) {
    std::env::set_var(GAME_DIR_VAR, &paths.game);
    std::env::set_var(NATIVE_DIR_VAR, &paths.native);
    std::env::set_var(ANIM_DIR_VAR, &paths.animations);
    let _ = single_player::GameData::select(Some(&paths.game));
}
