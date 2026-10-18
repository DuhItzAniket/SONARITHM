// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Lists output and input devices with their default configs (used to pick a measurement endpoint).

use cpal::traits::{DeviceTrait, HostTrait};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let host = cpal::default_host();
    for d in host.output_devices()? {
        let name = d
            .description()
            .map(|x| x.name().to_owned())
            .unwrap_or_default();
        println!("output {name}: {:?}", d.default_output_config());
    }
    for d in host.input_devices()? {
        let name = d
            .description()
            .map(|x| x.name().to_owned())
            .unwrap_or_default();
        println!("input {name}: {:?}", d.default_input_config());
    }
    Ok(())
}
