#![no_std]
#![allow(clippy::unusual_byte_groupings)]

use embedded_hal::digital::{OutputPin, PinState};

// Addressing, as documented in Tasmota xlgt_09_sm2335.ino
//
// Select the chip and perform  perform / mode to enter.
// 0bDD0MMNNN
//   ^^----------- DD, identification = 11
//     ^---------- reserved = 0
//      ^^-------- MM, mode:         standby = 00
//                       3 channels    (RGB) = 01
//                       2 channels     (CW) = 10
//                       5 channels (RGB+CW) = 11
//        ^^^----- NNN, offset: value 0b000 to 0b100 => start at OUT1 to OUT5
const ADDR_STANDBY: u8 = 0b11_0_00_000;
const ADDR_START_3CH: u8 = 0b11_0_01_000;
const ADDR_START_2CH: u8 = 0b11_0_10_000;
const ADDR_START_5CH: u8 = 0b11_0_11_000;

pub const BIT_DEPTH: u8 = 10;

pub struct Sm2335Egh<D, C> {
    data: D,
    clk: C,
    rgb_power_level: u8,
    cw_power_level: u8,
}

impl<D, C> Sm2335Egh<D, C>
where
    D: OutputPin,
    C: OutputPin,
{
    pub fn init(mut data_pin: D, mut clk_pin: C) -> Self {
        data_pin.set_high().ok();
        clk_pin.set_high().ok();
        Self { data: data_pin, clk: clk_pin, rgb_power_level: 0x2, cw_power_level: 0x4 }
    }

    /// Power levels in current (mA), as documented in ESPHome sm10bit_base.cpp
    ///
    /// | HEX | RGB level | White level | Comment             |
    /// |-----|-----------|-------------|---------------------|
    /// | 0x0 |      10mA |         5mA |                     |
    /// | 0x1 |      20mA |        10mA |                     |
    /// | 0x2 |      30mA |        15mA | Default color value |
    /// | 0x3 |      40mA |        20mA |                     |
    /// | 0x4 |      50mA |        25mA | Default white value |
    /// | 0x5 |      60mA |        30mA |                     |
    /// | 0x6 |      70mA |        35mA |                     |
    /// | 0x7 |      80mA |        40mA |                     |
    /// | 0x8 |      90mA |        45mA |                     |
    /// | 0x9 |     100mA |        50mA |                     |
    /// | 0xA |     110mA |        55mA |                     |
    /// | 0xB |     120mA |        60mA |                     |
    /// | 0xC |     130mA |        65mA |                     |
    /// | 0xD |     140mA |        70mA |                     |
    /// | 0xE |     150mA |        75mA |                     |
    /// | 0xF |     160mA |        80mA |                     |
    pub fn set_power_levels(&mut self, rgb_level: u8, cw_level: u8) {
        self.rgb_power_level = rgb_level & 0xF;
        self.cw_power_level = cw_level & 0xF;
    }

    /// Write the values of all 5 channels to the controller, with each channel value given as a normalized float.
    ///
    /// Like [`Self::write`], but each channel value is given as a normalized flot in the range `[0.0, 1.0)`.
    pub fn write_normalized(&mut self, channel_values: &[f32; 5]) {
        self.write(&core::array::from_fn(|i| (channel_values[i] * (1u16 << BIT_DEPTH) as f32) as u16))
    }

    /// Write the values of all 5 channels to the controller, with each channel value given as a 10-bit integer
    ///
    /// To clarify, the channel values should be in the range `[0, 2^{BIT_DEPTH} = 1024)`).
    /// Unlike [`Self::write_normalized`], this version doesn't use any float ops.
    ///
    /// Mainly in order to reduce power usage (I assume that's the reason), there are 4 different "modes" the controller can be in.
    /// They are
    /// - standby, with all channels disabled;
    /// - 3 channel mode with OUT1, OUT2, and OUT3 enabled -- typically the RGB channels;
    /// - 2 channel mode with OUT4 and OUT5 enabled -- typically the white channels (cool & warm);
    /// - 5 channel mode with all outputs enabled.
    /// The different modes will be entered automatically depending on which elements in the argument array are zero.
    ///
    /// # Examples
    /// ```
    /// # use embedded_hal::digital::OutputPin;
    /// # fn example<P: OutputPin>(data_pin: P, clock_pin: P) {
    /// use sm2335egh::Sm2335Egh;
    /// let mut led_controller = Sm2335Egh::init(data_pin, clock_pin);
    /// // Depending on the board, OUT1 may not necessarily be used to drive the red color channel etc
    /// let (red, green, blue) = (1023, 0, 800);
    /// // By leaving OUT4-5 as zero while at least one of OUT1-3 is nonzero, we automatically enter the 3 channel mode.
    /// led_controller.write(&[blue, red, green, 0, 0]);
    /// # }
    /// ```
    pub fn write(&mut self, channel_values: &[u16; 5]) {
        let mut msg = Msg::zeroed();
        msg.set_channel_values(channel_values);
        match channel_values {
            [0, 0, 0, 0, 0] => {
                msg.set_addr(ADDR_STANDBY);
            }
            [_rgb @ .., 0, 0] => {
                msg.set_addr(ADDR_START_3CH);
                msg.set_rgb_power_level(self.rgb_power_level);
            }
            [0, 0, 0, _cw @ ..] => {
                msg.set_addr(ADDR_START_2CH);
                msg.set_cw_power_level(self.cw_power_level);
            }
            _all => {
                msg.set_addr(ADDR_START_5CH);
                msg.set_rgb_power_level(self.rgb_power_level);
                msg.set_cw_power_level(self.cw_power_level);
            }
        }
        self.write_msg(&msg)
    }

    fn write_msg(&mut self, msg: &Msg) {
        self.data.set_low().ok();
        for byte in msg.0 {
            for i in (0..8).rev() {
                let bit = ((byte >> i) & 1) == 1;
                self.clk.set_low().ok();
                self.data.set_state(PinState::from(bit)).ok();
                self.clk.set_high().ok();
            }
            self.clk.set_low().ok();
            self.data.set_high().ok();
            self.clk.set_high().ok();
        }
        self.clk.set_low().ok();
        self.clk.set_high().ok();
        self.data.set_high().ok();
    }
}

struct Msg([u8; 12]);

impl Msg {
    fn zeroed() -> Self {
        Msg([0; 12])
    }

    fn set_addr(&mut self, addr: u8) {
        self.0[0] = addr;
    }

    fn set_rgb_power_level(&mut self, lvl: u8) {
        self.0[1] = (lvl << 4) | (self.0[1] & 0x0F);
    }

    fn set_cw_power_level(&mut self, lvl: u8) {
        self.0[1] = (self.0[1] & 0xF0) | lvl & 0xF;
    }

    fn set_channel_values(&mut self, vals: &[u16; 5]) {
        for (i, &val) in vals.iter().enumerate() {
            self.0[2 + i * 2..][..2].copy_from_slice(val.min((1 << BIT_DEPTH) - 1).to_be_bytes().as_slice());
        }
    }
}
