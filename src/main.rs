use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod config;
mod patcher_laa;
mod patcher_preload;
mod pe;

use config::Config;
use patcher_laa::{detect_version, patch_exe, GameVersion};
use patcher_preload::gather_and_create_mod;

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Battle Brothers CLI Patcher
///
/// A command-line tool to apply the 4GB (LAA) patch and create mod preload files
/// for Battle Brothers. Works on Linux/WINE and Windows.
#[derive(Parser)]
#[command(name = "bb-patcher")]
#[command(version = VERSION)]
#[command(about = "Battle Brothers CLI Patcher - Apply 4GB patch and create mod preloads")]
struct Cli {
	#[command(subcommand)]
	command: Commands,
}

#[derive(Subcommand)]
enum Commands {
	/// Apply the 4GB (LAA) patch to BattleBrothers.exe
	///
	/// This patch allows the game to use up to 4GB of RAM instead of 2GB,
	/// which is necessary for running heavy mod lists without crashes.
	Patch4gb {
		/// Path to BattleBrothers.exe or the game directory
		#[arg(short, long)]
		path: Option<PathBuf>,

		/// Skip Steam DRM removal check (patch may not work correctly)
		#[arg(long)]
		skip_steam_drm: bool,
	},

	/// Create the mod preload file (~mod_msu_launcher.zip)
	///
	/// Scans all mods in the data folder and creates a preload manifest
	/// that registers mod resources with the game's mod system.
	Preload {
		/// Path to BattleBrothers.exe or the game directory
		#[arg(short, long)]
		path: Option<PathBuf>,
	},

	/// Run both 4GB patch and preload creation
	All {
		/// Path to BattleBrothers.exe or the game directory
		#[arg(short, long)]
		path: Option<PathBuf>,

		/// Skip Steam DRM removal check (patch may not work correctly)
		#[arg(long)]
		skip_steam_drm: bool,
	},

	/// Detect the game version without making changes
	Detect {
		/// Path to BattleBrothers.exe or the game directory
		#[arg(short, long)]
		path: Option<PathBuf>,
	},

	/// Check if the game is already patched with LAA
	Check {
		/// Path to BattleBrothers.exe or the game directory
		#[arg(short, long)]
		path: Option<PathBuf>,
	},

	/// Set the game path in the config file
	SetPath {
		/// Path to BattleBrothers.exe or the game directory
		path: PathBuf,
	},

	/// Show current configuration
	Config,
}

fn resolve_game_path(path: Option<PathBuf>) -> Result<Config> {
	let mut config = Config::load_or_default();

	if let Some(p) = path {
		// User provided a path - validate and use it
		if p.is_file()
			&& p.file_name()
				.map(|f| f == "BattleBrothers.exe")
				.unwrap_or(false)
		{
			config.set_path_from_exe(&p)?;
		} else if p.is_dir() {
			config.set_path(&p)?;
		} else {
			return Err(anyhow!(
				"Invalid path: {:?}\nExpected path to BattleBrothers.exe or the game directory",
				p
			));
		}
	}

	if config.bb_path.is_none() {
		return Err(anyhow!(
			"Game path not found. Please specify with --path or run 'bb-patcher set-path <PATH>'\n\
            Example: bb-patcher patch4gb --path /path/to/Battle\\ Brothers"
		));
	}

	Ok(config)
}

fn cmd_patch4gb(path: Option<PathBuf>, skip_steam_drm: bool) -> Result<()> {
	let config = resolve_game_path(path)?;

	let exe_path = config
		.get_bb_exe_path()
		.context("Could not find BattleBrothers.exe")?;

	println!("Applying 4GB (LAA) patch to: {:?}", exe_path.as_ref());

	let result = patch_exe(exe_path.as_ref(), skip_steam_drm)?;
	println!("  {}", result);

	Ok(())
}

fn cmd_preload(path: Option<PathBuf>) -> Result<()> {
	let config = resolve_game_path(path)?;

	let data_path = config
		.get_bb_data_path()
		.context("Could not find data folder")?;

	println!("Creating mod preload from: {:?}", data_path.as_ref());

	let resources = gather_and_create_mod(&data_path)?;
	println!(
		"  Created ~mod_msu_launcher.zip with {} on_start and {} on_running resources",
		resources.on_start_count(),
		resources.on_running_count()
	);

	Ok(())
}

