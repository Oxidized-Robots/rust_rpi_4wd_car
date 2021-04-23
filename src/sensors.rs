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
use embedded_hal::PwmPin;
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
    /// Instance of [IrProximity](IrProximity).
    ir_proximity: IrProximity,
    /// Instance of [InputPin] connected to left infrared (IR) proximity pin.
    ///
    /// [InputPin]: rppal::gpio::InputPin
    ir_left: InputPin,
    /// Instance of [InputPin] connected to right infrared (IR) proximity pin.
    ///
    /// [InputPin]: rppal::gpio::InputPin
    ir_right: InputPin,
    /// Instance of [InputPin] connected to left light dependant resister (LDR)
    /// tracking pin.
    ///
    /// [InputPin]: rppal::gpio::InputPin
    ldr_left: InputPin,
    /// Instance of [InputPin] connected to right light dependant resister (LDR)
    /// tracking pin.
    ///
    /// [InputPin]: rppal::gpio::InputPin
    ldr_right: InputPin,
    /// Instance of [Sonar](Sonar).
    sonar: Sonar,
    /// Instance of [LineTracking](LineTracking).
    tracking: LineTracking,
    /// Instance of [InputPin] connected to left line tracking input 1 pin.
    ///
    /// [InputPin]: rppal::gpio::InputPin
    track_left1: InputPin,
    /// Instance of [InputPin] connected to left line tracking input 2 pin.
    ///
    /// [InputPin]: rppal::gpio::InputPin
    track_left2: InputPin,
    /// Instance of [InputPin] connected to right line tracking input 1 pin.
    ///
    /// [InputPin]: rppal::gpio::InputPin
    track_right1: InputPin,
    /// Instance of [InputPin] connected to right line tracking input 2 pin.
    ///
    /// [InputPin]: rppal::gpio::InputPin
    track_right2: InputPin,
}

