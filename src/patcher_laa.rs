use crate::pe::{
	ImageDosHeader, ImageFileHeader, IMAGE_DOS_SIGNATURE, IMAGE_FILE_LARGE_ADDRESS_AWARE,
	IMAGE_NT_SIGNATURE,
};
use anyhow::{anyhow, Context, Result};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::io::{Read, Seek, SeekFrom, Write};
use std::mem::size_of;
use std::{fs::File, path::Path};

const GOG_HASH_STR: &str = include_str!("../hashes/gog.txt");
const STEAM_HASH_STR: &str = include_str!("../hashes/steam.txt");
const STEAMLESS_HASH_STR: &str = include_str!("../hashes/steamless.txt");

fn get_hash_set_from_str(hash_str: &str) -> HashSet<Vec<u8>> {
	hash_str
		.lines()
		.filter(|line| !line.is_empty())
		.map(|line| const_hex::decode(line).unwrap())
		.collect()
}

fn read_and_check_pe_magic_number(file: &mut File, seek_back: bool) -> Result<()> {
	let mut pe_magic_number: [u8; 4] = [0; 4];
	file.read_exact(&mut pe_magic_number)?;

	let signature = u32::from_le_bytes(pe_magic_number);
	if signature != IMAGE_NT_SIGNATURE {
		return Err(anyhow!("Invalid PE magic number"));
	}

	if seek_back {
		file.seek(SeekFrom::Current(-(size_of::<[u8; 4]>() as i64)))?;
	}

	Ok(())
}

fn seek_to_pe_header(file: &mut File) -> Result<()> {
	file.seek(SeekFrom::Start(0))?;
	let mut dos_header_bytes = [0u8; size_of::<ImageDosHeader>()];
	file.read_exact(&mut dos_header_bytes)?;

	// Safety: ImageDosHeader is repr(C, packed) and contains only primitive types
	let dos_header: ImageDosHeader =
		unsafe { std::ptr::read_unaligned(dos_header_bytes.as_ptr() as *const ImageDosHeader) };

	let e_magic = dos_header.e_magic;
	if e_magic != IMAGE_DOS_SIGNATURE {
		return Err(anyhow!("Invalid DOS magic number : {:X}", e_magic));
	}

	file.seek(SeekFrom::Start(dos_header.e_lfanew as u64))?;

	read_and_check_pe_magic_number(file, true)
}

fn read_image_file_header(file: &mut File) -> Result<ImageFileHeader> {
	read_and_check_pe_magic_number(file, false)?;
	let mut header_bytes = [0u8; size_of::<ImageFileHeader>()];
	file.read_exact(&mut header_bytes)?;

	// Safety: ImageFileHeader is repr(C, packed) and contains only primitive types
	let file_header: ImageFileHeader =
		unsafe { std::ptr::read_unaligned(header_bytes.as_ptr() as *const ImageFileHeader) };
	Ok(file_header)
}

fn write_image_file_header(file: &mut File, header: &ImageFileHeader) -> Result<()> {
	if file.metadata()?.permissions().readonly() {
		return Err(anyhow!(
			"Couldn't write IMAGE_FILE_HEADER: File is readonly"
		));
	}
	read_and_check_pe_magic_number(file, false)?;

	// Safety: ImageFileHeader is repr(C, packed) and contains only primitive types
	let header_bytes: &[u8] = unsafe {
		std::slice::from_raw_parts(
			header as *const ImageFileHeader as *const u8,
			size_of::<ImageFileHeader>(),
		)
	};
	file.write_all(header_bytes)
		.context("Couldn't write IMAGE_FILE_HEADER")?;
	Ok(())
}

fn make_laa(path: &Path) -> Result<()> {
	let mut file = File::options().read(true).write(true).open(path)?;
	seek_to_pe_header(&mut file)?;
	let mut file_header = read_image_file_header(&mut file)?;
	file_header.characteristics |= IMAGE_FILE_LARGE_ADDRESS_AWARE;
	seek_to_pe_header(&mut file)?;
	write_image_file_header(&mut file, &file_header)?;
	Ok(())
}

