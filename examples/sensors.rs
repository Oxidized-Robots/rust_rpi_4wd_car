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
//! Example of sensors data use in both raw and Yahboom postback format.
//!
//! Ultrasonic data is collected using the default on demand single ping per
//! call method and _not_ the active sonar option.
extern crate rust_rpi_4wd_car;

use anyhow::{Context, Result};
use rppal::system::DeviceInfo;
use rust_rpi_4wd_car::Sensors;
use std::{
    sync::atomic::{AtomicBool, Ordering},
    sync::Arc,
    thread::sleep,
    time::Duration,
};

fn main() -> Result<()> {
    println!(
        "Beginning sensors tests on {}",
        DeviceInfo::new()
            .context("Failed to get new DeviceInfo")?
            .model()
    );
    sleep(Duration::from_secs(2));
    let mut sensors = Sensors::new().context("Failed to get instance")?;
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
        test(&mut sensors);
        sleep(Duration::from_secs_f64(0.5));
    }
    println!();
    println!("Finished sensors tests");
    Ok(())
}

fn test(sensors: &mut Sensors) {
    // Raw sensor data.
    let distance = sensors.sonar_distance().unwrap_or(-1.0);
    let ir = sensors.ir_proximity();
    let ldr = sensors.ldr_tracking();
    let tracking = sensors.line_tracking();
    println!(
        "ultrasonic: {:6.2}, line: {:?}, ir: {:?}, ldr: {:?}",
        distance, tracking, ir, ldr
    );
    sleep(Duration::from_millis(34));
    // Yahboom compatible postback data.
    println!("{}", sensors.as_yb_postback());
}
