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
    let pause = 1.0;
    // Loop until Ctrl-C is received.
    while running.load(Ordering::SeqCst) {
        println!("Ctrl-C (Cmd + . for Mac OS) to stop");
        println!("speed: {}%", speed);
        test(&mut motors, speed, pause).context("Tests failed")?;
        sleep(Duration::from_secs_f64(pause));
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
    let speed = speed.into().unwrap_or(25).min(75) as i8;
    let difference = speed - speed / 4;
    let pause = pause.into().unwrap_or(1.0);
    let millis = Duration::from_secs_f64(pause);
    // Short movement pause to help protect motors from quick direction changes.
    let m_pause = Duration::from_millis(100);
    println!("movement enabled: forward");
    motors.movement(speed, speed)?;
    motors.enable(true);
    sleep(millis);
    motors.movement(0, 0)?;
    sleep(m_pause);
    println!("movement enabled: back");
    motors.movement(-speed, -speed)?;
    sleep(millis);
    motors.movement(0, 0)?;
    sleep(m_pause);
    println!("movement enabled: left");
    motors.movement(0, speed)?;
    sleep(millis);
    motors.movement(0, 0)?;
    sleep(m_pause);
    println!("movement enabled: back left");
    motors.movement(0, -speed)?;
    sleep(millis);
    println!("movement enabled: right");
    motors.movement(speed, 0)?;
    sleep(millis);
    motors.movement(0, 0)?;
    sleep(m_pause);
    println!("movement enabled: back right");
    motors.movement(-speed, 0)?;
    sleep(millis);
    motors.movement(0, 0)?;
    sleep(m_pause);
    println!("movement enabled: forward curve left");
    motors.movement(
        speed.saturating_sub(difference),
        speed.saturating_add(difference),
    )?;
    sleep(millis);
    motors.movement(0, 0)?;
    sleep(m_pause);
    println!("movement enabled: backward curve left");
    motors.movement(
        -(speed.saturating_sub(difference)),
        -(speed.saturating_add(difference)),
    )?;
    sleep(millis);
    motors.movement(0, 0)?;
    sleep(m_pause);
    println!("movement enabled: forward curve right");
    motors.movement(
        speed.saturating_add(difference),
        speed.saturating_sub(difference),
    )?;
    sleep(millis);
    motors.movement(0, 0)?;
    sleep(m_pause);
    println!("movement enabled: backward curve right");
    motors.movement(
        -(speed.saturating_add(difference)),
        -(speed.saturating_sub(difference)),
    )?;
    sleep(millis);
    motors.movement(0, 0)?;
    sleep(m_pause);
    println!("movement enabled: spin left");
    motors.movement(-speed, speed)?;
    sleep(millis);
    motors.movement(0, 0)?;
    sleep(m_pause);
    println!("movement enabled: spin right");
    motors.movement(speed, -speed)?;
    sleep(millis);
    println!("movement enabled: brake");
    motors.brake()?;
    sleep(millis);
    // motors.enable(false);
    println!("movement disabled: forward");
    motors.movement(speed, speed)?;
    sleep(millis);
    println!("movement disabled: back");
    motors.movement(-speed, -speed)?;
    sleep(millis);
    Ok(())
}
