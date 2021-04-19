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
//! A common set of error and result types used in the library.

use thiserror::Error;

/// Provides a shared set of error types used by the library.
#[derive(Debug, Error)]
pub enum Rr4cError {
    #[error("Was given invalid command: '{0}'")]
    BadCommand(String),
    #[error("Was given bad command value in command: '{0}'")]
    BadCommandValue(String),
    #[error("Gpio access failed")]
    Gpio(#[from] rppal::gpio::Error),
    #[error("Was given an invalid or incomplete command: '{0}'")]
    IncompleteCommand(String),
    #[error("Given unknown command: '{0}'")]
    UnknownCommand(String),
    #[error("Given unknown led command: '{0}'")]
    UnknownLedCommand(u8),
    #[error("Given unknown mode command: '{0}'")]
    UnknownModeCommand(String),
    #[error("Given unknown motor command '{0}'")]
    UnknownMotorCommand(u8),
    #[error("Given unknown motor speed command '{0}'")]
    UnknownMotorSpeedCommand(u8),
    #[error("Given unknown servo command '{0}'")]
    UnknownServoCommand(u8),
    #[error("Given unknown spin command '{0}'")]
    UnknownSpinCommand(u8),
}

/// Result type used when return value is needed from methods in library.
pub type Rr4cResult<T> = std::result::Result<T, Rr4cError>;

/// Result type used when return value is _NOT_ needed from methods in library.
pub type Result = std::result::Result<(), Rr4cError>;
