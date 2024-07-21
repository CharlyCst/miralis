//! Build

use crate::artifacts::{build_target, Target};
use crate::config::read_config;
use crate::BuildArgs;

pub fn build(args: &BuildArgs) {
    let cfg = read_config(&args.config);
    let miralis = build_target(Target::Miralis, &cfg);

    if let Some(config) = &args.config {
        println!(
            "Built Miralis with config '{}', binary available at:",
            config.display()
        );
    } else {
        println!("Built Miralis, binary available at:");
    }
    println!("{}", miralis.display());
}
