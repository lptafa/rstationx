use crate::utils;
use crate::utils::Error;
use std::string::String;

#[derive(Clone, Copy, Debug)]
enum TextureDepth {
    T4 = 0,
    T8 = 1,
    T15 = 2,
}

#[derive(Clone, Copy, Debug)]
enum Field {
    Bottom = 0,
    Top = 1,
}

#[derive(Clone, Copy, Debug)]
struct HorizontalRes(u8);

impl HorizontalRes {
    fn from_fields(hr1: u8, hr2: u8) -> Self {
        Self((hr2 & 1) | ((hr1 & 3) << 1))
    }

    fn into_status(self) -> u32 {
        (self.0 as u32) << 16
    }
}

#[derive(Clone, Copy, Debug)]
enum VerticalRes {
    Y240 = 0,
    Y480 = 1,
}

#[derive(Clone, Copy, Debug)]
enum VMode {
    NTSC = 0,
    PAL = 1,
}

#[derive(Clone, Copy, Debug)]
enum DisplayDepth {
    D15 = 0,
    D24 = 1,
}

#[derive(Clone, Copy, Debug)]
enum DMADirection {
    Off = 0,
    FIFO = 1,
    CPU2GP0 = 2, // Leet H4x0r Mustafa's idea ;^)
    VRAM2CPU = 3,
}

#[derive(Clone, Copy, Debug)]
struct DrawingArea {
    left: u16,
    right: u16,
    top: u16,
    bottom: u16,
}

pub struct GPU {
    semi_transparency: u8,
    texture_base: (u8, u8),
    texture_depth: TextureDepth,

    texture_disable: bool,
    draw_to_display: bool,
    force_set_mask_bit: bool,
    preserve_masked_pixels: bool,
    interlacing: bool,
    display_disabled: bool,
    dithering: bool,
    interrupt: bool,
    texture_flip: (bool, bool),

    hres: HorizontalRes,
    vres: VerticalRes,
    field: Field,
    vmode: VMode,
    display_depth: DisplayDepth,
    dma_direction: DMADirection,

    texture_window_mask: (u8, u8),
    texture_window_offset: (u8, u8),
    drawing_area: DrawingArea,
    drawing_offset: (i16, i16),
    display_vram_start: (u16, u16),
    display_horiz_range: (u16, u16),
    display_line_range: (u16, u16),
}

impl GPU {
    pub fn new() -> GPU {
        GPU {
            semi_transparency: 0,
            texture_base: (0, 0),
            texture_depth: TextureDepth::T4,

            texture_disable: false,
            draw_to_display: false,
            force_set_mask_bit: false,
            preserve_masked_pixels: false,
            interlacing: false,
            display_disabled: true,
            dithering: false,
            interrupt: false,
            texture_flip: (false, false),

            hres: HorizontalRes::from_fields(0, 0),
            vres: VerticalRes::Y240,
            field: Field::Top,
            vmode: VMode::NTSC,
            display_depth: DisplayDepth::D15,
            dma_direction: DMADirection::Off,

            texture_window_mask: (0, 0),
            texture_window_offset: (0, 0),
            drawing_area: DrawingArea { left: 0, right: 0, top: 0, bottom: 0 },
            drawing_offset: (0, 0),
            display_vram_start: (0, 0),
            display_horiz_range: (0, 0),
            display_line_range: (0, 0),
        }
    }

    pub fn status(&self) -> u32 {
        let r = 0
            | (self.texture_base.0 as u32) << 0
            | (self.texture_base.1 as u32) << 4
            | (self.semi_transparency as u32) << 5
            | (self.texture_depth as u32) << 7
            | (self.dithering as u32) << 9
            | (self.draw_to_display as u32) << 10
            | (self.force_set_mask_bit as u32) << 11
            | (self.preserve_masked_pixels as u32) << 12
            | (self.field as u32) << 13
            | (self.texture_disable as u32) << 15
            | self.hres.into_status()
            | (self.vres as u32) << 19
            | (self.vmode as u32) << 20
            | (self.display_depth as u32) << 21
            | (self.interlacing as u32) << 22
            | (self.display_disabled as u32) << 23
            | (self.interrupt as u32) << 24
            | 1 << 26
            | 1 << 27
            | 1 << 28
            | (self.dma_direction as u32) << 29
            | 0 << 31;

        let dma_request = match self.dma_direction {
            DMADirection::Off => 0,
            DMADirection::FIFO => 1,
            DMADirection::VRAM2CPU => (r >> 27) & 1,
            DMADirection::CPU2GP0 => (r >> 28) & 1,
        };

        r | dma_request << 25
    }

    pub fn load<T: TryFrom<u32>>(&self, offset: u32) -> T {
        let value: u32 = match offset {
            4 => 0x1c000000,
            _ => 0,
        };
        utils::to_t(value)
    }

    pub fn gp0(&mut self, val: u32) -> Result<(), String> {
        let opcode = (val >> 24) & 0xff;

        match opcode {
            0x00 => Ok(()),
            0xe1 => self.gp0_draw_mode(val),
            _ => Error!("Unhandled GP0 command 0x{:08x}", val),
        }
    }

    pub fn gp1(&mut self, val: u32) -> Result<(), String> {
        let opcode = (val >> 24) & 0xff;

        match opcode {
            0x00 => self.gp1_reset(val),
            _ => Error!("Unhandled GP1 command 0x{:08x}", val),
        }
    }

    fn gp0_draw_mode(&mut self, val: u32) -> Result<(), String> {
        self.texture_base.0 = (val & 0xf) as u8;
        self.texture_base.1 = ((val >> 4) & 1) as u8;
        self.semi_transparency = ((val >> 5) & 3) as u8;

        self.texture_depth = match (val >> 7) & 3 {
            0 => TextureDepth::T4,
            1 => TextureDepth::T8,
            2 => TextureDepth::T15,
            n => return Error!("Unhandled texture depth {:?}", n),
        };

        self.dithering = ((val >> 9) & 1) != 0;
        self.draw_to_display = ((val >> 10) & 1) != 0;
        self.texture_disable = ((val >> 11) & 1) != 0;
        self.texture_flip.0 = ((val >> 12) & 1) != 0;
        self.texture_flip.1 = ((val >> 13) & 1) != 0;

        Ok(())
    }

    fn gp1_reset(&mut self, _: u32) {
        self.interrupt = false;

        self.texture_base = (0, 0);
        self.semi_transparency = 0;
        self.texture_depth = TextureDepth::T4;
        self.texture_window_mask = (0, 0);
        self.texture_window_offset = (0, 0);
        self.dithering = false;
        self.draw_to_display = false;
        self.texture_disable = false;
        self.texture_flip = (false, false);
        self.drawing_area = DrawingArea{ left: 0, right: 0, top: 0, bottom: 0 };
        self.drawing_offset = (0, 0);
        self.force_set_mask_bit = false;
        self.preserve_masked_pixels = false;

        self.dma_direction = DMADirection::Off;

        self.display_disabled = true;
        self.display_vram_start = (0, 0);
        self.hres = HorizontalRes::from_fields(0, 0);
        self.vres = VerticalRes::Y240;

        self.vmode = VMode::NTSC;
        self.interlacing = true;
        self.display_horiz_range = (0x200, 0xc00);
        self.display_line_range = (0x10, 0x100);
        self.display_depth = DisplayDepth::D15;
    }
}
