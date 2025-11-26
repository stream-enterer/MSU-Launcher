//! Cross-platform PE (Portable Executable) header structures.
//! These are defined manually to avoid Windows-only dependencies.

/// DOS Header - 64 bytes at the start of every PE file
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, Default)]
pub struct ImageDosHeader {
	pub e_magic: u16,      // Magic number (MZ = 0x5A4D)
	pub e_cblp: u16,       // Bytes on last page of file
	pub e_cp: u16,         // Pages in file
	pub e_crlc: u16,       // Relocations
	pub e_cparhdr: u16,    // Size of header in paragraphs
	pub e_minalloc: u16,   // Minimum extra paragraphs needed
	pub e_maxalloc: u16,   // Maximum extra paragraphs needed
	pub e_ss: u16,         // Initial SS value
	pub e_sp: u16,         // Initial SP value
	pub e_csum: u16,       // Checksum
	pub e_ip: u16,         // Initial IP value
	pub e_cs: u16,         // Initial CS value
	pub e_lfarlc: u16,     // File address of relocation table
	pub e_ovno: u16,       // Overlay number
	pub e_res: [u16; 4],   // Reserved words
	pub e_oemid: u16,      // OEM identifier
	pub e_oeminfo: u16,    // OEM information
	pub e_res2: [u16; 10], // Reserved words
	pub e_lfanew: i32,     // File address of PE header
}

/// File Header - part of the PE header
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, Default)]
pub struct ImageFileHeader {
	pub machine: u16,                 // Target machine type
	pub number_of_sections: u16,      // Number of sections
	pub time_date_stamp: u32,         // Time/date stamp
	pub pointer_to_symbol_table: u32, // File offset to COFF symbol table
	pub number_of_symbols: u32,       // Number of symbols
	pub size_of_optional_header: u16, // Size of optional header
	pub characteristics: u16,         // File characteristics flags
}

// Constants
pub const IMAGE_DOS_SIGNATURE: u16 = 0x5A4D; // MZ
pub const IMAGE_NT_SIGNATURE: u32 = 0x00004550; // PE\0\0
pub const IMAGE_FILE_LARGE_ADDRESS_AWARE: u16 = 0x0020;
