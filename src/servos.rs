use rppal::gpio::{Gpio, OutputPin};
use std::time::Duration;
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
use crate::{Result, Rr4cError, Rr4cResult};

#[allow(dead_code)]
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
        let tilt = Servo::new_with_limits(Self::TILT, None, 2000)?;
        Ok(Self { front, pan, tilt })
    }
    pub fn set_camera_pan<A: Into<Option<u64>>>(&mut self, angle: A) -> Result {
        self.pan.position(angle)
    }
    pub fn set_camera_tilt<A: Into<Option<u64>>>(&mut self, angle: A) -> Result {
        self.tilt.position(angle)
    }
    pub fn set_front<A: Into<Option<u64>>>(&mut self, angle: A) -> Result {
        self.front.position(angle)
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
}

#[allow(dead_code)]
#[derive(Debug, PartialEq)]
struct Servo {
    pin: OutputPin,
    /// In degrees
    angle_range: u64,
    /// Frequency in Hz
    frequency: f64,
    /// In milliseconds (ms)
    limit_max: u64,
    /// In milliseconds (ms)
    limit_min: u64,
    /// 1 / frequency as a time duration.
    period: Duration,
    /// In milliseconds (ms)
    pulse_range: u64,
}

#[allow(dead_code)]
impl Servo {
    pub fn new(pin: u8) -> Rr4cResult<Self> {
        let mut pin = Gpio::new()?.get(pin)?.into_output();
        let period = Duration::from_secs_f64(1.0 / Self::FREQUENCY);
        // Nominally all servos use 1.5ms as center of range which should be a
        // safe initial value.
        pin.set_pwm(period, Duration::from_millis(Self::CENTER_PULSE))?;
        Ok(Self {
            pin,
            angle_range: Self::ANGLE_RANGE,
            frequency: Self::FREQUENCY,
            limit_max: Self::MAX_PULSE,
            limit_min: Self::MIN_PULSE,
            period,
            pulse_range: Self::MAX_PULSE - Self::MIN_PULSE,
        })
    }
    pub fn new_with_kitchen_sink<AR, LN, LX, FQ>(
        pin: u8,
        angle_range: AR,
        limit_min: LN,
        limit_max: LX,
        frequency: FQ,
    ) -> Rr4cResult<Self>
    where
        AR: Into<Option<u64>>,
        LN: Into<Option<u64>>,
        LX: Into<Option<u64>>,
        FQ: Into<Option<f64>>,
    {
        let pin = Gpio::new()?.get(pin)?.into_output();
        let angle_range = angle_range.into().unwrap_or(Self::ANGLE_RANGE);
        let frequency = frequency.into().unwrap_or(Self::FREQUENCY);
        let limit_min = limit_min.into().unwrap_or(Self::MIN_PULSE);
        let limit_max = limit_max.into().unwrap_or(Self::MAX_PULSE);
        let period = Duration::from_secs_f64(1.0 / frequency);
        let pulse_range = Self::MAX_PULSE - Self::MIN_PULSE;
        Ok(Self {
            pin,
            angle_range,
            frequency,
            limit_max,
            limit_min,
            period,
            pulse_range,
        })
    }
    pub fn new_with_angle_range<AR: Into<u64>>(pin: u8, angle_range: AR) -> Rr4cResult<Self> {
        let pin = Gpio::new()?.get(pin)?.into_output();
        let angle_range = angle_range.into().min(360);
        let period = Duration::from_secs_f64(1.0 / Self::FREQUENCY);
        Ok(Self {
            pin,
            angle_range,
            frequency: Self::FREQUENCY,
            limit_max: Self::MAX_PULSE,
            limit_min: Self::MIN_PULSE,
            period,
            pulse_range: Self::MAX_PULSE - Self::MIN_PULSE,
        })
    }
    pub fn new_with_frequency<FQ: Into<f64>>(pin: u8, frequency: FQ) -> Rr4cResult<Self> {
        let pin = Gpio::new()?.get(pin)?.into_output();
        let frequency = frequency.into();
        let period = Duration::from_secs_f64(1.0 / frequency);
        Ok(Self {
            pin,
            angle_range: Self::ANGLE_RANGE,
            frequency,
            limit_max: Self::MAX_PULSE,
            limit_min: Self::MIN_PULSE,
            period,
            pulse_range: Self::MAX_PULSE - Self::MIN_PULSE,
        })
    }
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
            frequency: Self::FREQUENCY,
            limit_max,
            limit_min,
            period,
            pulse_range: Self::MAX_PULSE - Self::MIN_PULSE,
        })
    }
    pub fn position<A: Into<Option<u64>>>(&mut self, angle: A) -> Result {
        let angle = angle
            .into()
            .unwrap_or(self.angle_range / 2)
            .min(self.angle_range);
        let mut pw = Self::MIN_PULSE + (angle as u64 * self.pulse_range / self.angle_range);
        pw = pw.clamp(self.limit_min, self.limit_max);
        let pw = Duration::from_micros(pw);
        // eprintln!("period: {:?}, duty_cycle: {:?}", self.period, pw);
        self.pin.set_pwm(self.period, pw).map_err(Rr4cError::Gpio)
    }
    pub fn stop(&mut self) -> Result {
        self.pin.clear_pwm().map_err(Rr4cError::Gpio)
    }
    /// Default servo angle range in degrees
    const ANGLE_RANGE: u64 = 180;
    /// Default center pulse width in milliseconds (ms)
    const CENTER_PULSE: u64 = 1500;
    /// Default servo frequency in hertz
    const FREQUENCY: f64 = 50.0;
    /// Default maximum pulse width in milliseconds (ms)
    const MAX_PULSE: u64 = 2500;
    /// Default minimum pulse width in milliseconds (ms)
    const MIN_PULSE: u64 = 500;
}
