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
//! Contains higher level command and control components.

use crate::{Hids, Motors, Result, Rr4cError, Rr4cResult, Servos};
use std::thread::sleep;
use std::time::Duration;

/// A robot mode & command decoder.
///
/// Expected to be used as part of a Tcp (Web), Bluetooth, or other server and
/// client type control schema.
/// Could also be used in a CLI or file based scripting system.
#[derive(Debug)]
pub struct Decoder {
    /// Holds instance of `Hids` structure.
    hids: Hids,
    /// Used to track current LED color.
    led_color: u8,
    /// Holds the current command mode.
    mode: CarModes,
    /// Holds an instance of `Motors` structure.
    motors: Motors,
    /// Holds current default motor speed.
    motor_speed: i8,
    /// Holds a instance of `Servos` structure.
    servos: Servos,
}

impl Decoder {
    /// Constructor
    pub fn new() -> Rr4cResult<Self> {
        let mut servos = Servos::new()?;
        servos.servos_init()?;
        Ok(Self {
            hids: Hids::new()?,
            led_color: 0,
            mode: CarModes::Remote,
            motors: Motors::new()?,
            motor_speed: 25,
            servos,
        })
    }
    /// Top level command decoder.
    ///
    /// ## Arguments
    /// * `line` - String containing a single command frame that starts with a
    /// '$' and ends with a '#'.
    pub fn rr_decode<'a, L: Into<&'a str>>(&mut self, line: L) -> Result {
        let line = line.into();
        if let Some(line) = line
            .strip_prefix("$RR4W,")
            .and_then(|v| v.strip_suffix("#"))
        {
            for piece in line.split_terminator(',') {
                if piece.len() <= 3 {
                    return Err(Rr4cError::UnknownCommand(piece.to_string()));
                }
                match &piece[..=3] {
                    "CAM" => {
                        if piece.len() > 3 {
                            self.cam_decode(piece)?;
                        } else {
                            self.servos.set_camera_pan(None)?;
                            self.servos.set_camera_tilt(None)?;
                        }
                        continue;
                    }
                    "FAN" => {
                        self.fan_decode(piece)?;
                        continue;
                    }
                    "FRT" => {
                        if piece.len() > 3 {
                            self.frt_decode(piece)?;
                        } else {
                            self.servos.set_front(None)?;
                        }
                        continue;
                    }
                    "LED" => {
                        if piece.len() > 3 {
                            self.led_decode(piece)?;
                        } else {
                            self.hids.lights(0, 0, 0)?;
                        }
                        continue;
                    }
                    "MTR" => {
                        if piece.len() > 3 {
                            self.mtr_decode(piece)?;
                        } else {
                            self.motors.movement(self.motor_speed, self.motor_speed)?;
                        }
                        continue;
                    }
                    y => {
                        return Err(Rr4cError::UnknownCommand(y.to_string()));
                    }
                }
            }
            Ok(())
        } else {
            self.mode = CarModes::Remote;
            Err(Rr4cError::BadCommand(line.to_string()))
        }
    }
    /// Yahboom command decoder.
    ///
    /// ## Arguments
    /// * `line` - String containing a single command frame that starts with a
    /// '$' and ends with a '#'.
    pub fn yb_decode<'a, L: Into<&'a str>>(&mut self, line: L) -> Result {
        let line = line.into();
        if !line.starts_with('$') || !line.ends_with('#') {
            self.mode = CarModes::Remote;
            return Err(Rr4cError::BadCommand(line.to_string()));
        }
        if let Some(line) = line.strip_prefix("$4WD,").and_then(|v| v.strip_suffix("#")) {
            // Front servo
            if let Some(remains) = line.strip_prefix("PTZ") {
                let pos: u8 = remains
                    .parse()
                    .map_err(|_| Rr4cError::BadCommandValue(line.to_string()))?;
                self.servos.set_front(pos)?;
            // LEDs
            } else if let Some(remains) = line.strip_prefix("CLR") {
                let mut red: u8;
                let mut green: u8;
                let mut blue: u8;
                if let Some(idx_g) = remains.find(",CLG") {
                    red = remains[0..idx_g]
                        .parse()
                        .map_err(|_| Rr4cError::BadCommandValue(line.to_string()))?;
                    // Scale to %
                    red = 100 * red / 255;
                    if let Some(idx_b) = remains.find(",CLB") {
                        green = remains[(idx_g + 4)..idx_b]
                            .parse()
                            .map_err(|_| Rr4cError::BadCommandValue(line.to_string()))?;
                        // Scale to %
                        green = 100 * green / 255;
                        blue = remains[(idx_b + 4)..]
                            .parse()
                            .map_err(|_| Rr4cError::BadCommandValue(line.to_string()))?;
                        // Scale to %
                        blue = 100 * blue / 255;
                        return self.hids.lights(red, green, blue);
                    }
                }
                return Err(Rr4cError::BadCommand(line.to_string()));
            } else if let Some(remains) = line.strip_prefix("MODE") {
                return match remains {
                    "00" | "10" | "20" | "30" | "40" | "50" | "60" => {
                        self.motors.brake()?;
                        self.mode = CarModes::Remote;
                        self.hids.lights(100, 0, 0)?;
                        self.hids.beep(1.0);
                        self.hids.lights(0, 0, 0)
                    }
                    "11" => {
                        self.mode = CarModes::Remote;
                        self.alert_mode(None)?;
                        Ok(())
                    }
                    "21" => {
                        self.mode = CarModes::Tracking;
                        self.alert_mode(None)?;
                        self.tracking_mode()
                    }
                    "31" => {
                        self.mode = CarModes::UltrasonicAvoid;
                        self.alert_mode(None)?;
                        self.ultrasonic_avoid()
                    }
                    "41" => {
                        self.mode = CarModes::LedColors;
                        self.alert_mode(None)?;
                        self.led_colors()
                    }
                    "51" => {
                        self.mode = CarModes::LightSeeking;
                        self.alert_mode(None)?;
                        self.light_seeking()
                    }
                    "61" => {
                        self.mode = CarModes::InfraredFollow;
                        self.alert_mode(None)?;
                        self.infrared_follow()
                    }
                    r => {
                        self.motors.brake()?;
                        self.mode = CarModes::Remote;
                        self.hids.lights(100, 0, 0)?;
                        self.hids.beep(1.0);
                        self.hids.lights(0, 0, 0)?;
                        Err(Rr4cError::UnknownModeCommand(r.to_string()))
                    }
                };
            } else {
                return Err(Rr4cError::UnknownCommand(line.to_string()));
            }
        } else if let Some(line) = line.strip_prefix("$").and_then(|v| v.strip_suffix("#")) {
            // Have compound command.
            let bytes = line.as_bytes();
            // Update motor speed first so its available to use with any direction command.
            match bytes[6] {
                b'0' => {}
                b'1' => self.motor_speed = (self.motor_speed + Self::SPEED_INCREMENT).min(100),
                b'2' => self.motor_speed = (self.motor_speed - Self::SPEED_INCREMENT).max(0),
                y => return Err(Rr4cError::UnknownMotorSpeedCommand(y)),
            }
            // Check for spin or regular motor direction
            match bytes[2] {
                // Not spin
                b'0' => match bytes[1] {
                    b'0' => self.motors.brake()?,
                    b'1' => self.motors.movement(self.motor_speed, self.motor_speed)?,
                    b'2' => self.motors.movement(-self.motor_speed, -self.motor_speed)?,
                    b'3' => self.motors.movement(0, self.motor_speed)?,
                    b'4' => self.motors.movement(self.motor_speed, 0)?,
                    b'5' => self.motors.movement(0, -self.motor_speed)?, // Non Yahboom extension
                    b'6' => self.motors.movement(-self.motor_speed, 0)?, // Non Yahboom extension
                    y => return Err(Rr4cError::UnknownMotorCommand(y)),
                },
                b'1' => self.motors.movement(-self.motor_speed, self.motor_speed)?,
                b'2' => self.motors.movement(self.motor_speed, -self.motor_speed)?,
                y => return Err(Rr4cError::UnknownSpinCommand(y)),
            };
            if bytes[4] == b'1' {
                self.hids.whistle();
            };
            // Servos
            match bytes[8] {
                b'0' => {}
                b'1' => self.servos.front_left()?,
                b'2' => self.servos.front_right()?,
                b'3' => self.servos.camera_tilt_up()?,
                b'4' => self.servos.camera_tilt_down()?,
                b'5' => self.servos.set_camera_tilt(90)?,
                b'6' => self.servos.camera_pan_left()?,
                b'7' => self.servos.camera_pan_right()?,
                b'8' => self.servos.set_camera_pan(90)?,
                b'9' => self.servos.set_front(90)?, // Non Yahboom extension
                y => return Err(Rr4cError::UnknownServoCommand(y)),
            }
            // Yahboom hacky front servo reset.
            if bytes[16] == b'1' {
                self.servos.set_front(90)?;
            }
            // LEDs
            match bytes[12] {
                b'0' => {
                    self.led_color = 0;
                    self.hids.set_color(self.led_color)?;
                }
                b'1' => {
                    self.led_color += 1;
                    self.hids.set_color(self.led_color)?;
                }
                b'2' => {
                    self.led_color = 2;
                    self.hids.set_color(self.led_color)?;
                }
                b'3' => {
                    self.led_color = 3;
                    self.hids.set_color(self.led_color)?;
                }
                b'4' => {
                    self.led_color = 4;
                    self.hids.set_color(self.led_color)?;
                }
                b'5' => {
                    self.led_color = 5;
                    self.hids.set_color(self.led_color)?;
                }
                b'6' => {
                    self.led_color = 6;
                    self.hids.set_color(self.led_color)?;
                }
                b'7' => {
                    self.led_color = 7;
                    self.hids.set_color(self.led_color)?;
                }
                b'8' => {
                    self.led_color = 0;
                    self.hids.set_color(self.led_color)?;
                }
                y => return Err(Rr4cError::UnknownLedCommand(y)),
            }
            // Fan (outfire)
            if bytes[14] == b'1' {
                self.hids.toggle_fan()?;
            }
        } else {
            return Err(Rr4cError::BadCommand(line.to_string()));
        }
        Ok(())
    }
    /// Visual/audio human mode change alerter.
    ///
    /// ## Arguments
    /// * `mode` - Optional `CarModes` to use for alert.
    /// Defaults to the internally tracked mode value.
    fn alert_mode(&mut self, mode: Option<CarModes>) -> Result {
        let mode = mode.unwrap_or(self.mode);
        let length = 0.2;
        let delay = Duration::from_secs_f64(length);
        let count = match mode {
            CarModes::Remote => 1u8,
            CarModes::Tracking => 2,
            CarModes::UltrasonicAvoid => 3,
            CarModes::LedColors => 4,
            CarModes::LightSeeking => 5,
            CarModes::InfraredFollow => 6,
        };
        for i in 0..count {
            self.hids.set_color(i)?;
            self.hids.beep(length);
            self.hids.lights(0, 0, 0)?;
            sleep(delay);
        }
        Ok(())
    }
    fn infrared_follow(&self) -> Result {
        todo!()
    }
    fn led_colors(&self) -> Result {
        todo!()
    }
    fn light_seeking(&self) -> Result {
        todo!()
    }
    /// Camera command decoder.
    ///
    /// ## Arguments
    /// * `piece` - Segment of command frame to be decoded.
    //noinspection DuplicatedCode
    fn cam_decode(&mut self, piece: &str) -> Result {
        match &piece[3..4] {
            "I" => {
                self.servos.set_camera_pan(None)?;
                self.servos.set_camera_tilt(None)
            }
            "P" => {
                if piece.len() == 5 {
                    if &piece[5..6] == "L" {
                        return self.servos.camera_pan_left();
                    }
                    if &piece[5..6] == "R" {
                        return self.servos.camera_pan_right();
                    }
                }
                let angle: Option<u8>;
                if piece.len() >= 5 {
                    angle = Some(
                        piece[5..]
                            .parse()
                            .map_err(|_| Rr4cError::BadCommandValue(piece.to_string()))?,
                    );
                } else {
                    angle = None;
                }
                self.servos.set_camera_pan(angle)
            }
            "T" => {
                if piece.len() == 5 {
                    if &piece[5..6] == "D" {
                        return self.servos.camera_tilt_down();
                    }
                    if &piece[5..6] == "U" {
                        return self.servos.camera_tilt_up();
                    }
                }
                let angle: Option<u8>;
                if piece.len() >= 5 {
                    angle = Some(
                        piece[5..]
                            .parse()
                            .map_err(|_| Rr4cError::BadCommandValue(piece.to_string()))?,
                    );
                } else {
                    angle = None;
                }
                self.servos.set_camera_tilt(angle)
            }
            _ => {
                let mut angles = Vec::new();
                for v in piece[4..].split(':') {
                    angles.push(
                        v.parse::<u8>()
                            .map_err(|_| Rr4cError::BadCommandValue(piece.to_string()))?,
                    );
                }
                match angles.len() {
                    1 => {
                        self.servos.set_camera_pan(angles[0])?;
                        self.servos.set_camera_tilt(angles[0])
                    }
                    2 => {
                        self.servos.set_camera_pan(angles[0])?;
                        self.servos.set_camera_tilt(angles[1])
                    }
                    _ => Err(Rr4cError::BadCommandValue(piece.to_string())),
                }
            }
        }
    }
    /// Fan command decoder.
    ///
    /// ## Arguments
    /// * `piece` - Segment of command frame to be decoded.
    fn fan_decode(&mut self, piece: &str) -> Result {
        match &piece[3..4] {
            // Toggle Fan On/Off
            "T" => self.hids.toggle_fan(),
            // Turn Fan Off
            "0" => {
                self.hids.blow(0.01);
                Ok(())
            }
            // Turn Fan On for 10 secs.
            "1" => {
                self.hids.blow(10.0);
                Ok(())
            }
            _ => Err(Rr4cError::BadCommandValue(piece.to_string())),
        }
    }
    /// Front servo command decoder.
    ///
    /// ## Arguments
    /// * `piece` - Segment of command frame to be decoded.
    fn frt_decode(&mut self, piece: &str) -> Result {
        match &piece[3..4] {
            "I" => self.servos.set_front(None),
            "L" => self.servos.front_left(),
            "R" => self.servos.front_right(),
            _ => {
                let angle: Option<u8> = Some(
                    piece[4..]
                        .parse()
                        .map_err(|_| Rr4cError::BadCommandValue(piece.to_string()))?,
                );
                self.servos.set_front(angle)
            }
        }
    }
    /// LED command decoder.
    ///
    /// ## Arguments
    /// * `piece` - Segment of command frame to be decoded.
    //noinspection DuplicatedCode
    fn led_decode(&mut self, piece: &str) -> Result {
        match &piece[3..4] {
            "B" => {
                let brightness: Option<u8>;
                if piece.len() > 4 {
                    brightness = Some(
                        piece[5..]
                            .parse()
                            .map_err(|_| Rr4cError::BadCommandValue(piece.to_string()))?,
                    );
                } else {
                    brightness = None;
                }
                self.hids.set_blue(brightness)
            }
            "C" => {
                let index: u8;
                if piece.len() == 5 {
                    index = piece[5..]
                        .parse()
                        .map_err(|_| Rr4cError::BadCommandValue(piece.to_string()))?;
                } else {
                    return Err(Rr4cError::BadCommandValue(piece.to_string()));
                }
                self.hids.set_color(index)
            }
            "G" => {
                let brightness: Option<u8>;
                if piece.len() > 4 {
                    brightness = Some(
                        piece[5..]
                            .parse()
                            .map_err(|_| Rr4cError::BadCommandValue(piece.to_string()))?,
                    );
                } else {
                    brightness = None;
                }
                self.hids.set_green(brightness)
            }
            "R" => {
                let brightness: Option<u8>;
                if piece.len() > 4 {
                    brightness = Some(
                        piece[5..]
                            .parse()
                            .map_err(|_| Rr4cError::BadCommandValue(piece.to_string()))?,
                    );
                } else {
                    brightness = None;
                }
                self.hids.set_red(brightness)
            }
            _ => {
                let mut colors = Vec::new();
                for v in piece[4..].split(':') {
                    colors.push(
                        v.parse::<u8>()
                            .map_err(|_| Rr4cError::BadCommandValue(piece.to_string()))?,
                    );
                }
                match colors.len() {
                    1 => {
                        // Use the same brightness for all colors to have white.
                        self.hids.lights(colors[0], colors[0], colors[0])
                    }
                    3 => self.hids.lights(colors[0], colors[1], colors[2]),
                    _ => Err(Rr4cError::BadCommandValue(piece.to_string())),
                }
            }
        }
    }
    /// Motor command decoder.
    ///
    /// ## Arguments
    /// * `piece` - Segment of command frame to be decoded.
    //noinspection DuplicatedCode
    fn mtr_decode(&mut self, piece: &str) -> Result {
        match &piece[3..4] {
            // Motor Accelerate
            "A" => {
                self.motor_speed = (self.motor_speed + Self::SPEED_INCREMENT).min(100);
                let (mut left, mut right) = self.motors.speeds();
                match left.signum() {
                    1 => left = (left + Self::SPEED_INCREMENT).min(100),
                    0 => left = Self::SPEED_INCREMENT,
                    -1 => left = (left - Self::SPEED_INCREMENT).max(-100),
                    _ => unreachable!(),
                }
                match right.signum() {
                    1 => right = (right + Self::SPEED_INCREMENT).min(100),
                    -1 => right = (right - Self::SPEED_INCREMENT).max(-100),
                    0 => right = Self::SPEED_INCREMENT,
                    _ => unreachable!(),
                }
                self.motors.movement(left, right)
            }
            // Motor Decelerate
            "D" => {
                self.motor_speed =
                    (self.motor_speed - Self::SPEED_INCREMENT).max(Self::SPEED_INCREMENT);
                let (mut left, mut right) = self.motors.speeds();
                match left.signum() {
                    -1 => left = (left + Self::SPEED_INCREMENT).min(-Self::SPEED_INCREMENT),
                    0 => left = 0,
                    1 => left = (left - Self::SPEED_INCREMENT).max(Self::SPEED_INCREMENT),
                    _ => unreachable!(),
                }
                match right.signum() {
                    -1 => right = (right + Self::SPEED_INCREMENT).min(-Self::SPEED_INCREMENT),
                    1 => right = (right - Self::SPEED_INCREMENT).max(Self::SPEED_INCREMENT),
                    0 => right = 0,
                    _ => unreachable!(),
                }
                self.motors.movement(left, right)
            }
            // Motor Enable/Disable
            "E" => {
                if piece == "MTRE0" || piece == "MTRE1" {
                    self.motors.enable(piece == "MTRE1");
                    Ok(())
                } else {
                    Err(Rr4cError::BadCommandValue(piece.to_string()))
                }
            }
            // Motor Left
            "L" => {
                let speed: i8;
                if piece.len() >= 4 {
                    speed = piece[4..]
                        .parse()
                        .map_err(|_| Rr4cError::BadCommandValue(piece.to_string()))?;
                } else {
                    speed = self.motor_speed;
                }
                self.motors.movement(speed, 0)
            }
            // Motor Right
            "R" => {
                let speed: i8;
                if piece.len() >= 4 {
                    speed = piece[4..]
                        .parse()
                        .map_err(|_| Rr4cError::BadCommandValue(piece.to_string()))?;
                } else {
                    speed = self.motor_speed;
                }
                self.motors.movement(0, speed)
            }
            // Motor Spin Left/Right
            "S" => {
                if piece.len() < 5 {
                    return Err(Rr4cError::BadCommand(piece.to_string()));
                }
                let speed: i8;
                if piece.len() > 5 {
                    speed = piece[5..]
                        .parse()
                        .map_err(|_| Rr4cError::BadCommandValue(piece.to_string()))?;
                } else {
                    speed = self.motor_speed;
                }
                if &piece[4..5] == "L" {
                    self.motors.movement(-speed, speed)
                } else if &piece[4..5] == "R" {
                    self.motors.movement(speed, -speed)
                } else {
                    Err(Rr4cError::BadCommand(piece.to_string()))
                }
            }
            // Base Motor command that can do everything.
            _ => {
                let mut speeds = Vec::new();
                for v in piece[4..].split(':') {
                    speeds.push(
                        v.parse::<i8>()
                            .map_err(|_| Rr4cError::BadCommandValue(piece.to_string()))?,
                    );
                }
                match speeds.len() {
                    1 => {
                        if speeds[0] == 1 || speeds[0] == 0 {
                            self.motors.enable(speeds[0] == 1);
                            Ok(())
                        } else {
                            Err(Rr4cError::BadCommandValue(piece.to_string()))
                        }
                    }
                    2 => self.motors.movement(speeds[0], speeds[1]),
                    3 => {
                        let (left, right, enable) = (speeds[0], speeds[1], speeds[2]);
                        self.motors.movement(left, right)?;
                        if enable == 1 || enable == 0 {
                            self.motors.enable(enable == 1);
                            Ok(())
                        } else {
                            Err(Rr4cError::BadCommandValue(piece.to_string()))
                        }
                    }
                    _ => Err(Rr4cError::BadCommandValue(piece.to_string())),
                }
            }
        }
    }
    fn tracking_mode(&mut self) -> Result {
        todo!()
    }
    fn ultrasonic_avoid(&self) -> Result {
        todo!()
    }
    /// Increment value used when change motor speed in a command.
    const SPEED_INCREMENT: i8 = 10;
}

/// Used to track current robot control mode.
#[derive(Debug, Copy, Clone)]
pub(crate) enum CarModes {
    /// Car is by default in `Remote` mode if another mode has not been selected.
    Remote,
    /// Automated line tracking mode.
    Tracking,
    /// Combined ultrasonic and proximity obstacle avoidance mode.
    UltrasonicAvoid,
    LedColors,
    /// Follows a visible light source.
    LightSeeking,
    /// todo
    InfraredFollow,
}
