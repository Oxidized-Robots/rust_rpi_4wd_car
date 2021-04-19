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
//! Contains all the servo related components.

use crate::{Result, Rr4cError, Rr4cResult};
use embedded_hal::Pwm;
use rppal::gpio::{Gpio, OutputPin};
use std::time::Duration;

/// Allows simple control of the robot's servos alone or in unison with each other.
#[derive(Debug, PartialEq)]
pub struct Servos {
    front: Servo,
    pan: Servo,
    tilt: Servo,
}

impl Servos {
    pub fn new() -> Rr4cResult<Self> {
        let front = Servo::new(Self::FRONT)?;
        let pan = Servo::new(Self::PAN)?;
        let tilt = Servo::new_with_limits(Self::TILT, None, 2_000_000)?;
        Ok(Self { front, pan, tilt })
    }
    pub fn camera_pan_left(&mut self) -> Result {
        self.pan
            .set_position((self.pan.position() + Self::SERVO_STEP).min(180))
    }
    pub fn camera_pan_right(&mut self) -> Result {
        self.pan
            .set_position(self.pan.position().saturating_sub(Self::SERVO_STEP))
    }
    pub fn camera_tilt_down(&mut self) -> Result {
        self.tilt
            .set_position(self.tilt.position().saturating_sub(Self::SERVO_STEP))
    }
    pub fn camera_tilt_up(&mut self) -> Result {
        self.tilt
            .set_position((self.tilt.position() + Self::SERVO_STEP).min(180))
    }
    pub fn front_left(&mut self) -> Result {
        self.front
            .set_position((self.front.position() + Self::SERVO_STEP).min(180))
    }
    pub fn front_right(&mut self) -> Result {
        self.front
            .set_position(self.front.position().saturating_sub(Self::SERVO_STEP))
    }
    pub fn set_camera_pan<A: Into<Option<u8>>>(&mut self, angle: A) -> Result {
        self.pan.set_position(angle)
    }
    pub fn set_camera_tilt<A: Into<Option<u8>>>(&mut self, angle: A) -> Result {
        self.tilt.set_position(angle)
    }
    pub fn set_front<A: Into<Option<u8>>>(&mut self, angle: A) -> Result {
        self.front.set_position(angle)
    }
    pub fn servos_init(&mut self) -> Result {
        self.set_camera_pan(None)?;
        self.set_camera_tilt(None)?;
        self.set_front(None)
    }
    pub fn servos_stop(&mut self) -> Result {
        self.front.stop()?;
        self.pan.stop()?;
        self.tilt.stop()
    }
    // named servos
    const FRONT: u8 = 23;
    const PAN: u8 = 11;
    const TILT: u8 = 9;
    const SERVO_STEP: u8 = 10;
}

#[allow(dead_code)]
#[derive(Debug, PartialEq)]
pub(crate) struct Servo {
    pin: OutputPin,
    /// In degrees (°)
    angle_range: u8,
    angle_range_u64: u64,
    /// Frequency in Hz
    frequency: f64,
    /// In ~~milliseconds (ms)~~ nanoseconds (ns)
    limit_max: u64,
    /// In ~~milliseconds (ms)~~ nanoseconds (ns)
    limit_min: u64,
    /// 1 / frequency as a time duration.
    period: Duration,
    /// In ~~milliseconds (ms)~~ nanoseconds (ns)
    pulse_range: u64,
}

