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
use rust_rpi_4wd_car::{Motors, Result as CarResult};
use std::{
    sync::atomic::{AtomicBool, Ordering},
    sync::Arc,
    thread::sleep,
    time::Duration,
};

fn main() -> Result<()> {
    println!(
        "Beginning motor tests on {}",
        DeviceInfo::new()
            .context("Failed to get new DeviceInfo")?
            .model()
    );
    sleep(Duration::from_secs(2));
    let mut motors = Motors::new().context("Failed to get instance")?;
    // Stuff needed to nicely handle Ctrl-C from user.
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .context("Error setting Ctrl-C handler")?;
    let speed = 25;
    // Loop until Ctrl-C is received.
    while running.load(Ordering::SeqCst) {
        println!("Ctrl-C (Cmd + . for Mac OS) to stop");
        println!("speed: {}%", speed);
        test(&mut motors, speed, 1.0).context("Tests failed")?;
        sleep(Duration::from_secs_f64(0.5));
        println!();
    }
    println!("Finished motor tests");
    Ok(())
}

fn test<S, P>(motors: &mut Motors, speed: S, pause: P) -> CarResult
where
    S: Into<Option<u8>>,
    P: Into<Option<f64>>,
{
    let speed = speed.into().unwrap_or(50);
    let pause = pause.into().unwrap_or(0.5);
    let millis = Duration::from_secs_f64(pause);
    let tenth = Duration::from_millis(100);
    println!("forward");
    motors.forward(speed)?;
    sleep(millis);
    motors.brake()?;
    sleep(tenth);
    println!("back");
    motors.back(speed)?;
    sleep(millis);
    motors.brake()?;
    sleep(tenth);
    println!("left");
    motors.left(speed)?;
    sleep(millis);
    motors.brake()?;
    sleep(tenth);
    println!("back left");
    motors.back_left(speed)?;
    sleep(millis);
    motors.brake()?;
    sleep(tenth);
    println!("right");
    motors.right(speed)?;
    sleep(millis);
    motors.brake()?;
    sleep(tenth);
    println!("back right");
    motors.back_right(speed)?;
    sleep(millis);
    motors.brake()?;
    sleep(tenth);
    println!("spin left");
    motors.spin_left(speed)?;
    sleep(millis);
    motors.brake()?;
    sleep(tenth);
    println!("spin right");
    motors.spin_right(speed)?;
    sleep(millis);
    println!("brake");
    motors.brake()
}
