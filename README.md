# QuasarX OS

**QuasarX OS** is a modular, terminal-first operating system built from scratch with a custom kernel written primarily in **Rust**, with key low-level components in **C**. Its goal is to explore the inner workings of modern operating systems with emphasis on performance, low-level control, and security.

> ⚠️ This is a work-in-progress system. It is functional but **not production-ready**.

---

## 🚀 Overview

- 🧠 **Kronos Kernel**: A monolithic, preemptive multitasking kernel with paging, trap handling, memory allocators, and interrupt control.
- 🐚 **nnsh Shell**: A command-line interface resembling UNIX shells, with built-in command dispatch and user scripting.
- 📦 **io.initX Init System**: A BSD-style service manager responsible for booting the userland, daemons, and service tree.
- 📁 **FAT Filesystem Support**: Includes FAT12/16/32 support with long filename (LFN) capabilities and mountable USB volumes.
- 🌐 **Full Network Stack**: ARP, IPv4, ICMP, UDP, DNS, TCP with basic socket abstraction and daemons.
- 🔐 **POSIX-style Auth & Permissions**: User/group management, `/etc/passwd`, `UID/GID`, ACL support and `sudo`.

---

## 🔧 Project Goals

- Build a full OS from bootloader to userland in Rust/C.
- Teach kernel internals: GDT/IDT, paging, heap, USB, file systems, and networking.
- Provide a minimal yet extensible userland environment (CLI-first).
- Stay modular: replaceable shell, init, drivers, FS, etc.
- Emphasize low-latency terminal computing over heavy GUIs.

---

## 🧱 System Components

| Component       | Description                                  |
|----------------|----------------------------------------------|
| `Kronos`       | Core kernel with task switching, MMU, traps  |
| `nnsh`         | Default shell (Unix-like CLI)                |
| `ceres`        | Minimal terminal text editor                 |
| `sirius`       | Terminal-based ASCII/UTF-8 web browser       |
| `lilica`       | Optional graphical window manager (command-started) |
| `io.initX`     | BSD-style init system for service control    |
| `pkg`          | Built-in package manager (install/remove/list) |

---

## 📡 Networking Stack

- ARP (Address Resolution Protocol)
- IPv4 Layer with routing logic
- ICMP (ping support)
- UDP + basic DNS resolver (`nslookup`)
- TCP with basic stream interface and services
- DHCP client (optional)
- FTP/TFTP (under implementation)
- TLS (in progress)

---

## 🔐 Security and Userland

- UID/GID with ACL and mount permissions
- Login/logout with credential storage
- Command access controlled via permissions
- Service visibility and privilege restrictions
- Built-in firewall and service access control

---

## 🧪 Status

| Feature                  | Status      |
|--------------------------|-------------|
| Kernel                  | ✅ Functional (multitasking, paging) |
| Shell                   | ✅ Working (nnsh) |
| Filesystem              | ✅ FAT with LFN support |
| Networking              | ✅ UDP, TCP, DNS, ICMP implemented |
| Userland                | ✅ Basic tools: ceres, sirius |
| Init system             | ✅ io.initX (BSD-style) |
| USB support             | ✅ OHCI and keyboard/mouse |
| Package Manager         | ✅ `pkg install/remove/list` |
| GUI/WM (optional)       | 🧪 Prototype (LilicaWM) |
| TLS & Secure Comms      | 🚧 In progress |
| Web browser             | 🚧 CLI-mode only (Sirius) |

---

## 🛠️ Build & Run

**To run in QEMU:**

sh
qemu-system-x86_64 -m 512M -drive format=raw,file=quasarx.img

**You can also create a bootable USB image using:**

sh
dd if=quasarx.img of=/dev/sdX bs=4M status=progress

📝 License
MIT License – For educational and experimental purposes only.

Welcome to QuasarX OS — a handcrafted operating system and kernel project, built from first principles with passion and curiosity.