fn cmd_all(path: Option<PathBuf>, skip_steam_drm: bool) -> Result<()> {
	let config = resolve_game_path(path)?;

	// 4GB Patch
	if let Some(exe_path) = config.get_bb_exe_path() {
		println!("Applying 4GB (LAA) patch to: {:?}", exe_path.as_ref());
		match patch_exe(exe_path.as_ref(), skip_steam_drm) {
			Ok(result) => println!("  {}", result),
			Err(e) => println!("  Warning: {}", e),
		}
	} else {
		println!("Warning: Could not find BattleBrothers.exe, skipping 4GB patch");
	}

	// Preload
	if let Some(data_path) = config.get_bb_data_path() {
		println!("\nCreating mod preload from: {:?}", data_path.as_ref());
		let resources = gather_and_create_mod(&data_path)?;
		println!(
			"  Created ~mod_msu_launcher.zip with {} on_start and {} on_running resources",
			resources.on_start_count(),
			resources.on_running_count()
		);
	} else {
		return Err(anyhow!("Could not find data folder"));
	}

	Ok(())
}

fn cmd_detect(path: Option<PathBuf>) -> Result<()> {
	let config = resolve_game_path(path)?;

	let exe_path = config
		.get_bb_exe_path()
		.context("Could not find BattleBrothers.exe")?;

	println!("Detecting version of: {:?}", exe_path.as_ref());

	let version = detect_version(exe_path.as_ref())?;
	match version {
		GameVersion::Steam => {
			println!("  Version: Steam (has DRM)");
			println!("  Note: You'll need to remove DRM before patching on Linux/WINE");
		}
		GameVersion::Steamless => {
			println!("  Version: Steam (DRM already removed)");
			println!("  Ready for 4GB patch!");
		}
		GameVersion::Gog => {
			println!("  Version: GOG (no DRM)");
			println!("  Ready for 4GB patch!");
		}
		GameVersion::AlreadyPatched => {
			println!("  Version: Already patched with 4GB/LAA");
			println!("  No action needed!");
		}
		GameVersion::Unknown => {
			println!("  Version: Unknown");
			println!("  This may be a new game version. Please report on GitHub.");
		}
	}

	Ok(())
}

fn cmd_check(path: Option<PathBuf>) -> Result<()> {
	let config = resolve_game_path(path)?;

	let exe_path = config
		.get_bb_exe_path()
		.context("Could not find BattleBrothers.exe")?;

	println!("Checking LAA status of: {:?}", exe_path.as_ref());

	let is_patched = patcher_laa::is_laa(exe_path.as_ref())?;
	if is_patched {
		println!("  Status: PATCHED (Large Address Aware flag is set)");
	} else {
		println!("  Status: NOT PATCHED (needs 4GB patch)");
	}

	Ok(())
}

fn cmd_set_path(path: PathBuf) -> Result<()> {
	let mut config = Config::load_or_default();

	if path.is_file()
		&& path
			.file_name()
			.map(|f| f == "BattleBrothers.exe")
			.unwrap_or(false)
	{
		let bb_path = config.set_path_from_exe(&path)?;
		println!("Game path set to: {:?}", bb_path);
	} else if path.is_dir() {
		config.set_path(&path)?;
		println!("Game path set to: {:?}", path);
	} else {
		return Err(anyhow!(
			"Invalid path: {:?}\nExpected path to BattleBrothers.exe or the game directory",
			path
		));
	}

	Ok(())
}

fn cmd_config() -> Result<()> {
	let config = Config::load_or_default();

	println!("Current configuration:");
	match &config.bb_path {
		Some(path) => {
			println!("  Game path: {:?}", path);

			if let Some(exe) = config.get_bb_exe_path() {
				println!("  Executable: {:?} (found)", exe.as_ref());
			} else {
				println!("  Executable: NOT FOUND");
			}

			if let Some(data) = config.get_bb_data_path() {
				println!("  Data folder: {:?} (found)", data.as_ref());
			} else {
				println!("  Data folder: NOT FOUND");
			}
		}
		None => {
			println!("  Game path: Not configured");
			println!("  Use 'bb-patcher set-path <PATH>' to configure");
		}
	}

	Ok(())
}

fn main() {
	let cli = Cli::parse();

	let result = match cli.command {
		Commands::Patch4gb {
			path,
			skip_steam_drm,
		} => cmd_patch4gb(path, skip_steam_drm),
		Commands::Preload { path } => cmd_preload(path),
		Commands::All {
			path,
			skip_steam_drm,
		} => cmd_all(path, skip_steam_drm),
		Commands::Detect { path } => cmd_detect(path),
		Commands::Check { path } => cmd_check(path),
		Commands::SetPath { path } => cmd_set_path(path),
		Commands::Config => cmd_config(),
	};

	if let Err(e) = result {
		eprintln!("Error: {:#}", e);
		std::process::exit(1);
	}
}
