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
use rust_rpi_4wd_car::{Result as CarResult, Servos};
use std::{thread::sleep, time::Duration};

fn main() -> Result<()> {
    println!(
        "Beginning servo tests on {}",
        DeviceInfo::new()
            .context("Failed to get new DeviceInfo")?
            .model()
    );
    sleep(Duration::from_secs(2));
    let mut servos = Servos::new().context("Failed to get instance")?;
    test(&mut servos, 1.5).context("Tests failed")?;
    sleep(Duration::from_secs(1));
    println!("Finished servo tests");
    Ok(())
}

fn test<P: Into<Option<f64>>>(servos: &mut Servos, pause: P) -> CarResult {
    let pause = pause.into().unwrap_or(0.5);
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
    servos.servos_stop()
}
