use alloc::{vec::Vec, string::String};
use crate::vfs::{File, Filesystem, Directory, DirEntry};
use crate::vga_println;
use core::slice;
use core::str;

pub const SECTOR_SIZE: usize = 512;

pub struct Fat12Volume {
    image: &'static [u8],
}

impl Fat12Volume {
    /// Cria o volume a partir de uma fat image embutida
    pub fn new() -> Self {
        let data = env!("FAT12_BYTES");
        let decoded = base64::decode(data).expect("base64 inválido");
        let image = Box::leak(decoded.into_boxed_slice());
        Self { image }
    }

    /// Lê um setor lógico (512 bytes)
    pub fn read_sector(&self, lba: usize) -> &[u8] {
        let start = lba * SECTOR_SIZE;
        let end = start + SECTOR_SIZE;
        &self.image[start..end]
    }

    /// Lê e imprime entradas do Root Directory (setores 19 a 32)
    pub fn list_root_dir(&self) {
        for sector in 19..=32 {
            let data = self.read_sector(sector);

            for entry in data.chunks(32) {
                if entry[0] == 0x00 {
                    return;
                }
                if entry[0] == 0xE5 {
                    continue;
                }

                let name = str::from_utf8(&entry[0..8]).unwrap_or("???").trim();
                let ext  = str::from_utf8(&entry[8..11]).unwrap_or("???").trim();

                let size = u32::from_le_bytes([
                    entry[28], entry[29], entry[30], entry[31]
                ]);

                vga_println!("Arquivo: {}.{} ({} bytes)", name, ext, size);
            }
        }
    }
}

/// FAT12 usa 12 bits por entrada — lógica de decodificação
fn read_fat_entry(&self, cluster: u16) -> u16 {
    let fat_offset = (cluster as usize * 3) / 2;
    let fat = self.read_fat(); // setores 1–9 (9 * 512 = 4608 bytes)

    let (first, second) = (fat[fat_offset], fat[fat_offset + 1]);

    if cluster & 1 == 0 {
        ((second as u16 & 0x0F) << 8) | first as u16
    } else {
        ((second as u16) << 4) | ((first as u16 & 0xF0) >> 4)
    }
}

/// Retorna slice com os 9 setores da FAT
fn read_fat(&self) -> &[u8] {
    let start = SECTOR_SIZE * 1;
    let end = SECTOR_SIZE * 10;
    &self.image[start..end]
}

fn read_directory_from_cluster(&self, start_cluster: u16) -> Vec<DirEntry> {
    let mut entries = Vec::new();
    let mut cluster = start_cluster;

    while cluster < 0xFF8 {
        let lba = 33 + (cluster - 2) as usize;
        let data = self.read_sector(lba);

        for entry in data.chunks(32) {
            if entry[0] == 0x00 || entry[0] == 0xE5 {
                continue;
            }

            let raw_name = &entry[0..11];
            let name = core::str::from_utf8(raw_name).unwrap_or("").trim().replace(" ", "");

            let attr = entry[11];
            let is_dir = attr & 0x10 != 0;

            let cluster = u16::from_le_bytes([entry[26], entry[27]]);
            let size = u32::from_le_bytes([entry[28], entry[29], entry[30], entry[31]]);

            entries.push(DirEntry {
                name,
                is_dir,
                cluster,
                size,
            });
        }

        cluster = self.read_fat_entry(cluster);
    }

    entries
}

pub fn find_file(&self, name: &str) -> Option<(u16, u32)> {
    for sector in 19..=32 {
        let data = self.read_sector(sector);

        for entry in data.chunks(32) {
            if entry[0] == 0x00 || entry[0] == 0xE5 {
                continue;
            }

            let raw_name = &entry[0..11];
            let file_name = core::str::from_utf8(raw_name).unwrap_or("").trim();

            let cleaned_name = file_name.replace(" ", "");

            if cleaned_name.eq_ignore_ascii_case(name) {
                let cluster = u16::from_le_bytes([entry[26], entry[27]]);
                let size = u32::from_le_bytes([
                    entry[28], entry[29], entry[30], entry[31],
                ]);
                return Some((cluster, size));
            }
        }
    }
    None
}

