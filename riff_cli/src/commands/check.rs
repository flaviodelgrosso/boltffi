use crate::check::{EnvironmentCheck, install_missing_targets};
use crate::error::Result;
use crate::target::RustTarget;

pub struct CheckOptions {
    pub fix: bool,
    pub apple: bool,
    pub android: bool,
}

impl Default for CheckOptions {
    fn default() -> Self {
        Self {
            fix: false,
            apple: true,
            android: true,
        }
    }
}

pub fn run_check(options: CheckOptions) -> Result<bool> {
    let mut required_targets = Vec::new();

    if options.apple {
        required_targets.extend(RustTarget::ALL_IOS.iter().cloned());
    }

    if options.android {
        required_targets.extend(RustTarget::ALL_ANDROID.iter().cloned());
    }

    let check = EnvironmentCheck::run(&required_targets);

    print_environment_status(&check, &options);

    if options.fix && check.has_missing_targets() {
        println!();
        println!("Installing missing targets...");
        install_missing_targets(&check.missing_targets)?;
        println!("Done!");
    }

    let all_good = !check.has_missing_targets()
        && (!options.apple || check.is_ready_for_apple())
        && (!options.android || check.is_ready_for_android());

    Ok(all_good)
}

fn print_environment_status(check: &EnvironmentCheck, options: &CheckOptions) {
    println!("Environment");

    match &check.rust_version {
        Some(version) => println!("  {} {}", status_icon(true), version),
        None => println!("  {} Rust not found", status_icon(false)),
    }

    println!();

    if options.apple {
        println!("Apple Targets");
        RustTarget::ALL_IOS.iter().for_each(|target| {
            let installed = check.installed_targets.iter().any(|t| t == target.triple());
            println!("  {} {}", status_icon(installed), target.triple());
        });
        println!();

        println!("Apple Tools");
        println!("  {} Xcode CLI tools", status_icon(check.tools.xcode_cli));
        println!("  {} lipo", status_icon(check.tools.lipo));
        println!("  {} xcodebuild", status_icon(check.tools.xcodebuild));
        println!();
    }

    if options.android {
        println!("Android Targets");
        RustTarget::ALL_ANDROID.iter().for_each(|target| {
            let installed = check.installed_targets.iter().any(|t| t == target.triple());
            println!("  {} {}", status_icon(installed), target.triple());
        });
        println!();

        println!("Android Tools");
        match &check.tools.android_ndk {
            Some(path) => println!("  {} Android NDK ({})", status_icon(true), path),
            None => println!("  {} Android NDK not found", status_icon(false)),
        }
        println!();
    }

    if check.has_missing_targets() {
        println!("Missing targets can be installed with:");
        check.fix_commands().iter().for_each(|cmd| {
            println!("  {}", cmd);
        });
        println!();
        println!("Or run: riff check --fix");
    }
}

fn status_icon(success: bool) -> &'static str {
    if success { "[ok]" } else { "[missing]" }
}
