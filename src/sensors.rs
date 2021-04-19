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
//! Contains all sensor related components.

use crate::Rr4cResult;
use rppal::gpio::{Gpio, InputPin, Level, OutputPin, Trigger::Both};
use std::{
    ops::Add,
    sync::{atomic::AtomicBool, atomic::Ordering, Arc, Mutex},
    thread::sleep,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

/// Simplifies working with robot's ultrasonic, tracking, and proximity sensors.
#[derive(Debug)]
pub struct Sensors {
    ir_proximity: IrProximity,
    ir_left: InputPin,
    ir_right: InputPin,
    ldr_left: InputPin,
    ldr_right: InputPin,
    tracking: Tracking,
    track_left1: InputPin,
    track_left2: InputPin,
    track_right1: InputPin,
    track_right2: InputPin,
    ultrasonic: AmUltrasonic,
    ultrasonic_echo: InputPin,
    ultrasonic_trigger: OutputPin,
}

impl Sensors {
    /// Get new sensor instance.
    ///
    /// ## Arguments
    /// * `temperature` - Temperature in °C.
    /// A `None` value will set a default of 20°C.
    /// * `humidity` - Relative humidity as %.
    /// A `None` value will set a default of 40%.
    pub fn new<T: Into<Option<f32>>, H: Into<Option<f32>>>(
        temperature: T,
        humidity: H,
    ) -> Rr4cResult<Self> {
        let gpio = Gpio::new()?;
        // IR
        let (ir_left, ir_right, ir_proximity) = Sensors::ir_init(&gpio)?;
        // LDR
        let ldr_left = gpio.get(Self::LDR_LEFT)?.into_input();
        let ldr_right = gpio.get(Self::LDR_RIGHT)?.into_input();
        // Tracking
        let (track_left1, track_left2, track_right1, track_right2, tracking) =
            Sensors::tracking_init(&gpio)?;
        // Ultrasonic
        let (ultrasonic, ultrasonic_echo, ultrasonic_trigger) =
            Sensors::ultrasonic_init(&gpio, temperature, humidity)?;
        Ok(Self {
            ir_proximity,
            ir_left,
            ir_right,
            ldr_left,
            ldr_right,
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
            self.ir_proximity.left.load(Ordering::Acquire),
            self.ir_proximity.right.load(Ordering::Acquire),
        )
    }
    pub fn ldr(&self) -> (bool, bool) {
        (
            self.ldr_left.read() == Level::High,
            self.ldr_right.read() == Level::High,
        )
    }
    pub fn as_yb_postback(&mut self) -> String {
        let mut ultrasonics = Vec::new();
        let dur = Duration::from_millis(30);
        for _ in 0..5 {
            let mut raw: Option<f32> = self.ultrasonic();
            while raw.is_none() {
                sleep(dur);
                raw = self.ultrasonic();
            }
            ultrasonics.push((raw.unwrap() * 100.0) as u32);
        }
        ultrasonics.sort_unstable();
        let distance = (ultrasonics[1] + ultrasonics[2] + ultrasonics[3]) / 300;
        let (ir_l, ir_r) = self.ir_proximities();
        let (ldr_l, ldr_r) = self.ldr();
        let (trk_l1, trk_l2, trk_r1, trk_r2) = self.tracking();
        format!(
            "$4WD,CSB{},PV8.3,GS0,LF{}{}{}{},HW{}{},GM{}{}#",
            distance,
            trk_l1 as u8,
            trk_l2 as u8,
            trk_r1 as u8,
            trk_r2 as u8,
            ir_l as u8,
            ir_r as u8,
            ldr_l as u8,
            ldr_r as u8
        )
    }
    pub fn tracking(&self) -> (bool, bool, bool, bool) {
        (
            self.tracking.left1.load(Ordering::Acquire),
            self.tracking.left2.load(Ordering::Acquire),
            self.tracking.right1.load(Ordering::Acquire),
            self.tracking.right2.load(Ordering::Acquire),
        )
    }
    pub fn ultrasonic(&mut self) -> Option<f32> {
        let timeout = (SystemTime::now()).add(Duration::from_nanos(Self::ULTRASONIC_TIMEOUT));
        self.ping();
        // let mut falling: Option<Duration> = None;
        // let mut rising: Option<Duration> = None;
        while SystemTime::now() < timeout {
            // Release lock as early as possible so interrupt thread can grab it.
            {
                let mut ultrasonic = self.ultrasonic.lock().expect("Someone broke the lock");
                if let Some(distance) = ultrasonic.distance {
                    ultrasonic.distance = None;
                    return Some(distance);
                }
                // if ultrasonic.rising.is_some() && ultrasonic.distance.is_some() {
                //     rising = ultrasonic.rising;
                //     falling = ultrasonic.distance;
                //     ultrasonic.rising = None;
                //     ultrasonic.distance = None;
                //     break;
                // }
            }
            sleep(Duration::from_micros(1));
        }
        // if let Some(rising) = rising {
        //     if let Some(falling) = falling {
        //         if let Some(diff) = falling.checked_sub(rising) {
        //             let distance = diff.as_secs_f32() * self.speed_of_sound;
        //             if distance > 2.0 && distance < 500.0 {
        //                 return Some(distance);
        //             }
        //         }
        //     }
        // }
        None
    }
    fn ir_init(gpio: &Gpio) -> Rr4cResult<(InputPin, InputPin, IrProximity)> {
        let mut ir_left = gpio.get(Self::INFRARED_LEFT)?.into_input();
        let mut ir_right = gpio.get(Self::INFRARED_RIGHT)?.into_input();
        let ir_proximity = IrProximity::new();
        let sense = ir_proximity.left.clone();
        sense.store(ir_left.is_low(), Ordering::SeqCst);
        ir_left.set_async_interrupt(Both, move |level| {
            sense.store(level == Level::Low, Ordering::Release);
        })?;
        let sense = ir_proximity.right.clone();
        sense.store(ir_right.is_low(), Ordering::SeqCst);
        ir_right.set_async_interrupt(Both, move |level| {
            sense.store(level == Level::Low, Ordering::Release);
        })?;
        Ok((ir_left, ir_right, ir_proximity))
    }
    fn ping(&mut self) {
        self.ultrasonic_trigger.set_high();
        sleep(Duration::from_nanos(10000));
        self.ultrasonic_trigger.set_low();
        sleep(Duration::from_nanos(2000));
    }
    fn tracking_init(gpio: &Gpio) -> Rr4cResult<TrackingResult> {
        let mut track_left1 = gpio.get(Sensors::TRACK_LEFT_1)?.into_input();
        let mut track_left2 = gpio.get(Sensors::TRACK_LEFT_2)?.into_input();
        let mut track_right1 = gpio.get(Sensors::TRACK_RIGHT_1)?.into_input();
        let mut track_right2 = gpio.get(Sensors::TRACK_RIGHT_2)?.into_input();
        let tracking = Tracking::new();
        let sense = tracking.left1.clone();
        sense.store(track_left1.is_low(), Ordering::SeqCst);
        track_left1.set_async_interrupt(Both, move |level| {
            sense.store(level == Level::Low, Ordering::Release);
        })?;
        let sense = tracking.left2.clone();
        sense.store(track_left2.is_low(), Ordering::SeqCst);
        track_left2.set_async_interrupt(Both, move |level| {
            sense.store(level == Level::Low, Ordering::Release);
        })?;
        let sense = tracking.right1.clone();
        sense.store(track_right1.is_low(), Ordering::SeqCst);
        track_right1.set_async_interrupt(Both, move |level| {
            sense.store(level == Level::Low, Ordering::Release);
        })?;
        let sense = tracking.right2.clone();
        sense.store(track_right2.is_low(), Ordering::SeqCst);
        track_right2.set_async_interrupt(Both, move |level| {
            sense.store(level == Level::Low, Ordering::Release);
        })?;
        Ok((
            track_left1,
            track_left2,
            track_right1,
            track_right2,
            tracking,
        ))
    }
    fn ultrasonic_init<T, H>(
        gpio: &Gpio,
        temperature: T,
        humidity: H,
    ) -> Rr4cResult<(AmUltrasonic, InputPin, OutputPin)>
    where
        T: Into<Option<f32>>,
        H: Into<Option<f32>>,
    {
        let ultrasonic = Arc::new(Mutex::new(Ultrasonic::new(temperature, humidity)));
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
                Level::Low => {
                    if let Some(rising) = ultrasonic.rising {
                        ultrasonic.rising = None;
                        if let Some(diff) = dur.checked_sub(rising) {
                            let distance = diff.as_secs_f32() * ultrasonic.speed_of_sound;
                            if distance > 2.0 && distance < 500.0 {
                                ultrasonic.distance = Some(distance);
                            }
                        }
                    }
                }
                Level::High => {
                    ultrasonic.rising = Some(dur);
                    ultrasonic.distance = None;
                }
            }
        })?;
        Ok((ultrasonic, ultrasonic_echo, ultrasonic_trigger))
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
    pub distance: Option<f32>,
    pub rising: Option<Duration>,
    pub speed_of_sound: f32,
}

impl Ultrasonic {
    pub fn new<T: Into<Option<f32>>, H: Into<Option<f32>>>(temperature: T, humidity: H) -> Self {
        let temperature = temperature.into().unwrap_or(20.0).min(65.5).max(-40.0);
        let humidity = humidity.into().unwrap_or(40.0).min(100.0).max(0.0);
        // (331.3m/s + 0.606m/°C * temperature°C + 0.0124m/% * humidity%)
        // * (100 cm/meter / 2 out and back)
        let speed_of_sound = (331.3 + 0.606 * temperature + 0.0124 * humidity) * 50.0;
        Self {
            distance: None,
            rising: None,
            speed_of_sound,
        }
    }
}

type AmUltrasonic = Arc<Mutex<Ultrasonic>>;
type TrackingResult = (InputPin, InputPin, InputPin, InputPin, Tracking);
