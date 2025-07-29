use crate::vga_buffer::{vga_print, vga_println};
use crate::vfs::{VFS_INSTANCE};
use alloc::{string::String, vec::Vec};

pub fn run_shell() {
    let mut cwd = String::from("/");

    loop {
        vga_print!("[{}]$ ", cwd);
        let line = read_line();

        let mut parts = line.trim().split_whitespace();
        let cmd = match parts.next() {
            Some(c) => c,
            None => continue,
        };

        match cmd {
            "help" => {
                vga_println!("Comandos disponíveis:");
                vga_println!("  help          - mostra esta ajuda");
                vga_println!("  ls            - lista arquivos");
                vga_println!("  cat <arquivo> - mostra conteúdo");
                vga_println!("  cd <dir>      - muda de diretório");
                vga_println!("  clear         - limpa a tela");
            }

            "ls" => {
                let dir = VFS_INSTANCE.lock().list_dir(&cwd);
                match dir {
                    Some(d) => {
                        for e in d.entries {
                            if e.is_dir {
                                vga_println!("<DIR> {}", e.name);
                            } else {
                                vga_println!("     {} ({} bytes)", e.name, e.size);
                            }
                        }
                    }
                    None => vga_println!("Diretório inválido"),
                }
            }

            "cd" => {
                let arg = parts.next();
                if let Some(target) = arg {
                    if target == ".." && cwd != "/" {
                        if let Some(pos) = cwd.rfind('/') {
                            cwd.truncate(pos);
                            if cwd.is_empty() {
                                cwd = "/".into();
                            }
                        }
                    } else {
                        let mut new_path = cwd.clone();
                        if !cwd.ends_with('/') {
                            new_path.push('/');
                        }
                        new_path.push_str(target);

                        if VFS_INSTANCE.lock().list_dir(&new_path).is_some() {
                            cwd = new_path;
                        } else {
                            vga_println!("Diretório não encontrado");
                        }
                    }
                } else {
                    vga_println!("Uso: cd <diretório>");
                }
            }

            "cat" => {
                let filename = parts.next();
                if let Some(f) = filename {
                    let mut full_path = cwd.clone();
                    if !cwd.ends_with('/') {
                        full_path.push('/');
                    }
                    full_path.push_str(f);

                    if let Some(mut file) = VFS_INSTANCE.lock().open(&full_path) {
                        while let Some(b) = file.read_byte() {
                            if b == b'\n' {
                                vga_println!();
                            } else {
                                vga_print!("{}", b as char);
                            }
                        }
                        vga_println!();
                    } else {
                        vga_println!("Arquivo não encontrado");
                    }
                } else {
                    vga_println!("Uso: cat <arquivo>");
                }
            }

            "clear" => {
                crate::vga_buffer::clear_screen();
            }

            _ => {
                vga_println!("Comando não reconhecido: '{}'", cmd);
            }

"exec" => {
    if let Some(f) = parts.next() {
        let mut full_path = cwd.clone();
        if !cwd.ends_with('/') {
            full_path.push('/');
        }
        full_path.push_str(f);

        if let Some(mut file) = VFS_INSTANCE.lock().open(&full_path) {
            let data = file.data;

            const LOAD_ADDR: usize = 0x50000;
            let exec_mem = LOAD_ADDR as *mut u8;

            unsafe {
                for (i, byte) in data.iter().enumerate() {
                    core::ptr::write_volatile(exec_mem.add(i), *byte);
                }

                vga_println!("Executando '{}'", f);

                let entry: extern "C" fn() = core::mem::transmute(LOAD_ADDR);
                entry(); // executa o binário

                vga_println!("\nFim da execução de '{}'", f);
            }
        } else {
            vga_println!("Binário '{}' não encontrado", f);
        }
    } else {
        vga_println!("Uso: exec <arquivo>");
    }

        }
    }
}

fn read_line() -> String {
    use x86_64::instructions::interrupts;
    use spin::Mutex;
    use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
    use crate::keyboard::KEYBOARD;
    use crate::vga_buffer::vga_print;

    let mut buf = String::new();

    loop {
        interrupts::disable();
        let mut keyboard = KEYBOARD.lock();
        if let Some(key_event) = keyboard.next_key() {
            if let Some(key) = key_event {
                match key {
                    DecodedKey::Unicode(c) => {
                        if c == '\n' || c == '\r' {
                            vga_println!();
                            break;
                        } else if c == '\x08' || c == '\x7F' { // backspace
                            if !buf.is_empty() {
                                buf.pop();
                                vga_print!("\x08 \x08");
                            }
                        } else {
                            buf.push(c);
                            vga_print!("{}", c);
                        }
                    }
                    _ => {}
                }
            }
        }
        interrupts::enable();
    }

    buf
}
