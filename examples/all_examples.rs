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
extern crate rust_rpi_4wd_car;

use anyhow::{Context, Result};
use rppal::system::DeviceInfo;
use rust_rpi_4wd_car::{Hids, Motors, Sensors, Servos};
use std::{
    sync::atomic::{AtomicBool, Ordering},
    sync::Arc,
    thread::sleep,
    time::Duration,
};

fn main() -> Result<()> {
    println!(
        "Beginning all examples on {}",
        DeviceInfo::new()
            .context("Failed to get new DeviceInfo")?
            .model()
    );
    let mut hids = Hids::new().context("Failed to get hids instance")?;
    let mut motors = Motors::new().context("Failed to get motors instance")?;
    let mut sensors = Sensors::new(None).context("Failed to get sensors instance")?;
    let mut servos = Servos::new().context("Failed to get servos instance")?;
    // Stuff needed to nicely handle Ctrl-C from user.
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .context("Error setting Ctrl-C handler")?;
    println!("Ctrl-C (Cmd + . for Mac OS) to stop");
    // Loop until Ctrl-C is received.
    while running.load(Ordering::SeqCst) {
        hids_test(&mut hids, 0.5).context("Hids tests failed")?;
        if !running.load(Ordering::SeqCst) {
            break;
        }
        sensors_test(&mut sensors).context("Sensors tests failed")?;
        if !running.load(Ordering::SeqCst) {
            break;
        }
        servos_test(&mut servos, None).context("Servo tests failed")?;
        if !running.load(Ordering::SeqCst) {
            break;
        }
        motors_test(&mut motors, None, None).context("Motors tests failed")?;
        sleep(Duration::from_secs_f64(2.0));
    }
    println!();
    println!("Finished all examples");
    Ok(())
}

