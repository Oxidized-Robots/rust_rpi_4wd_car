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

use crate::Rr4cResult;
use rppal::gpio::{Gpio, InputPin, Level, OutputPin, Trigger::Both};
use std::ops::Add;
use std::sync::{atomic::AtomicBool, atomic::Ordering::SeqCst, Arc, Mutex};
use std::thread::sleep;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub struct Sensors {
    ir_proximity: IrProximity,
    ir_left: InputPin,
    ir_right: InputPin,
    ldr_left: InputPin,
    ldr_right: InputPin,
    speed_of_sound: f32,
    tracking: Tracking,
    track_left1: InputPin,
    track_left2: InputPin,
    track_right1: InputPin,
    track_right2: InputPin,
    ultrasonic: Arc<Mutex<Ultrasonic>>,
    ultrasonic_echo: InputPin,
    ultrasonic_trigger: OutputPin,
}

impl Sensors {
    /// Get new sensor instance.
    ///
    /// ## Arguments
    /// `temperature` - Temperature in °C.
    /// A `None` value will set a default of 20°C.
    pub fn new<T>(temperature: T) -> Rr4cResult<Self>
    where
        T: Into<Option<f32>>,
    {
        let temperature = temperature.into().unwrap_or(20.0);
        // (331.3m/s + 0.606m/°C * temperature°C) * (100 cm/meter / 2 out and back)
        let speed_of_sound = (331.3 + 0.606 * temperature) * 50.0;
        let gpio = Gpio::new()?;
        // IR
        let ir_proximity = IrProximity::new();
        let mut ir_left = gpio.get(Self::INFRARED_LEFT)?.into_input();
        let mut ir_right = gpio.get(Self::INFRARED_RIGHT)?.into_input();
        let sense = ir_proximity.left.clone();
        ir_left.set_async_interrupt(Both, move |level| {
            sense.store(level == Level::Low, SeqCst);
        })?;
        let sense = ir_proximity.right.clone();
        ir_right.set_async_interrupt(Both, move |level| {
            sense.store(level == Level::Low, SeqCst);
        })?;
        // LDR
        let ldr_left = gpio.get(Self::LDR_LEFT)?.into_input();
        let ldr_right = gpio.get(Self::LDR_RIGHT)?.into_input();
        // Tracking
        let tracking = Tracking::new();
        let mut track_left1 = gpio.get(Self::TRACK_LEFT_1)?.into_input();
        let mut track_left2 = gpio.get(Self::TRACK_LEFT_2)?.into_input();
        let mut track_right1 = gpio.get(Self::TRACK_RIGHT_1)?.into_input();
        let mut track_right2 = gpio.get(Self::TRACK_RIGHT_2)?.into_input();
        let sense = tracking.left1.clone();
        track_left1.set_async_interrupt(Both, move |level| {
            sense.store(level == Level::Low, SeqCst);
        })?;
        let sense = tracking.left2.clone();
        track_left2.set_async_interrupt(Both, move |level| {
            sense.store(level == Level::Low, SeqCst);
        })?;
        let sense = tracking.right1.clone();
        track_right1.set_async_interrupt(Both, move |level| {
            sense.store(level == Level::Low, SeqCst);
        })?;
        let sense = tracking.right2.clone();
        track_right2.set_async_interrupt(Both, move |level| {
            sense.store(level == Level::Low, SeqCst);
        })?;
        // Ultrasonic
        let ultrasonic = Arc::new(Mutex::new(Ultrasonic::new()));
        let mut ultrasonic_echo = gpio.get(Self::ULTRASONIC_ECHO)?.into_input();
        let mut ultrasonic_trigger = gpio.get(Self::ULTRASONIC_TRIGGER)?.into_output();
        ultrasonic_trigger.set_low();
        let sense = ultrasonic.clone();
        ultrasonic_echo.set_async_interrupt(Both, move |level| {
            let mut ultrasonic = sense.lock().expect("Someone broke the lock");
            let dur = (SystemTime::now())
                .duration_since(UNIX_EPOCH)
                .expect("Bad robot!!! No time traveling to the past!");
            match level {
                Level::Low => ultrasonic.falling = Some(dur),
                Level::High => {
                    ultrasonic.rising = Some(dur);
                    ultrasonic.falling = None;
                }
            }
        })?;
        Ok(Self {
            ir_proximity,
            ir_left,
            ir_right,
            ldr_left,
            ldr_right,
            speed_of_sound,
            tracking,
            track_left1,
            track_left2,
            track_right1,
            track_right2,
            ultrasonic,
            ultrasonic_echo,
            ultrasonic_trigger,
        })
    }
    pub fn ir_proximities(&self) -> (bool, bool) {
        (
            self.ir_proximity.left.load(SeqCst),
            self.ir_proximity.right.load(SeqCst),
        )
    }
    pub fn ldr(&self) -> (bool, bool) {
        (
            self.ldr_left.read() == Level::High,
            self.ldr_right.read() == Level::High,
        )
    }
    pub fn tracking(&self) -> (bool, bool, bool, bool) {
        (
            self.tracking.left1.load(SeqCst),
            self.tracking.left2.load(SeqCst),
            self.tracking.right1.load(SeqCst),
            self.tracking.right2.load(SeqCst),
        )
    }
    pub fn ultrasonic(&mut self) -> Option<f32> {
        let timeout = (SystemTime::now()).add(Duration::from_nanos(Self::ULTRASONIC_TIMEOUT));
        self.ping();
        while SystemTime::now() < timeout {
            let mut ultrasonic = self.ultrasonic.lock().expect("Someone broke the lock");
            if ultrasonic.falling.is_none() || ultrasonic.rising.is_none() {
                sleep(Duration::from_micros(1));
                continue;
            }
            let diff = ultrasonic
                .falling
                .unwrap()
                .checked_sub(ultrasonic.rising.unwrap());
            ultrasonic.falling = None;
            ultrasonic.rising = None;
            if let Some(diff) = diff {
                let distance = diff.as_secs_f32() * self.speed_of_sound;
                if distance > 2.0 && distance < 500.0 {
                    return Some(distance);
                }
                return None;
            }
        }
        None
    }
    fn ping(&mut self) {
        self.ultrasonic_trigger.set_high();
        sleep(Duration::from_nanos(10000));
        self.ultrasonic_trigger.set_low();
        sleep(Duration::from_nanos(2000));
    }
    /// Left infrared obstacle input pin
    const INFRARED_LEFT: u8 = 12;
    /// Right infrared obstacle input pin
    const INFRARED_RIGHT: u8 = 17;
    /// Left light dependent resistor (LDR) input pin
    const LDR_LEFT: u8 = 7;
    /// Right light dependent resistor (LDR) input pin
    const LDR_RIGHT: u8 = 6;
    const TRACK_LEFT_1: u8 = 3;
    const TRACK_LEFT_2: u8 = 5;
    const TRACK_RIGHT_1: u8 = 4;
    const TRACK_RIGHT_2: u8 = 18;
    const ULTRASONIC_ECHO: u8 = 0;
    /// Timeout in nanoseconds (ns) ≈ 30 Hz
    pub const ULTRASONIC_TIMEOUT: u64 = 33_333_000;
    const ULTRASONIC_TRIGGER: u8 = 1;
}