pub fn read_file_contents(&self, filename: &str) {
    if let Some((mut cluster, size)) = self.find_file(filename) {
        vga_println!("Arquivo {} ({} bytes)", filename, size);

        let mut remaining = size;
        while cluster < 0xFF8 {
            let lba = 33 + (cluster - 2) as usize;
            let sector = self.read_sector(lba);

            let to_read = core::cmp::min(remaining, SECTOR_SIZE as u32);
            let content = &sector[..to_read as usize];

            for &byte in content {
                if byte == b'\r' || byte == 0 {
                    continue;
                } else if byte == b'\n' {
                    vga_println!();
                } else {
                    use crate::vga_buffer::vga_print;
                    vga_print!("{}", byte as char);
                }
            }

            remaining -= to_read;
            cluster = self.read_fat_entry(cluster);
        }

        vga_println!();
    } else {
        vga_println!("Arquivo '{}' não encontrado", filename);
    }
}

impl Filesystem for Fat12Volume {
    fn open(&self, filename: &str) -> Option<File> {
        let (mut cluster, size) = self.find_file(filename)?;
        let mut remaining = size;
        let mut data = Vec::new();

        while cluster < 0xFF8 && remaining > 0 {
            let lba = 33 + (cluster - 2) as usize;
            let sector = self.read_sector(lba);
            let to_read = core::cmp::min(remaining, SECTOR_SIZE as u32);
            data.extend_from_slice(&sector[..to_read as usize]);

            remaining -= to_read;
            cluster = self.read_fat_entry(cluster);
        }

        Some(File {
            name: filename.to_string(),
            pos: 0,
            data,
        })
    }

    fn open(&self, path: &str) -> Option<File> {
        let path = path.trim_matches('/');

        if let Some((mut cluster, size)) = self.find_file(path) {
            let mut remaining = size;
            let mut data = Vec::new();

            while cluster < 0xFF8 && remaining > 0 {
                let lba = 33 + (cluster - 2) as usize;
                let sector = self.read_sector(lba);
                let to_read = core::cmp::min(remaining, SECTOR_SIZE as u32);
                data.extend_from_slice(&sector[..to_read as usize]);

                remaining -= to_read;
                cluster = self.read_fat_entry(cluster);
            }

            return Some(File {
                name: path.to_string(),
                pos: 0,
                data,
            });
        }

        None
    }

    fn list_dir(&self, path: &str) -> Option<Directory> {
        let path = path.trim_matches('/');

        if path.is_empty() {
            // root dir: setores 19–32
            let mut entries = Vec::new();

            for sector in 19..=32 {
                let data = self.read_sector(sector);

                for entry in data.chunks(32) {
                    if entry[0] == 0x00 || entry[0] == 0xE5 {
                        continue;
                    }

                    let raw_name = &entry[0..11];
                    let name = core::str::from_utf8(raw_name).unwrap_or("").trim().replace(" ", "");

                    let attr = entry[11];
                    let is_dir = attr & 0x10 != 0;

                    let cluster = u16::from_le_bytes([entry[26], entry[27]]);
                    let size = u32::from_le_bytes([entry[28], entry[29], entry[30], entry[31]]);

                    entries.push(DirEntry {
                        name,
                        is_dir,
                        cluster,
                        size,
                    });
                }
            }

            Some(Directory {
                name: "/".into(),
                entries,
            })
        } else {
            // subdiretório
            let dir = self.read_directory_from_cluster(
                self.find_file(path)?.0
            );

            Some(Directory {
                name: path.into(),
                entries: dir,
            })
        }
    }
}