pub fn is_laa(path: &Path) -> Result<bool> {
	let mut file = File::open(path)?;
	seek_to_pe_header(&mut file)?;
	let file_header = read_image_file_header(&mut file)?;
	Ok(file_header.characteristics & IMAGE_FILE_LARGE_ADDRESS_AWARE != 0)
}

fn sha_hash_path(path: &Path) -> Result<Vec<u8>> {
	let mut file = File::open(path)?;
	let mut hasher = Sha256::new();
	std::io::copy(&mut file, &mut hasher)?;
	Ok(hasher.finalize().to_vec())
}

fn make_backup(path: &Path, backup_extension: &str) -> Result<()> {
	let backup_path = format!(
		"{}.{}",
		path.to_str()
			.with_context(|| format!("Couldn't parse file path {:?}", path))?,
		backup_extension
	);
	std::fs::copy(path, backup_path).with_context(move || {
		format!(
			"Failed to create backup of file {:?} with extension {}",
			path, backup_extension
		)
	})?;
	Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameVersion {
	Steam,
	Steamless,
	Gog,
	AlreadyPatched,
	Unknown,
}

impl std::fmt::Display for GameVersion {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			GameVersion::Steam => write!(f, "Steam"),
			GameVersion::Steamless => write!(f, "Steamless"),
			GameVersion::Gog => write!(f, "GOG"),
			GameVersion::AlreadyPatched => write!(f, "Already Patched"),
			GameVersion::Unknown => write!(f, "Unknown"),
		}
	}
}

pub fn detect_version(exe_path: &Path) -> Result<GameVersion> {
	let hash = sha_hash_path(exe_path)?;
	if get_hash_set_from_str(STEAM_HASH_STR).contains(&hash) {
		Ok(GameVersion::Steam)
	} else if get_hash_set_from_str(STEAMLESS_HASH_STR).contains(&hash) {
		Ok(GameVersion::Steamless)
	} else if get_hash_set_from_str(GOG_HASH_STR).contains(&hash) {
		Ok(GameVersion::Gog)
	} else if is_laa(exe_path)? {
		Ok(GameVersion::AlreadyPatched)
	} else {
		Ok(GameVersion::Unknown)
	}
}

pub fn patch_exe(exe_path: &Path, skip_steam_drm: bool) -> Result<String> {
	let version = detect_version(exe_path)?;
	match version {
		GameVersion::Steam => {
			if skip_steam_drm {
				println!("  Steam version detected, but skipping DRM removal as requested");
				println!("  Note: The 4GB patch may not work correctly without DRM removal");
				make_backup(exe_path, "steam_backup")?;
				make_laa(exe_path).context("Failed to apply 4GB Patch")?;
				Ok("Patched Steam Version (DRM intact - may not work correctly)".to_string())
			} else {
				Err(anyhow!(
					"Steam version detected. Steam DRM removal requires running Steamless.CLI.exe on Windows.\n\
					Options:\n\
					1. Run Steamless manually on Windows first, then use this tool\n\
					2. Use --skip-steam-drm to patch anyway (may not work correctly)\n\
					3. Use the GOG version which doesn't have DRM"
				))
			}
		}
		GameVersion::Steamless => {
			make_backup(exe_path, "steamless_backup")?;
			make_laa(exe_path).context("Failed to apply 4GB Patch")?;
			Ok("Patched Steamless Version".to_string())
		}
		GameVersion::Gog => {
			make_backup(exe_path, "gog_backup")?;
			make_laa(exe_path).context("Failed to apply 4GB Patch")?;
			Ok("Patched GOG Version".to_string())
		}
		GameVersion::AlreadyPatched => Ok("Already patched".to_string()),
		GameVersion::Unknown => Err(anyhow!(
			"Unknown version of Battle Brothers.\n\
            Hash: {}\n\
            If this is a new version, please report it on GitHub.",
			const_hex::encode(sha_hash_path(exe_path)?)
		)),
	}
}