#[derive(Debug)]
pub struct IrProximity {
    pub left: Arc<AtomicBool>,
    pub right: Arc<AtomicBool>,
}

impl IrProximity {
    //noinspection DuplicatedCode
    pub fn new() -> Self {
        let left = Arc::new(AtomicBool::new(false));
        let right = Arc::new(AtomicBool::new(false));
        Self { left, right }
    }
}

#[derive(Debug)]
pub struct Tracking {
    pub left1: Arc<AtomicBool>,
    pub left2: Arc<AtomicBool>,
    pub right1: Arc<AtomicBool>,
    pub right2: Arc<AtomicBool>,
}

impl Tracking {
    //noinspection DuplicatedCode
    pub fn new() -> Self {
        let left1 = Arc::new(AtomicBool::new(false));
        let right1 = Arc::new(AtomicBool::new(false));
        let left2 = Arc::new(AtomicBool::new(false));
        let right2 = Arc::new(AtomicBool::new(false));
        Self {
            left1,
            left2,
            right1,
            right2,
        }
    }
}

#[derive(Debug)]
pub struct Ultrasonic {
    pub rising: Option<Duration>,
    pub falling: Option<Duration>,
}

impl Ultrasonic {
    pub fn new() -> Self {
        Self {
            rising: None,
            falling: None,
        }
    }
}
