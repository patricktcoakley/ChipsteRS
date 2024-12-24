use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct Quirks: u8 {
        const VF_RESET = 0b0000_0001;  // VF reset after logic ops
        const LOAD_STORE_INC_I = 0b0000_0010;  // True: Do not increment I after load/store
        const VBLANK    = 0b0000_0100;  // Wait for vblank on draw vs draw immediately
        const WRAP    = 0b0000_1000;  // True: sprites wrap, false: sprites clip
        const SHIFT = 0b0001_0000;  // True: shifts load VY into VX before shifting
        const JUMP = 0b0010_0000;  // True: jump to XNN + VX, False: jump to NNN + V0
    }
}

#[derive(Debug, Clone)]
pub enum Target {
    CosmacVIP,
    Modern,
    Chip48,
    SuperChip,
    XoChip,
}

#[derive(Debug, Clone)]
pub struct Platform {
    pub target: Target,
    pub video_width: u16,
    pub video_height: u16,
    pub quirks: Quirks,
    pub tick_rate: u16,
}

impl Platform {
    pub fn new(variant: Target) -> Self {
        match variant {
            Target::CosmacVIP => Self {
                target: variant,
                video_width: 64,
                video_height: 32,
                quirks: Quirks::VF_RESET | Quirks::VBLANK,
                tick_rate: 15,
            },
            Target::Modern => Self {
                target: variant,
                video_width: 64,
                video_height: 32,
                quirks: Quirks::VF_RESET | Quirks::VBLANK,
                tick_rate: 12,
            },
            Target::Chip48 => Self {
                target: variant,
                video_width: 64,
                video_height: 32,
                quirks: Quirks::SHIFT | Quirks::JUMP,
                tick_rate: 30,
            },
            Target::SuperChip => Self {
                target: variant,
                video_width: 128,
                video_height: 64,
                quirks: Quirks::LOAD_STORE_INC_I,
                tick_rate: 30,
            },
            Target::XoChip => Self {
                target: variant,
                video_width: 128,
                video_height: 64,
                quirks: Quirks::WRAP,
                tick_rate: 100,
            },
        }
    }

    pub fn default() -> Self {
        Self::new(Target::CosmacVIP)
    }

    pub fn has_quirk(&self, quirk: Quirks) -> bool {
        self.quirks.contains(quirk)
    }
}