#[allow(dead_code)]
impl Servo {
    /// Minimal constructor with everything using defaults.
    ///
    /// ## Arguments
    /// * `pin` - BCM pin #
    pub fn new(pin: u8) -> Rr4cResult<Self> {
        let pin = Gpio::new()?.get(pin)?.into_output();
        let period = Duration::from_secs_f64(1.0 / Self::FREQUENCY);
        Ok(Self {
            pin,
            angle_range: Self::ANGLE_RANGE,
            angle_range_u64: Self::ANGLE_RANGE as u64,
            frequency: Self::FREQUENCY,
            limit_max: Self::MAX_PULSE,
            limit_min: Self::MIN_PULSE,
            period,
            pulse_range: Self::MAX_PULSE - Self::MIN_PULSE,
        })
    }
    /// Maximal constructor with no defaults.
    ///
    /// ## Arguments
    /// * `pin` - BCM pin #
    /// * `angle_range` - Maximum angle of servo movement.
    ///                   Assumes 0° for start.
    ///                   Restricted to 30-240°.
    /// * `limit_min` - Minimum servo pulse width given in nanoseconds (ns).
    ///                 Allows using servo over a reduced angular range compared
    ///                 to its `angle_range`.
    /// * `limit_max` - Maximum servo pulse width given in nanoseconds (ns).
    ///                 Allows using servo over a reduced angular range compared
    ///                 to its `angle_range`.
    /// * `frequency` - Frequency in Hz.
    pub fn new_with_kitchen_sink<AR, LN, LX, FQ>(
        pin: u8,
        angle_range: AR,
        limit_min: LN,
        limit_max: LX,
        frequency: FQ,
    ) -> Rr4cResult<Self>
    where
        AR: Into<Option<u8>>,
        LN: Into<Option<u64>>,
        LX: Into<Option<u64>>,
        FQ: Into<Option<f64>>,
    {
        let pin = Gpio::new()?.get(pin)?.into_output();
        let angle_range = angle_range
            .into()
            .unwrap_or(Self::ANGLE_RANGE)
            .max(30)
            .min(240);
        let frequency = frequency.into().unwrap_or(Self::FREQUENCY);
        let limit_min = limit_min.into().unwrap_or(Self::MIN_PULSE);
        let limit_max = limit_max.into().unwrap_or(Self::MAX_PULSE);
        let period = Duration::from_secs_f64(1.0 / frequency);
        let pulse_range = Self::MAX_PULSE - Self::MIN_PULSE;
        Ok(Self {
            pin,
            angle_range,
            angle_range_u64: angle_range as u64,
            frequency,
            limit_max,
            limit_min,
            period,
            pulse_range,
        })
    }
    /// Constructor with angle range.
    ///
    /// ## Arguments
    /// * `pin` - BCM pin #
    /// * `angle_range` - Maximum angle of servo movement.
    ///                   Assumes 0° for start.
    ///                   Restricted to 30-240°.
    pub fn new_with_angle_range<AR: Into<u8>>(pin: u8, angle_range: AR) -> Rr4cResult<Self> {
        let pin = Gpio::new()?.get(pin)?.into_output();
        let angle_range = angle_range.into().max(30).min(240);
        let period = Duration::from_secs_f64(1.0 / Self::FREQUENCY);
        Ok(Self {
            pin,
            angle_range,
            angle_range_u64: angle_range as u64,
            frequency: Self::FREQUENCY,
            limit_max: Self::MAX_PULSE,
            limit_min: Self::MIN_PULSE,
            period,
            pulse_range: Self::MAX_PULSE - Self::MIN_PULSE,
        })
    }
    /// Constructor with custom frequency.
    ///
    /// ## Arguments
    /// * `pin` - BCM pin #
    /// * `frequency` - Frequency in Hz.
    pub fn new_with_frequency<FQ: Into<f64>>(pin: u8, frequency: FQ) -> Rr4cResult<Self> {
        let pin = Gpio::new()?.get(pin)?.into_output();
        let frequency = frequency.into();
        let period = Duration::from_secs_f64(1.0 / frequency);
        Ok(Self {
            pin,
            angle_range: Self::ANGLE_RANGE,
            angle_range_u64: Self::ANGLE_RANGE as u64,
            frequency,
            limit_max: Self::MAX_PULSE,
            limit_min: Self::MIN_PULSE,
            period,
            pulse_range: Self::MAX_PULSE - Self::MIN_PULSE,
        })
    }
    /// Constructor with servo min and/or max limits.
    ///
    /// ## Arguments
    /// * `pin` - BCM pin #
    /// * `limit_min` - Minimum servo pulse width given in nanoseconds (ns).
    ///                 Allows using servo over a reduced angular range compared
    ///                 to its `angle_range`.
    /// * `limit_max` - Maximum servo pulse width given in nanoseconds (ns).
    ///                 Allows using servo over a reduced angular range compared
    ///                 to its `angle_range`.
    pub fn new_with_limits<LN, LX>(pin: u8, limit_min: LN, limit_max: LX) -> Rr4cResult<Self>
    where
        LN: Into<Option<u64>>,
        LX: Into<Option<u64>>,
    {
        let pin = Gpio::new()?.get(pin)?.into_output();
        let limit_min = limit_min.into().unwrap_or(Self::MIN_PULSE);
        let limit_max = limit_max.into().unwrap_or(Self::MAX_PULSE);
        let period = Duration::from_secs_f64(1.0 / Self::FREQUENCY);
        Ok(Self {
            pin,
            angle_range: Self::ANGLE_RANGE,
            angle_range_u64: Self::ANGLE_RANGE as u64,
            frequency: Self::FREQUENCY,
            limit_max,
            limit_min,
            period,
            pulse_range: Self::MAX_PULSE - Self::MIN_PULSE,
        })
    }
    /// Get position in integer degrees (°)
    pub fn position(&self) -> u8 {
        let dc = self.pin.get_duty(());
        let period = self.pin.get_period();
        ((period.mul_f64(dc).as_nanos() as u64 - Self::MIN_PULSE) * self.angle_range_u64
            / self.pulse_range) as u8
    }
    /// Set position from integer degrees (°)
    ///
    /// Do to the cheap servos being used, use of software PWM, and the limited
    /// need for more accurate pointing it was decided that a simple integer
    /// angle was more than enough for the application.
    ///
    /// ## Arguments
    /// * `angle` - New position angle in integer degrees (°)
    pub fn set_position<A: Into<Option<u8>>>(&mut self, angle: A) -> Result {
        let angle = angle
            .into()
            .unwrap_or(self.angle_range / 2)
            .min(self.angle_range) as u64;
        let mut pw = Self::MIN_PULSE + (angle * self.pulse_range / self.angle_range_u64);
        pw = pw.max(self.limit_min).min(self.limit_max);
        self.pin
            .set_pwm(self.period, Duration::from_nanos(pw))
            .map_err(Rr4cError::Gpio)
    }
    ///Stop (clear) active PWM
    pub fn stop(&mut self) -> Result {
        self.pin.clear_pwm().map_err(Rr4cError::Gpio)
    }
    /// Default servo angle range in degrees (°)
    const ANGLE_RANGE: u8 = 180;
    /// Default center pulse width in ~~microseconds (μs)~~ nanoseconds (ns)
    pub const CENTER_PULSE: u64 = 1_500_000;
    /// Default servo frequency in hertz
    const FREQUENCY: f64 = 50.0;
    /// Default maximum pulse width in ~~microseconds (μs)~~ nanoseconds (ns)
    pub const MAX_PULSE: u64 = 2_500_000;
    /// Default minimum pulse width in ~~microseconds (μs)~~ nanoseconds (ns)
    pub const MIN_PULSE: u64 = 500_000;
    const NANOS_PER_SEC: f64 = 1_000_000_000.0;
}
