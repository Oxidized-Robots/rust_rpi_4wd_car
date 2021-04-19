// Copyright © 2021-present, Michael Cummings
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
// MIT License
//
// Copyright © 2021-present, Michael Cummings <mgcummings@yahoo.com>.
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.
//! Contains all the human interactive components.

use crate::{Result, Rr4cError, Rr4cResult};
use rppal::gpio::{Gpio, IoPin, Level, Mode, OutputPin, PullUpDown};
use std::thread::sleep;
use std::time::Duration;

/// Proven easier access to audio, visual, and other forms of human interaction
/// with the robot.
#[derive(Debug)]
pub struct Hids {
    /// Instance of [IoPin] connected to both the buzzer and the `key` button.
    ///
    /// [IoPin]: rppal::gpio::IoPin
    buzz_key: IoPin,
    /// Instance of [OutputPin] connected to the fan motor.
    ///
    /// [OutputPin]: rppal::gpio::OutputPin
    fan: OutputPin,
    /// Instance of [OutputPin] connected to red LEDs.
    ///
    /// [OutputPin]: rppal::gpio::OutputPin
    led_r: OutputPin,
    /// Instance of [OutputPin] connected to green LEDs.
    ///
    /// [OutputPin]: rppal::gpio::OutputPin
    led_g: OutputPin,
    /// Instance of [OutputPin] connected to blue LEDs.
    ///
    /// [OutputPin]: rppal::gpio::OutputPin
    led_b: OutputPin,
}

