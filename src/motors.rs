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

use crate::error::{Result, Rr4cError};
use embedded_hal::Pwm;
use rppal::gpio::{Gpio, OutputPin};

pub struct Motors {
    a_in1: OutputPin,
    a_in2: OutputPin,
    a_pwm: OutputPin,
    b_in1: OutputPin,
    b_in2: OutputPin,
    b_pwm: OutputPin,
}

impl Motors {
    pub fn new() -> std::result::Result<Self, Rr4cError> {
        let mut a_in1 = Gpio::new()?.get(Self::A_IN1)?.into_output();
        let mut a_in2 = Gpio::new()?.get(Self::A_IN2)?.into_output();
        let mut a_pwm = Gpio::new()?.get(Self::A_PWM)?.into_output();
        let mut b_in1 = Gpio::new()?.get(Self::B_IN1)?.into_output();
        let mut b_in2 = Gpio::new()?.get(Self::B_IN2)?.into_output();
        let mut b_pwm = Gpio::new()?.get(Self::B_PWM)?.into_output();
        a_in1.set_high();
        a_in2.set_high();
        b_in1.set_high();
        b_in2.set_high();
        a_pwm.set_pwm_frequency(Self::FREQUENCY, 0.0)?;
        b_pwm.set_pwm_frequency(Self::FREQUENCY, 0.0)?;
        Ok(Self {
            a_in1,
            a_in2,
            a_pwm,
            b_in1,
            b_in2,
            b_pwm,
        })
    }
    pub fn back<S: Into<Option<u8>>>(&mut self, speed: S) -> Result {
        let speed = speed.into().unwrap_or(50).min(100);
        self.movement(-(speed as i8), -(speed as i8))
    }
    pub fn back_left<S: Into<Option<u8>>>(&mut self, speed: S) -> Result {
        let speed = speed.into().unwrap_or(50).min(100);
        self.movement(0, -(speed as i8))
    }
    pub fn back_right<S: Into<Option<u8>>>(&mut self, speed: S) -> Result {
        let speed = speed.into().unwrap_or(50).min(100);
        self.movement(-(speed as i8), 0)
    }
    pub fn brake(&mut self) -> Result {
        self.a_in1.set_low();
        self.a_in2.set_low();
        self.b_in1.set_low();
        self.b_in2.set_low();
        self.a_pwm.set_pwm_frequency(Self::FREQUENCY, 0.0)?;
        self.b_pwm.set_pwm_frequency(Self::FREQUENCY, 0.0)?;
        Ok(())
    }
    pub fn forward<S: Into<Option<u8>>>(&mut self, speed: S) -> Result {
        let speed = speed.into().unwrap_or(50).min(100);
        self.movement(speed as i8, speed as i8)
    }
    pub fn left<S: Into<Option<u8>>>(&mut self, speed: S) -> Result {
        let speed = speed.into().unwrap_or(50).min(100);
        self.movement(0, speed as i8)
    }
    pub fn movement<L: Into<Option<i8>>, R: Into<Option<i8>>>(
        &mut self,
        left_speed: L,
        right_speed: R,
    ) -> Result {
        let left_speed = left_speed.into().unwrap_or(50);
        let right_speed = right_speed.into().unwrap_or(50);
        let left_dc: f64;
        let right_dc: f64;
        match left_speed.is_positive() {
            true => {
                left_dc = left_speed.min(100) as f64 * 0.01;
                self.a_in1.set_high();
                self.a_in2.set_low();
            }
            false => {
                left_dc = left_speed.max(-100) as f64 * -0.01;
                self.a_in1.set_low();
                self.a_in2.set_high();
            }
        }
        match right_speed.is_positive() {
            true => {
                right_dc = right_speed.min(100) as f64 * 0.01;
                self.b_in1.set_high();
                self.b_in2.set_low();
            }
            false => {
                right_dc = right_speed.max(-100) as f64 * -0.01;
                self.b_in1.set_low();
                self.b_in2.set_high();
            }
        }
        self.a_pwm.set_pwm_frequency(Self::FREQUENCY, left_dc)?;
        self.b_pwm.set_pwm_frequency(Self::FREQUENCY, right_dc)?;
        Ok(())
    }
    pub fn right<S: Into<Option<u8>>>(&mut self, speed: S) -> Result {
        let speed = speed.into().unwrap_or(50).min(100);
        self.movement(speed as i8, 0)
    }
    pub fn speed(&self) -> (u8, u8) {
        let left = (self.a_pwm.get_duty(()) * 100.0) as u8;
        let right = (self.b_pwm.get_duty(()) * 100.0) as u8;
        (left, right)
    }
    pub fn spin_left<S: Into<Option<u8>>>(&mut self, speed: S) -> Result {
        let speed = speed.into().unwrap_or(50).min(100);
        self.movement(-(speed as i8), speed as i8)
    }
    pub fn spin_right<S: Into<Option<u8>>>(&mut self, speed: S) -> Result {
        let speed = speed.into().unwrap_or(50).min(100);
        self.movement(speed as i8, -(speed as i8))
    }
    // Left side
    const A_IN1: u8 = 20;
    const A_IN2: u8 = 21;
    const A_PWM: u8 = 16;
    // Right side
    const B_IN1: u8 = 19;
    const B_IN2: u8 = 26;
    const B_PWM: u8 = 13;
    const FREQUENCY: f64 = 3000.0; // In Hz
}
