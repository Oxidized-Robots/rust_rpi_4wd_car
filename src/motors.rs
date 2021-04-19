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
//! Contains all the motor related components.

use crate::error::{Result, Rr4cError, Rr4cResult};
use embedded_hal::Pwm;
use rppal::gpio::{Gpio, OutputPin};

/// Proves a simpler interface for the robot's motors.
#[derive(Debug)]
pub struct Motors {
    /// Instance of [OutputPin] connected to left motor input 1 pin of motor
    /// driver chip.
    ///
    /// [OutputPin]: rppal::gpio::OutputPin
    a_in1: OutputPin,
    /// Instance of [OutputPin] connected to left motor input 2 pin of motor
    /// driver chip.
    ///
    /// [OutputPin]: rppal::gpio::OutputPin
    a_in2: OutputPin,
    /// Instance of [OutputPin] connected to left motor PWM pin of motor
    /// driver chip.
    ///
    /// [OutputPin]: rppal::gpio::OutputPin
    a_pwm: OutputPin,
    /// Instance of [OutputPin] connected to right motor input 1 pin of motor
    /// driver chip.
    ///
    /// [OutputPin]: rppal::gpio::OutputPin
    b_in1: OutputPin,
    /// Instance of [OutputPin] connected to right motor input 2 pin of motor
    /// driver chip.
    ///
    /// [OutputPin]: rppal::gpio::OutputPin
    b_in2: OutputPin,
    /// Instance of [OutputPin] connected to right motor PWM pin of motor
    /// driver chip.
    ///
    /// [OutputPin]: rppal::gpio::OutputPin
    b_pwm: OutputPin,
    /// Default motor speed when `None` argument is used with [`movement()`]
    /// method.
    ///
    /// [`movement()`]: Motors::movement()
    default_speed: i8,
    /// Speed scale factor
    ///
    /// Used to scale actual speed so given speeds are always 0-100%.
    speed_scale: f64,
}