impl Hids {
    /// Constructor
    pub fn new() -> Rr4cResult<Self> {
        let gpio = Gpio::new()?;
        let mut buzz_key = gpio.get(Self::BUZZ_KEY)?.into_io(Mode::Input);
        buzz_key.set_pullupdown(PullUpDown::PullUp);
        let mut fan = gpio.get(Self::FAN)?.into_output();
        fan.set_reset_on_drop(false);
        fan.set_high();
        let mut led_r = gpio.get(Self::LED_R)?.into_output();
        let mut led_g = gpio.get(Self::LED_G)?.into_output();
        let mut led_b = gpio.get(Self::LED_B)?.into_output();
        led_r.set_low();
        led_g.set_low();
        led_b.set_low();
        Ok(Self {
            buzz_key,
            fan,
            led_r,
            led_g,
            led_b,
        })
    }
    /// Used to sound buzzer.
    ///
    /// ## Arguments
    ///
    /// * `secs` - Optional number of seconds to sound buzzer.
    /// Defaults to 0.1 seconds.
    /// Internally limited between 0.1 and 10 secs.
    pub fn beep<S: Into<Option<f64>>>(&mut self, secs: S) {
        let secs = secs.into().unwrap_or(0.1).abs().min(10.0).max(0.1);
        let dur = Duration::from_secs_f64(secs);
        let off = Duration::from_secs_f64(0.01);
        self.buzz_key.set_mode(Mode::Output);
        // Ensure not already on.
        self.buzz_key.set_high();
        sleep(off);
        // On
        self.buzz_key.set_low();
        sleep(dur);
        // Off
        self.buzz_key.set_high();
        sleep(off);
        self.buzz_key.set_mode(Mode::Input);
    }
    /// Turn on the fan motor to blow out flame.
    ///
    /// ## Arguments
    ///
    /// * `secs` - Optional number of seconds to run fan.
    /// Defaults to 2 seconds.
    /// Internal limits to maximum of 60 seconds.
    pub fn blow<S: Into<Option<f64>>>(&mut self, secs: S) {
        let secs = secs.into().unwrap_or(2.0).abs().min(60.0);
        let wait = Duration::from_secs_f64(secs);
        self.fan.set_low();
        sleep(wait);
        self.fan.set_high();
        sleep(Duration::from_millis(100));
    }
    /// Waits for the `KEY` button on the robot to be pressed.
    ///
    /// Filters out noise/debounce the button press.
    pub fn key_press(&mut self) {
        let mut state = Level::High;
        let mut history = 0b01010101u8;
        self.buzz_key.set_mode(Mode::Input);
        self.buzz_key.set_pullupdown(PullUpDown::PullUp);
        let dur = Duration::from_millis(3);
        while history != 255 {
            let level = self.buzz_key.read();
            let changed = match level {
                Level::Low => state == Level::High,
                Level::High => state == Level::Low,
            };
            if changed {
                // Did we read 8 stable values in a row prior to
                // this (i.e. all bits are 0 or 1)?
                if history == 0 || history == 255 {
                    state = level
                }
            }
            history = match level {
                Level::Low => history.rotate_left(1) | 0b00000001,
                Level::High => history.rotate_left(1) & 0b11111110,
            };
            sleep(dur);
        }
    }
    /// Used to set all three color LEDs at the same time.
    ///
    /// ## Arguments
    ///
    /// * `red` - Optional brightness for the red LEDs.
    /// See [`set_red()`] for more info.
    /// * `green` - Optional brightness for the green LEDs.
    /// See [`set_green()`] for more info.
    /// * `blue` - Optional brightness for the blue LEDs.
    /// See [`set_blue()`] for more info.
    ///
    /// [`set_red()`]: Hids::set_red()
    /// [`set_green()`]: Hids::set_green()
    /// [`set_blue()`]: Hids::set_blue()
    pub fn lights<R, G, B>(&mut self, red: R, green: G, blue: B) -> Result
    where
        R: Into<Option<u8>>,
        G: Into<Option<u8>>,
        B: Into<Option<u8>>,
    {
        self.set_red(red)?;
        self.set_green(green)?;
        self.set_blue(blue)
    }
    /// Sets to brightness of the blue LEDs.
    ///
    /// ## Arguments
    ///
    /// * `brightness` - How brightly the LED should be lit. 0-100(%) range with
    /// 50% default if `None` is used.
    ///
    /// ## Examples
    ///
    /// ```edition2018
    /// # #[cfg(target_arch = "arm")]
    /// # {
    /// # extern crate rust_rpi_4wd_car;
    /// use rust_rpi_4wd_car::{Hids, Result};
    /// use std::{thread::sleep, time::Duration};
    ///
    /// fn main() -> Result {
    ///     let mut hids = Hids::new()?;
    ///     let pause = Duration::from_millis(50);
    ///     println!("Varying brightness of LEDs");
    ///     for i in (0..100).step_by(10) {
    ///         hids.set_blue(i)?;
    ///         sleep(pause);
    ///     }
    ///     hids.set_blue(0)
    /// }
    /// # }
    /// ```
    ///
    pub fn set_blue<C: Into<Option<u8>>>(&mut self, brightness: C) -> Result {
        let brightness = brightness.into().unwrap_or(50).min(100);
        if brightness != 0 {
            let dc = brightness as f64 * 0.01f64;
            self.led_b
                .set_pwm_frequency(Self::FREQUENCY, dc)
                .map_err(Rr4cError::Gpio)
        } else {
            self.led_b.clear_pwm().map_err(Rr4cError::Gpio)
        }
    }
    /// Set the LEDs to show a color from a preset list of primary and secondary
    /// colors.
    ///
    /// ## Arguments
    /// * `Index` - Index of a color from list which can be found in the
    /// constant [`LED_COLORS`] array.
    ///
    /// [`LED_COLORS`]: Hids::LED_COLORS
    pub fn set_color<C: Into<u8>>(&mut self, index: C) -> Result {
        let (red, green, blue) = Self::LED_COLORS[index.into().min(8) as usize];
        self.set_red(red)?;
        self.set_green(green)?;
        self.set_blue(blue)
    }
    /// Sets to brightness of the green LEDs.
    ///
    /// ## Arguments
    ///
    /// * `brightness` - How brightly the LED should be lit. 0-100(%) range with
    /// 50% default if `None` is used.
    ///
    /// ## Examples
    ///
    /// ```edition2018
    /// # #[cfg(target_arch = "arm")]
    /// # {
    /// # extern crate rust_rpi_4wd_car;
    /// use rust_rpi_4wd_car::{Hids, Result};
    /// use std::{thread::sleep, time::Duration};
    ///
    /// fn main() -> Result {
    ///     let mut hids = Hids::new()?;
    ///     let pause = Duration::from_millis(50);
    ///     println!("Varying brightness of LEDs");
    ///     for i in (0..100).step_by(10) {
    ///         hids.set_green(i)?;
    ///         sleep(pause);
    ///     }
    ///     hids.set_green(0)
    /// }
    /// # }
    /// ```
    ///
    pub fn set_green<C: Into<Option<u8>>>(&mut self, value: C) -> Result {
        let value = value.into().unwrap_or(50).min(100);
        if value != 0 {
            let dc = value as f64 * 0.01f64;
            self.led_g
                .set_pwm_frequency(Self::FREQUENCY, dc)
                .map_err(Rr4cError::Gpio)
        } else {
            self.led_g.clear_pwm().map_err(Rr4cError::Gpio)
        }
    }
    /// Sets to brightness of the red LEDs.
    ///
    /// ## Arguments
    ///
    /// * `brightness` - How brightly the LED should be lit. 0-100(%) range with
    /// 50% default if `None` is used.
    ///
    /// ## Examples
    ///
    /// ```edition2018
    /// # #[cfg(target_arch = "arm")]
    /// # {
    /// # extern crate rust_rpi_4wd_car;
    /// use rust_rpi_4wd_car::{Hids, Result};
    /// use std::{thread::sleep, time::Duration};
    ///
    /// fn main() -> Result {
    ///     let mut hids = Hids::new()?;
    ///     let pause = Duration::from_millis(50);
    ///     println!("Varying brightness of LEDs");
    ///     for i in (0..100).step_by(10) {
    ///         hids.set_red(i)?;
    ///         sleep(pause);
    ///     }
    ///     hids.set_red(0)
    /// }
    /// # }
    /// ```
    ///
    pub fn set_red<C: Into<Option<u8>>>(&mut self, value: C) -> Result {
        let value = value.into().unwrap_or(50).min(100);
        if value != 0 {
            let dc = value as f64 * 0.01f64;
            self.led_r
                .set_pwm_frequency(Self::FREQUENCY, dc)
                .map_err(Rr4cError::Gpio)
        } else {
            self.led_r.clear_pwm().map_err(Rr4cError::Gpio)
        }
    }
    /// Toggle the fan on/off.
    pub fn toggle_fan(&mut self) -> Result {
        self.fan.toggle();
        Ok(())
    }
    /// A short bleep from the buzzer.
    pub fn whistle(&mut self) {
        self.beep(None);
    }
    /// The combine buzzer and `KEY` button pin #.
    const BUZZ_KEY: u8 = 8;
    /// The fan pin #.
    const FAN: u8 = 2;
    /// An array of RGB tuples of LED brightnesses as percentages from 0-100% to
    /// form black(Off), white(On) plus each of the 3 primary and secondary
    /// colors.
    const LED_COLORS: [(u8, u8, u8); 8] = [
        (0, 0, 0),       // Off
        (100, 100, 100), // White (On)
        (100, 0, 0),     // Red
        (0, 100, 0),     // Green
        (0, 0, 100),     // Blue
        (0, 100, 100),   // Cyan
        (100, 100, 100), // Magenta
        (100, 100, 0),   // Yellow
    ];
    /// Red LEDs pin #.
    const LED_R: u8 = 22;
    /// Green LEDs pin #.
    const LED_G: u8 = 27;
    /// Blue LEDs pin #.
    const LED_B: u8 = 24;
    /// Frequency use for LED PWM in Hz.
    const FREQUENCY: f64 = 300.0;
}
