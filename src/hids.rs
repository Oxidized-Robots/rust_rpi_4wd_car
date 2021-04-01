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

use crate::{Result, Rr4cResult};
use rppal::gpio::{Gpio, IoPin, Level, Mode, OutputPin, PullUpDown};
use std::thread::sleep;
use std::time::Duration;

pub struct Hids {
    buzz_key: IoPin,
    fan: OutputPin,
    led_r: OutputPin,
    led_g: OutputPin,
    led_b: OutputPin,
}

impl Hids {
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
        led_r.set_pwm_frequency(Self::FREQUENCY, 0.0)?;
        led_g.set_pwm_frequency(Self::FREQUENCY, 0.0)?;
        led_b.set_pwm_frequency(Self::FREQUENCY, 0.0)?;
        Ok(Self {
            buzz_key,
            fan,
            led_r,
            led_g,
            led_b,
        })
    }
    pub fn blow<S: Into<Option<f64>>>(&mut self, secs: S) {
        let secs = secs.into().unwrap_or(2.0);
        let wait = Duration::from_secs_f64(secs.abs().min(60.0));
        self.fan.set_low();
        sleep(wait);
        self.fan.set_high();
        sleep(Duration::from_millis(100));
    }
    pub fn key_press(&mut self) {
        let mut state = Level::High;
        let mut history = 0b01010101u8;
        self.buzz_key.set_mode(Mode::Input);
        self.buzz_key.set_pullupdown(PullUpDown::PullUp);
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
            sleep(Duration::from_millis(3));
        }
    }
    pub fn lights<R, G, B>(&mut self, red: R, green: G, blue: B) -> Result
    where
        R: Into<Option<u8>>,
        G: Into<Option<u8>>,
        B: Into<Option<u8>>,
    {
        let red = red.into().unwrap_or(50);
        let green = green.into().unwrap_or(50);
        let blue = blue.into().unwrap_or(50);
        let dc_r = red.min(100) as f64 * 0.01f64;
        let dc_g = green.min(100) as f64 * 0.01f64;
        let dc_b = blue.min(100) as f64 * 0.01f64;
        self.led_r.set_pwm_frequency(Self::FREQUENCY, dc_r)?;
        self.led_g.set_pwm_frequency(Self::FREQUENCY, dc_g)?;
        self.led_b.set_pwm_frequency(Self::FREQUENCY, dc_b)?;
        Ok(())
    }
    pub fn whistle(&mut self) {
        self.buzz_key.set_mode(Mode::Output);
        self.buzz_key.set_low();
        sleep(Duration::from_millis(100));
        self.buzz_key.set_high();
        sleep(Duration::from_millis(1));
    }
    const BUZZ_KEY: u8 = 8;
    const FAN: u8 = 2;
    // Definition of RGB module pins
    const LED_R: u8 = 22;
    const LED_G: u8 = 27;
    const LED_B: u8 = 24;
    const FREQUENCY: f64 = 300.0; // In Hz
}