impl Motors {
    /// Constructor
    pub fn new() -> Rr4cResult<Self> {
        let speed_scale = 0.01;
        let default_speed = (5000.0 * speed_scale) as i8;
        let gpio = Gpio::new()?;
        // Left
        let mut a_in1 = gpio.get(Self::A_IN1)?.into_output();
        let mut a_in2 = gpio.get(Self::A_IN2)?.into_output();
        let mut a_pwm = gpio.get(Self::A_PWM)?.into_output();
        // Right
        let mut b_in1 = gpio.get(Self::B_IN1)?.into_output();
        let mut b_in2 = gpio.get(Self::B_IN2)?.into_output();
        let mut b_pwm = gpio.get(Self::B_PWM)?.into_output();
        a_in1.set_low();
        a_in2.set_low();
        a_pwm.set_pwm_frequency(Self::FREQUENCY, 0.0)?;
        a_pwm.disable(());
        a_pwm.set_low();
        b_in1.set_low();
        b_in2.set_low();
        b_pwm.set_pwm_frequency(Self::FREQUENCY, 0.0)?;
        b_pwm.disable(());
        b_pwm.set_low();
        Ok(Self {
            a_in1,
            a_in2,
            a_pwm,
            b_in1,
            b_in2,
            b_pwm,
            default_speed,
            speed_scale,
        })
    }
    /// Stop the robot motors.
    pub fn brake(&mut self) -> Result {
        self.a_in1.set_low();
        self.a_in2.set_low();
        self.b_in1.set_low();
        self.b_in2.set_low();
        self.a_pwm.clear_pwm()?;
        self.b_pwm.clear_pwm().map_err(Rr4cError::Gpio)
    }
    /// Used to enable/disable robot moving.
    ///
    /// ## Arguments
    /// * `v` - Use `true` to enable robot moving else `false` to disable.
    pub fn enable(&mut self, v: bool) {
        if v {
            self.a_pwm.enable(());
            self.b_pwm.enable(());
        } else {
            self.a_pwm.disable(());
            self.b_pwm.disable(());
        }
    }
    /// Sets direction and speed of motors.
    ///
    /// __NOTE:__ Motors must be enabled by calling
    /// [`enable(true)`] before this command cause robot to start moving.
    ///
    /// ## Arguments
    ///
    /// Both motors use a -100(%) to +100(%) integer speed range.
    ///
    /// __NOTE:__ Speeds below 20% in either direction may be glitchy and ones
    /// below 10% are likely to cause little or no movement of the robot.
    ///
    /// * `left` - Speed and direction for left motors.
    /// * `right` - Speed and direction for right motors.
    ///
    /// ## Examples
    ///
    /// ```edition2018, no_run
    /// # #[cfg(target_arch = "arm")]
    /// # {
    /// use rust_rpi_4wd_car::{Motors, Result};
    ///
    /// fn main() -> Result {
    ///     let mut motors = Motors::new()?;
    ///     motors.enable(true);
    ///     motors.movement(-50, 50)
    /// }
    /// # }
    /// ```
    /// This will cause the robot to start spinning in place to the left.
    /// The left motors going backwards and the right motors forward.
    /// Both motors will be at ~50% speed.
    ///
    /// [`enable(true)`]: Motors::enable()
    pub fn movement<L, R>(&mut self, left: L, right: R) -> Result
    where
        L: Into<Option<i8>>,
        R: Into<Option<i8>>,
    {
        let left = left.into().unwrap_or(self.default_speed);
        let right = right.into().unwrap_or(self.default_speed);
        let left_dc: f64;
        let right_dc: f64;
        match left.signum() {
            1 => {
                left_dc = left.min(100) as f64 * self.speed_scale;
                self.a_in1.set_high();
                self.a_in2.set_low();
            }
            -1 => {
                left_dc = left.max(-100) as f64 * -self.speed_scale;
                self.a_in1.set_low();
                self.a_in2.set_high();
            }
            0 => {
                self.a_in1.set_low();
                self.a_in2.set_low();
                left_dc = 0.0;
            }
            _ => unreachable!(),
        }
        match right.signum() {
            1 => {
                right_dc = right.min(100) as f64 * self.speed_scale;
                self.b_in1.set_high();
                self.b_in2.set_low();
            }
            -1 => {
                right_dc = right.max(-100) as f64 * -self.speed_scale;
                self.b_in1.set_low();
                self.b_in2.set_high();
            }
            0 => {
                self.b_in1.set_low();
                self.b_in2.set_low();
                right_dc = 0.0;
            }
            _ => unreachable!(),
        }
        self.a_pwm.set_duty((), left_dc);
        self.b_pwm.set_duty((), right_dc);
        // self.a_pwm.set_pwm_frequency(Self::FREQUENCY, left_dc)?;
        // self.b_pwm.set_pwm_frequency(Self::FREQUENCY, right_dc)?;
        Ok(())
    }
    /// Access the current speeds of the left and right motors.
    ///
    /// __NOTE:__ Speeds will be return even when motors are _not_ actively
    /// being driven or enabled.
    pub fn speeds(&self) -> (i8, i8) {
        let left: i8;
        let right: i8;
        if self.a_in1.is_set_high() {
            left = (self.a_pwm.get_duty(()) * self.speed_scale.recip()) as i8;
        } else if self.a_in2.is_set_low() {
            left = 0;
        } else {
            left = (self.a_pwm.get_duty(()) * -self.speed_scale.recip()) as i8;
        }
        if self.b_in1.is_set_high() {
            right = (self.a_pwm.get_duty(()) * self.speed_scale.recip()) as i8;
        } else if self.b_in2.is_set_low() {
            right = 0;
        } else {
            right = (self.a_pwm.get_duty(()) * -self.speed_scale.recip()) as i8;
        }
        (left, right)
    }
    /// Left motor input 1 pin #.
    const A_IN1: u8 = 20;
    /// Left motor input 2 pin #.
    const A_IN2: u8 = 21;
    /// Left motor PWM pin #.
    const A_PWM: u8 = 16;
    /// Right motor input 1 pin #.
    const B_IN1: u8 = 19;
    /// Right motor input 2 pin #.
    const B_IN2: u8 = 26;
    /// Right motor PWM pin #.
    const B_PWM: u8 = 13;
    /// Frequency use for motor PWM in Hz.
    const FREQUENCY: f64 = 3000.0;
}