impl Sensors {
    /// Constructor which uses default values for all optional arguments.
    pub fn new() -> Rr4cResult<Self> {
        Self::new_with_temp_hum(None, None)
    }
    /// Constructor with `temperature` and `humidity` options.
    ///
    /// ## Arguments
    ///
    /// The `temperature` and `humidity` values are used to increase the
    /// accuracy of ultrasonic distance measurements.
    ///
    /// * `temperature` - Temperature in °C.
    /// A `None` value will set a default of 20°C.
    /// Temperatures are limited to between -40 and +65.5°C.
    /// * `humidity` - Relative humidity as %.
    /// A `None` value will set a default of 40%.
    pub fn new_with_temp_hum<T, H>(temperature: T, humidity: H) -> Rr4cResult<Self>
    where
        T: Into<Option<f32>>,
        H: Into<Option<f32>>,
    {
        let gpio = Gpio::new()?;
        // IR
        let (ir_left, ir_right, ir_proximity) = Sensors::ir_init(&gpio)?;
        // LDR Tracking
        let ldr_left = gpio.get(Self::LDR_LEFT)?.into_input();
        let ldr_right = gpio.get(Self::LDR_RIGHT)?.into_input();
        // line tracking
        let (track_left1, track_left2, track_right1, track_right2, tracking) =
            Sensors::line_tracking_init(&gpio)?;
        // Sonar
        let sonar = Sonar::new_with_temp_hum(temperature, humidity)?;
        Ok(Self {
            ir_proximity,
            ir_left,
            ir_right,
            ldr_left,
            ldr_right,
            sonar,
            tracking,
            track_left1,
            track_left2,
            track_right1,
            track_right2,
        })
    }
    /// Produces an Rr4c compatible postback response of sensor data.
    pub fn as_rr_postback(&mut self) -> String {
        let distance = self.sonar.distance().unwrap_or(-1.0);
        let (ir_l, ir_r) = self.ir_proximity();
        let (ldr_l, ldr_r) = self.ldr_tracking();
        let (line_l1, line_l2, line_r1, line_r2) = self.line_tracking();
        format!(
            "$RR4W,SNR{},LNF{}:{}:{}:{},IRP{}:{},LDR{}:{}#",
            distance as i16,
            line_l1 as u8,
            line_l2 as u8,
            line_r1 as u8,
            line_r2 as u8,
            ir_l as u8,
            ir_r as u8,
            ldr_l as u8,
            ldr_r as u8
        )
    }
    /// Produces an Yahboom compatible postback response of sensor data.
    pub fn as_yb_postback(&mut self) -> String {
        let distance = self.sonar.distance().unwrap_or(-1.0);
        let (ir_l, ir_r) = self.ir_proximity();
        let (ldr_l, ldr_r) = self.ldr_tracking();
        let (line_l1, line_l2, line_r1, line_r2) = self.line_tracking();
        format!(
            "$4WD,CSB{},PV8.3,GS0,LF{}{}{}{},HW{}{},GM{}{}#",
            distance as i16,
            line_l1 as u8,
            line_l2 as u8,
            line_r1 as u8,
            line_r2 as u8,
            ir_l as u8,
            ir_r as u8,
            ldr_l as u8,
            ldr_r as u8
        )
    }
    /// Used to acquire latest infrared (IR) proximity sensors data.
    pub fn ir_proximity(&self) -> (bool, bool) {
        (
            self.ir_proximity.left.load(Ordering::Acquire),
            self.ir_proximity.right.load(Ordering::Acquire),
        )
    }
    /// Used to acquire latest light dependant resister (LDR) tracking sensors
    /// data.
    pub fn ldr_tracking(&self) -> (bool, bool) {
        (
            self.ldr_left.read() == Level::High,
            self.ldr_right.read() == Level::High,
        )
    }
    /// Used to acquire latest line tracking sensors data.
    pub fn line_tracking(&self) -> (bool, bool, bool, bool) {
        (
            self.tracking.left1.load(Ordering::Acquire),
            self.tracking.left2.load(Ordering::Acquire),
            self.tracking.right1.load(Ordering::Acquire),
            self.tracking.right2.load(Ordering::Acquire),
        )
    }
    /// Enable/disable active background sonar pinging.
    ///
    /// ## Arguments
    ///
    /// * `enable` -Turns on active background sonar pinging when `true`.
    pub fn sonar_active(&mut self, enable: bool) {
        self.sonar.set_sonar_active(enable);
    }
    /// Used to acquire ultrasonic distance measurement if available.
    pub fn sonar_distance(&mut self) -> Option<f32> {
        self.sonar.distance()
    }
    /// Initialize all infrared (IR) proximity sensors related pins and data.
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
    /// Initialize all line tracking sensors related pins and data.
    fn line_tracking_init(gpio: &Gpio) -> Rr4cResult<LineInitResult> {
        let mut track_left1 = gpio.get(Sensors::LINE_LEFT_1)?.into_input();
        let mut track_left2 = gpio.get(Sensors::LINE_LEFT_2)?.into_input();
        let mut track_right1 = gpio.get(Sensors::LINE_RIGHT_1)?.into_input();
        let mut track_right2 = gpio.get(Sensors::LINE_RIGHT_2)?.into_input();
        let tracking = LineTracking::new();
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
    /// Left infrared obstacle input pin #.
    const INFRARED_LEFT: u8 = 12;
    /// Right infrared obstacle input pin #.
    const INFRARED_RIGHT: u8 = 17;
    /// Left light dependent resistor (LDR) input pin #.
    const LDR_LEFT: u8 = 7;
    /// Right light dependent resistor (LDR) input pin #.
    const LDR_RIGHT: u8 = 6;
    /// Left line tracking input 1 pin #.
    const LINE_LEFT_1: u8 = 3;
    /// Left line tracking input 2 pin #.
    const LINE_LEFT_2: u8 = 5;
    /// Right line tracking input 1 pin #.
    const LINE_RIGHT_1: u8 = 4;
    /// Right line tracking input 2 pin #.
    const LINE_RIGHT_2: u8 = 18;
}

/// Simple overwriting ring buffer used to queue ultrasonic distance readings
/// from active sonar.
#[derive(Clone, Copy, Debug)]
struct CircularQueue {
    depth: usize,
    read: usize,
    queue: [f32; 6],
    write: usize,
}

impl CircularQueue {
    pub fn new() -> Self {
        Self {
            depth: 0,
            read: 0,
            queue: [0.0; 6],
            write: 0,
        }
    }
    pub fn pop(&mut self) -> Option<f32> {
        // eprintln!("read: {}, write: {}", self.read, self.write);
        // If the reader has caught up the writer return None.
        if self.depth == 0 {
            None
        } else {
            let value = self.queue[self.read];
            self.read = self.read.saturating_add(1) % 6;
            self.depth = self.depth.saturating_sub(1);
            Some(value)
        }
    }
    pub fn push<V: Into<f32>>(&mut self, value: V) {
        self.queue[self.write] = value.into();
        let inc = self.depth.saturating_add(1).min(6);
        // If the writer is starting to lap the reader move the read forward to
        // oldest write
        if self.depth == inc {
            self.read = self.read.saturating_add(1) % 6;
        } else {
            self.depth = inc;
        }
        self.write = self.write.saturating_add(1) % 6;
    }
}

/// Holds data related to infrared (IR) proximity sensors.
#[derive(Debug)]
pub struct IrProximity {
    pub left: Arc<AtomicBool>,
    pub right: Arc<AtomicBool>,
}

impl IrProximity {
    pub fn new() -> Self {
        let left = Arc::new(AtomicBool::new(false));
        let right = Arc::new(AtomicBool::new(false));
        Self { left, right }
    }
}

/// Holds data related to line tracking sensors.
#[derive(Debug)]
pub struct LineTracking {
    pub left1: Arc<AtomicBool>,
    pub left2: Arc<AtomicBool>,
    pub right1: Arc<AtomicBool>,
    pub right2: Arc<AtomicBool>,
}

impl LineTracking {
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

/// Ultrasonic sonar device driver for Yahboom ultrasonic sensor or similar
/// devices like the HC-SR04 or HC-SR05.
#[derive(Debug)]
pub struct Sonar {
    /// Boolean used to track active sonar status.
    active_sonar: bool,
    /// Instance of [AmUltrasonic](AmUltrasonic).
    ultrasonic: AmUltrasonic,
    /// Instance of [InputPin] connected to ultrasonic echo input pin.
    ///
    /// [InputPin]: rppal::gpio::InputPin
    echo: InputPin,
    /// Instance of [OutputPin] connected to ultrasonic trigger output pin.
    ///
    /// [OutputPin]: rppal::gpio::OutputPin
    trigger: OutputPin,
}
impl Sonar {
    /// Constructor which uses default values for all optional arguments.
    pub fn new() -> Rr4cResult<Self> {
        Self::new_with_kitchen_sink(None, None, None, None)
    }
    /// Constructor with just `temperature` and `humidity` options.
    ///
    /// ## Arguments
    ///
    /// The `temperature` and `humidity` values are used to increase the
    /// accuracy of ultrasonic distance measurements.
    ///
    /// * `temperature` - Temperature in °C.
    /// A `None` value will set a default of 20°C.
    /// Temperatures are limited to between -40 and +65.5°C.
    /// * `humidity` - Relative humidity as %.
    /// A `None` value will set a default of 40%.
    pub fn new_with_temp_hum<T, H>(temperature: T, humidity: H) -> Rr4cResult<Self>
    where
        T: Into<Option<f32>>,
        H: Into<Option<f32>>,
    {
        Self::new_with_kitchen_sink(None, None, temperature, humidity)
    }
    /// Constructor with all optional arguments.
    ///
    /// ## Arguments
    ///
    /// The `temperature` and `humidity` values are used to increase the
    /// accuracy of ultrasonic distance measurements.
    ///
    /// * `echo` - Optional ultrasonic echo input pin #.
    /// * `trigger` - Optional ultrasonic trigger output pin #.
    /// * `temperature` - Temperature in °C.
    /// A `None` value will set a default of 20°C.
    /// * `humidity` - Relative humidity as %.
    /// A `None` value will set a default of 40%.
    pub fn new_with_kitchen_sink<E, R, T, H>(
        echo: E,
        trigger: R,
        temperature: T,
        humidity: H,
    ) -> Rr4cResult<Self>
    where
        E: Into<Option<u8>>,
        R: Into<Option<u8>>,
        T: Into<Option<f32>>,
        H: Into<Option<f32>>,
    {
        let gpio = Gpio::new()?;
        let mut echo = gpio.get(echo.into().unwrap_or(Self::ECHO))?.into_input();
        let mut trigger = gpio
            .get(trigger.into().unwrap_or(Self::TRIGGER))?
            .into_output();
        trigger.set_low();
        trigger.set_pwm_frequency(Self::ACTIVE_SONIC_FREQUENCY, Self::ACTIVE_SONIC_DUTY_CYCLE)?;
        let ultrasonic = Arc::new(Mutex::new(Ultrasonic::new(temperature, humidity)));
        let sense = ultrasonic.clone();
        let echo_closure = move |level| {
            let mut ultrasonic = sense.lock().expect("Someone broke the lock");
            let dur = (SystemTime::now())
                .duration_since(UNIX_EPOCH)
                .expect("Bad robot!!! No time traveling to the past!");
            match level {
                Level::Low => {
                    // Only process a falling edge when there was a leading edge.
                    if let Some(rising) = ultrasonic.rising {
                        ultrasonic.rising = None;
                        // Only process falling edge that happened after the
                        // leading edge.
                        if let Some(diff) = dur.checked_sub(rising) {
                            let distance = diff.as_secs_f32() * ultrasonic.speed_of_sound;
                            if distance > 2.0 && distance < 500.0 {
                                ultrasonic.queue.push(distance);
                            }
                        }
                    }
                }
                Level::High => {
                    ultrasonic.rising = Some(dur);
                }
            }
        };
        echo.set_async_interrupt(Both, echo_closure)?;
        Ok(Self {
            active_sonar: false,
            ultrasonic,
            echo,
            trigger,
        })
    }
    /// Sets if active sonar pinging should be used.
    ///
    /// ## Arguments
    ///
    /// * `enable` -Turns on active background sonar pinging when `true`.
    pub fn set_sonar_active(&mut self, enable: bool) {
        self.active_sonar = enable;
        if enable {
            self.trigger.enable();
        } else {
            self.trigger.disable();
        }
    }
    /// Used to acquire latest ultrasonic distance measurement if available.
    ///
    /// Polls for distance measurement in a loop with a timeout.
    pub fn distance(&mut self) -> Option<f32> {
        let timeout = (SystemTime::now()).add(Duration::from_nanos(Self::ULTRASONIC_TIMEOUT));
        let dur = Duration::from_micros(10);
        if !self.active_sonar {
            // Ping
            self.trigger.set_high();
            sleep(Duration::from_nanos(10000));
            self.trigger.set_low();
            sleep(Duration::from_nanos(2000));
        }
        while SystemTime::now() < timeout {
            // Release lock as early as possible so echo interrupt thread can
            // grab it.
            {
                let mut ultrasonic = self.ultrasonic.lock().expect("Someone broke the lock");
                if let Some(distance) = ultrasonic.queue.pop() {
                    return Some(distance);
                }
            }
            sleep(dur);
        }
        None
    }
    /// Timeout in nanoseconds (ns) ≈ 30 Hz
    pub const ULTRASONIC_TIMEOUT: u64 = 33_333_000;
    /// Ultrasonic echo input pin #.
    const ECHO: u8 = 0;
    /// Ultrasonic trigger output pin #.
    const TRIGGER: u8 = 1;
    /// Frequency for active sonic pings in Hz.
    const ACTIVE_SONIC_FREQUENCY: f64 = 30.0;
    /// PWM Duty cycle in % used for active sonic.
    const ACTIVE_SONIC_DUTY_CYCLE: f64 = 0.003;
}

/// Holds data related to ultrasonic measurements.
#[derive(Debug)]
struct Ultrasonic {
    /// Time of latest rising edge from echo pin.
    ///
    /// This is used in calculating `distance` along with the time of the
    /// falling edge.
    pub rising: Option<Duration>,
    /// Used in `distance` calculation.
    pub speed_of_sound: f32,
    /// Queue of latest available distances.
    pub queue: CircularQueue,
}

impl Ultrasonic {
    /// Constructor
    ///
    /// ## Arguments
    ///
    /// The `temperature` and `humidity` values are used to increase the
    /// accuracy of ultrasonic distance measurements.
    ///
    /// * `temperature` - Temperature in °C.
    /// A `None` value will set a default of 20°C.
    /// Temperatures are limited to between -40 and +65.5°C.
    /// * `humidity` - Relative humidity as %.
    /// A `None` value will set a default of 40%.
    pub fn new<T: Into<Option<f32>>, H: Into<Option<f32>>>(temperature: T, humidity: H) -> Self {
        let temperature = temperature.into().unwrap_or(20.0).min(65.5).max(-40.0);
        let humidity = humidity.into().unwrap_or(40.0).min(100.0).max(0.0);
        // (331.3m/s + 0.606m/°C * temperature°C + 0.0124m/% * humidity%)
        // * (100 cm/meter / 2 out and back)
        let speed_of_sound = (331.3 + 0.606 * temperature + 0.0124 * humidity) * 50.0;
        Self {
            rising: None,
            speed_of_sound,
            queue: CircularQueue::new(),
        }
    }
}

/// An `Arc` `Mutex` wrapper type for `Ultrasonic` measurement structure.
type AmUltrasonic = Arc<Mutex<Ultrasonic>>;
/// Result type from `tracking_init()` function.
type LineInitResult = (InputPin, InputPin, InputPin, InputPin, LineTracking);
