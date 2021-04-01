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
use rust_rpi_4wd_car::{Hids, Result as CarResult};
use std::{thread::sleep, time::Duration};

fn main() -> Result<()> {
    println!(
        "Beginning HIDs tests on {}",
        DeviceInfo::new()
            .context("Failed to get new DeviceInfo")?
            .model()
    );
    sleep(Duration::from_secs(2));
    let mut hids = Hids::new().context("Failed to get instance")?;
    test(&mut hids, 0.5).context("Tests failed")?;
    println!("Finished HIDs tests");
    Ok(())
}

fn test<P: Into<Option<f64>>>(hids: &mut Hids, pause: P) -> CarResult {
    let pause = pause.into().unwrap_or(0.5);
    let millis = Duration::from_secs_f64(pause.abs().min(10.0));
    println!("Press the 'KEY' button to start");
    hids.key_press();
    sleep(millis);
    println!("blow");
    hids.blow(0.5);
    sleep(millis);
    println!("whistle");
    hids.whistle();
    sleep(millis);
    hids.whistle();
    sleep(millis);
    hids.whistle();
    sleep(millis);
    println!("lights: white");
    hids.lights(100, 100, 100)?;
    sleep(millis);
    println!("lights: red");
    hids.lights(100, 0, 0)?;
    sleep(millis);
    println!("lights: yellow");
    hids.lights(100, 100, 0)?;
    sleep(millis);
    println!("lights: green");
    hids.lights(0, 100, 0)?;
    sleep(millis);
    println!("lights: cyan");
    hids.lights(0, 100, 100)?;
    sleep(millis);
    println!("lights: blue");
    hids.lights(0, 0, 100)?;
    sleep(millis);
    println!("lights: magenta");
    hids.lights(100, 0, 100)?;
    sleep(millis);
    println!("lights: off");
    hids.lights(0, 0, 0)
}
