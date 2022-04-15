use crate::utils;
use crate::utils::Error;
use std::string::String;

type Handler = fn(&mut GPU) -> Result<(), String>;

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

    gp0_command: CommandBuffer,
    gp0_command_remaining: u32,
    gp0_command_method: Handler,
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
            drawing_area: DrawingArea {
                left: 0,
                right: 0,
                top: 0,
                bottom: 0,
            },
            drawing_offset: (0, 0),
            display_vram_start: (0, 0),
            display_horiz_range: (0, 0),
            display_line_range: (0, 0),

            gp0_command: CommandBuffer::new(),
            gp0_command_remaining: 0,
            gp0_command_method: GPU::gp0_nop,
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
            // | (self.vres as u32) << 19
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
            0 => self.read(),
            4 => self.status(),
            _ => unreachable!(),
        };
        utils::to_t(value)
    }

    pub fn gp0(&mut self, val: u32) -> Result<(), String> {
        if self.gp0_command_remaining == 0 {
            let opcode = (val >> 24) & 0xff;

            let (len, method): (u32, Handler) = match opcode {
                0x00 => (1, GPU::gp0_nop),
                0x01 => (1, GPU::gp0_clear_cache),
                0x28 => (5, GPU::gp0_quad_mono_opaque),
                0xe1 => (1, GPU::gp0_draw_mode),
                0xe2 => (1, GPU::gp0_texture_window),
                0xe3 => (1, GPU::gp0_drawing_area_top_left),
                0xe4 => (1, GPU::gp0_drawing_area_bottom_right),
                0xe5 => (1, GPU::gp0_drawing_offset),
                0xe6 => (1, GPU::gp0_mask_bit_setting),
                _ => return Error!("Unhandled GP0 command 0x{:08x}", val),
            };
            self.gp0_command_remaining = len;
            self.gp0_command_method = method;

            self.gp0_command.clear();
        }
        self.gp0_command.push(val);
        self.gp0_command_remaining -= 1;

        if self.gp0_command_remaining == 0 {
            return (self.gp0_command_method)(self);
        }
        Ok(())
    }

    pub fn gp0_nop(&mut self) -> Result<(), String> { Ok(()) }

    pub fn gp0_clear_cache(&mut self) -> Result<(), String> {
        debug!("Unimplemented gp0 clear cache command");
        Ok(())
    }

    pub fn gp0_quad_mono_opaque(&mut self) -> Result<(), String> {
        println!("Draw quad");
        Ok(())
    }

    fn gp0_draw_mode(&mut self) -> Result<(), String> {
        let val = self.gp0_command[0];

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

    fn gp0_texture_window(&mut self) -> Result<(), String> {
        let val = self.gp0_command[0];

        let x_mask = (val & 0x1f) as u8;
        let y_mask = ((val >> 5) & 0x1f) as u8;
        self.texture_window_mask = (x_mask, y_mask);

        let x_offset = ((val >> 10) & 0x1f) as u8;
        let y_offset = ((val >> 15) & 0x1f) as u8;
        self.texture_window_offset = (x_offset, y_offset);
        Ok(())
    }

    fn gp0_drawing_area_top_left(&mut self) -> Result<(), String> {
        let val = self.gp0_command[0];

        self.drawing_area.top = ((val >> 10) & 0x3ff) as u16;
        self.drawing_area.left = (val & 0x3ff) as u16;
        Ok(())
    }

    fn gp0_drawing_area_bottom_right(&mut self) -> Result<(), String> {
        let val = self.gp0_command[0];

        self.drawing_area.top = ((val >> 10) & 0x3ff) as u16;
        self.drawing_area.left = (val & 0x3ff) as u16;
        Ok(())
    }

    fn gp0_drawing_offset(&mut self) -> Result<(), String> {
        let val = self.gp0_command[0];

        let x = (val & 0x7ff) as u16;
        let y = ((val >> 11) & 0x7ff) as u16;

        let x_se = ((x << 5) as i16) >> 5; // what the fuck
        let y_se = ((y << 5) as i16) >> 5;

        self.drawing_offset = (x_se, y_se);
        Ok(())
    }

    fn gp0_mask_bit_setting(&mut self) -> Result<(), String> {
        let val = self.gp0_command[0];

        self.force_set_mask_bit = (val & 1) != 0;
        self.preserve_masked_pixels = (val & 2) != 0;
        Ok(())
    }

    pub fn gp1(&mut self, val: u32) -> Result<(), String> {
        let opcode = (val >> 24) & 0xff;

        match opcode {
            0x00 => self.gp1_reset(val),
            0x04 => self.gp1_dma_direction(val),
            0x05 => self.gp1_display_vram_start(val),
            0x06 => self.gp1_display_horizontal_range(val),
            0x07 => self.gp1_display_vertical_range(val),
            0x08 => self.gp1_display_mode(val),
            _ => Error!("Unhandled GP1 command 0x{:08x}", val),
        }
    }

    fn gp1_reset(&mut self, _: u32) -> Result<(), String> {
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
        self.drawing_area = DrawingArea {
            left: 0,
            right: 0,
            top: 0,
            bottom: 0,
        };
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
        Ok(())
    }

    fn gp1_dma_direction(&mut self, val: u32) -> Result<(), String> {
        self.dma_direction = match val & 3 {
            0 => DMADirection::Off,
            1 => DMADirection::FIFO,
            2 => DMADirection::CPU2GP0,
            3 => DMADirection::VRAM2CPU,
            _ => unreachable!(),
        };
        Ok(())
    }

    fn gp1_display_vram_start(&mut self, val: u32) -> Result<(), String> {
        let x = (val & 0x3fe) as u16;
        let y = ((val >> 10) & 0x1ff) as u16;

        self.display_vram_start = (x, y);
        Ok(())
    }

    fn gp1_display_horizontal_range(&mut self, val: u32) -> Result<(), String> {
        let start = (val & 0xfff) as u16;
        let end = ((val >> 12) & 0xfff) as u16;
        self.display_horiz_range = (start, end);
        Ok(())
    }

    fn gp1_display_vertical_range(&mut self, val: u32) -> Result<(), String> {
        let start = (val & 0x3ff) as u16;
        let end = ((val >> 10) & 0x3ff) as u16;
        self.display_line_range = (start, end);
        Ok(())
    }

    fn gp1_display_mode(&mut self, val: u32) -> Result<(), String> {
        let hr1 = (val & 3) as u8;
        let hr2 = ((val >> 6) & 1) as u8;

        self.hres = HorizontalRes::from_fields(hr1, hr2);

        self.vres = match val & 0x4 != 0 {
            false => VerticalRes::Y240,
            true => VerticalRes::Y480,
        };

        self.vmode = match val & 0x8 != 0 {
            true => VMode::PAL,
            false => VMode::NTSC,
        };

        self.display_depth = match val & 0x10 != 0 {
            true => DisplayDepth::D15,
            false => DisplayDepth::D24,
        };

        self.interlacing = val & 0x20 != 0;

        if val & 0x80 != 0 {
            return Error!("Unsupported display mode 0x{:08x}", val);
        }
        Ok(())
    }

    pub fn read(&self) -> u32 {
        0
    }
}

struct CommandBuffer {
    data: [u32; 12],
    len: u8,
}

impl CommandBuffer {
    fn new() -> Self {
        Self {
            data: [0; 12],
            len: 0,
        }
    }

    fn clear(&mut self) {
        self.len = 0;
    }

    fn push(&mut self, word: u32) {
        if self.len >= 12 {
            panic!("Command command buffer index out of range");
        }
        self.data[self.len as usize] = word;
        self.len += 1;
    }
}

impl ::std::ops::Index<usize> for CommandBuffer {
    type Output = u32;

    fn index<'a>(&'a self, index: usize) -> &'a u32 {
        if index >= self.len as usize {
            panic!(
                "Command buffer index out of range: {:?} ({:?})",
                index, self.len
            );
        }
        &self.data[index]
    }
}
