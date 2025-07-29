use x86_64::instructions::port::Port;

const PIT_CHANNEL0: u16 = 0x40;
const PIT_COMMAND: u16 = 0x43;

/// Frequência base do PIT: 1.193.182 Hz
const PIT_FREQUENCY: u32 = 1193182;

/// Frequência desejada em Hz (ex: 100Hz → 10ms por tick)
const TARGET_HZ: u32 = 100;

pub fn init_pit() {
    let divisor = PIT_FREQUENCY / TARGET_HZ;

    unsafe {
        let mut command = Port::new(PIT_COMMAND);
        let mut channel0 = Port::new(PIT_CHANNEL0);

        // Modo 3 (square wave generator), canal 0, acesso low/high
        command.write(0x36u8);

        // Envia o divisor: low byte, depois high byte
        channel0.write((divisor & 0xFF) as u8);
        channel0.write((divisor >> 8) as u8);
    }
}