fn hids_test<P: Into<Option<f64>>>(hids: &mut Hids, pause: P) -> Result<()> {
    println!(
        "Beginning HIDs tests on {}",
        DeviceInfo::new()
            .context("Failed to get new DeviceInfo")?
            .model()
    );
    let pause = pause.into().unwrap_or(0.5);
    let millis = Duration::from_secs_f64(pause.abs().min(10.0));
    println!("Press the 'KEY' button to start");
    hids.key_press();
    sleep(millis);
    println!("blow");
    hids.blow(1.0);
    sleep(millis);
    println!("whistle");
    hids.whistle();
    sleep(millis);
    hids.whistle();
    sleep(millis);
    hids.whistle();
    sleep(millis);
    println!("lights: white");
    hids.set_color(1)?;
    sleep(millis);
    println!("lights: red");
    hids.set_color(2)?;
    sleep(millis);
    println!("lights: yellow");
    hids.set_color(7)?;
    sleep(millis);
    println!("lights: green");
    hids.set_color(3)?;
    sleep(millis);
    println!("lights: cyan");
    hids.set_color(5)?;
    sleep(millis);
    println!("lights: blue");
    hids.set_color(4)?;
    sleep(millis);
    println!("lights: magenta");
    hids.set_color(6)?;
    sleep(millis);
    println!("lights: off");
    hids.set_color(0)?;
    println!("Finished HIDs tests");
    Ok(())
}
fn motors_test<S, P>(motors: &mut Motors, speed: S, pause: P) -> Result<()>
where
    S: Into<Option<u8>>,
    P: Into<Option<f64>>,
{
    println!(
        "Beginning motor tests on {}",
        DeviceInfo::new()
            .context("Failed to get new DeviceInfo")?
            .model()
    );
    let speed = speed.into().unwrap_or(25) as i8;
    let pause = pause.into().unwrap_or(0.5);
    let millis = Duration::from_secs_f64(pause);
    println!("enable motors");
    motors.enable(true);
    sleep(millis);
    println!("forward");
    motors.movement(speed, speed)?;
    sleep(millis);
    println!("back");
    motors.movement(-speed, -speed)?;
    sleep(millis);
    println!("left");
    motors.movement(0, speed)?;
    sleep(millis);
    println!("back left");
    motors.movement(0, -speed)?;
    sleep(millis);
    println!("right");
    motors.movement(speed, 0)?;
    sleep(millis);
    println!("back right");
    motors.movement(-speed, 0)?;
    sleep(millis);
    println!("Forward curve left");
    motors.movement(speed as i8 - 10, speed as i8 + 10)?;
    sleep(millis);
    println!("Backward curve left");
    motors.movement(-(speed as i8 - 10), -(speed as i8 + 10))?;
    sleep(millis);
    println!("Forward curve right");
    motors.movement(speed as i8 + 10, speed as i8 - 10)?;
    sleep(millis);
    println!("Backward curve right");
    motors.movement(-(speed as i8 + 10), -(speed as i8 - 10))?;
    sleep(millis);
    println!("spin left");
    motors.movement(-speed, speed)?;
    sleep(millis);
    println!("spin right");
    motors.movement(speed, -speed)?;
    sleep(millis);
    println!("brake");
    motors.brake()?;
    println!("disabled motors");
    motors.enable(false);
    sleep(millis);
    println!("forward");
    motors.movement(speed, speed)?;
    sleep(millis);
    println!("back");
    motors.movement(-speed, -speed)?;
    sleep(millis);
    println!("Finished motor tests");
    Ok(())
}
fn sensors_test(sensors: &mut Sensors) -> Result<()> {
    println!(
        "Beginning sensors tests on {}",
        DeviceInfo::new()
            .context("Failed to get new DeviceInfo")?
            .model()
    );
    for _ in [0..5].iter() {
        let distance = sensors.ultrasonic().unwrap_or(-1.0);
        let ir = sensors.ir_proximities();
        let ldr = sensors.ldr();
        let tracking = sensors.tracking();
        println!(
            "ultrasonic: {:6.2}, tracking: {:?}, ir: {:?}, ldr: {:?}",
            distance, tracking, ir, ldr
        );
    }
    println!("Finished sensors tests");
    Ok(())
}
//noinspection DuplicatedCode
fn servos_test<P: Into<Option<f64>>>(servos: &mut Servos, pause: P) -> Result<()> {
    println!(
        "Beginning servo tests on {}",
        DeviceInfo::new()
            .context("Failed to get new DeviceInfo")?
            .model()
    );
    let pause = pause.into().unwrap_or(1.0);
    let millis = Duration::from_secs_f64(pause.abs().min(10.0));
    println!("Initialize");
    servos.servos_init()?;
    sleep(millis);
    println!("front 135 degrees (+45)");
    servos.set_front(135)?;
    sleep(millis);
    println!("front 90 degrees (center)");
    servos.set_front(90)?;
    sleep(millis);
    println!("front 45 degrees (-45)");
    servos.set_front(45)?;
    sleep(millis);
    println!("front 90 degrees (center)");
    servos.set_front(90)?;
    sleep(millis);
    println!("camera pan 135 degrees (+45)");
    servos.set_camera_pan(135)?;
    sleep(millis);
    println!("camera pan 90 degrees (center)");
    servos.set_camera_pan(90)?;
    sleep(millis);
    println!("camera pan 45 degrees (-45)");
    servos.set_camera_pan(45)?;
    sleep(millis);
    println!("camera pan 90 degrees (center)");
    servos.set_camera_pan(90)?;
    sleep(millis);
    println!("camera tilt 135 degrees (+45)");
    servos.set_camera_tilt(135)?;
    sleep(millis);
    println!("camera tilt 90 degrees (center)");
    servos.set_camera_tilt(90)?;
    sleep(millis);
    println!("camera tilt 45 degrees (-45)");
    servos.set_camera_tilt(45)?;
    sleep(millis);
    println!("camera tilt 90 degrees (center)");
    servos.set_camera_tilt(90)?;
    sleep(millis);
    println!("All servos");
    for angle in (0..=180).step_by(30) {
        println!("Angle: {} degrees", angle);
        servos.set_front(angle)?;
        servos.set_camera_pan(angle)?;
        servos.set_camera_tilt(angle)?;
        sleep(millis);
    }
    for angle in (0..=150).rev().step_by(30) {
        println!("Angle: {} degrees", angle);
        servos.set_front(angle)?;
        servos.set_camera_pan(angle)?;
        servos.set_camera_tilt(angle)?;
        sleep(millis);
    }
    println!("Reset");
    servos.servos_init()?;
    sleep(millis);
    println!("Camera hide");
    servos.set_camera_tilt(15)?;
    sleep(millis);
    println!("Stop");
    servos.servos_stop()?;
    println!("Finished servo tests");
    Ok(())
}
