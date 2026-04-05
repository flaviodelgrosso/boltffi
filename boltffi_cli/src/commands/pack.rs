use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

use crate::build::{
    BuildOptions, Builder, CargoBuildProfile, OutputCallback, all_successful, failed_targets,
    resolve_build_profile,
};
use crate::commands::generate::{GenerateOptions, GenerateTarget, run_generate_with_output};
use crate::config::{
    Config, SpmDistribution, SpmLayout, Target, WasmNpmTarget, WasmOptimizeLevel,
    WasmOptimizeOnMissing, WasmProfile,
};
use crate::error::{CliError, Result};
use crate::pack::{AndroidPackager, SpmPackageGenerator, XcframeworkBuilder, compute_checksum};
use crate::reporter::{Reporter, Step};
use crate::target::{BuiltLibrary, JavaHostTarget, Platform, RustTarget};

pub enum PackCommand {
    All(PackAllOptions),
    Apple(PackAppleOptions),
    Android(PackAndroidOptions),
    Wasm(PackWasmOptions),
    Java(PackJavaOptions),
}

pub struct PackAllOptions {
    pub release: bool,
    pub regenerate: bool,
    pub no_build: bool,
    pub experimental: bool,
    pub cargo_args: Vec<String>,
}

pub struct PackAppleOptions {
    pub release: bool,
    pub version: Option<String>,
    pub regenerate: bool,
    pub no_build: bool,
    pub spm_only: bool,
    pub xcframework_only: bool,
    pub layout: Option<SpmLayout>,
    pub cargo_args: Vec<String>,
}

pub struct PackAndroidOptions {
    pub release: bool,
    pub regenerate: bool,
    pub no_build: bool,
    pub cargo_args: Vec<String>,
}

pub struct PackWasmOptions {
    pub release: bool,
    pub regenerate: bool,
    pub no_build: bool,
    pub cargo_args: Vec<String>,
}

pub struct PackJavaOptions {
    pub release: bool,
    pub regenerate: bool,
    pub no_build: bool,
    pub experimental: bool,
    pub cargo_args: Vec<String>,
}

struct JvmBuildArtifacts {
    native_static_libraries: Vec<String>,
    native_link_search_paths: Vec<String>,
    static_library_filename: Option<String>,
}

struct NativeLinkMetadata {
    native_static_libraries: Vec<String>,
    native_link_search_paths: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct JvmCrateOutputs {
    builds_staticlib: bool,
    builds_cdylib: bool,
}

enum JvmNativeLinkInput {
    Staticlib(PathBuf),
    Cdylib(PathBuf),
}

impl JvmNativeLinkInput {
    fn path(&self) -> &Path {
        match self {
            Self::Staticlib(path) | Self::Cdylib(path) => path,
        }
    }
}

pub fn run_pack(config: &Config, command: PackCommand, reporter: &Reporter) -> Result<()> {
    match command {
        PackCommand::All(options) => pack_all(config, options, reporter),
        PackCommand::Apple(options) => pack_apple(config, options, reporter),
        PackCommand::Android(options) => pack_android(config, options, reporter),
        PackCommand::Wasm(options) => pack_wasm(config, options, reporter),
        PackCommand::Java(options) => pack_java(config, options, reporter),
    }
}

fn pack_all(config: &Config, options: PackAllOptions, reporter: &Reporter) -> Result<()> {
    ensure_java_no_build_supported(config, options.no_build, options.experimental, "pack all")?;

    let mut packed_any = false;

    if config.is_apple_enabled() {
        pack_apple(
            config,
            PackAppleOptions {
                release: options.release,
                version: None,
                regenerate: options.regenerate,
                no_build: options.no_build,
                spm_only: false,
                xcframework_only: false,
                layout: None,
                cargo_args: options.cargo_args.clone(),
            },
            reporter,
        )?;
        packed_any = true;
    }

    if config.is_android_enabled() {
        pack_android(
            config,
            PackAndroidOptions {
                release: options.release,
                regenerate: options.regenerate,
                no_build: options.no_build,
                cargo_args: options.cargo_args.clone(),
            },
            reporter,
        )?;
        packed_any = true;
    }

    if config.is_wasm_enabled() {
        pack_wasm(
            config,
            PackWasmOptions {
                release: options.release,
                regenerate: options.regenerate,
                no_build: options.no_build,
                cargo_args: options.cargo_args.clone(),
            },
            reporter,
        )?;
        packed_any = true;
    }

    if config.should_process(Target::Java, options.experimental) {
        pack_java(
            config,
            PackJavaOptions {
                release: options.release,
                regenerate: options.regenerate,
                no_build: options.no_build,
                experimental: options.experimental,
                cargo_args: options.cargo_args.clone(),
            },
            reporter,
        )?;
        packed_any = true;
    }

    if !packed_any {
        reporter.warning("no targets enabled in boltffi.toml");
    }

    reporter.finish();
    Ok(())
}

fn pack_apple(config: &Config, options: PackAppleOptions, reporter: &Reporter) -> Result<()> {
    if !config.is_apple_enabled() {
        return Err(CliError::CommandFailed {
            command: "targets.apple.enabled = false".to_string(),
            status: None,
        });
    }

    reporter.section("🍎", "Packing Apple");

    if !config.apple_include_macos() {
        reporter.warning("macOS excluded (targets.apple.include_macos = false)");
    }

    if options.spm_only && options.xcframework_only {
        return Err(CliError::CommandFailed {
            command: "cannot combine --spm-only and --xcframework-only".to_string(),
            status: None,
        });
    }

    let build_cargo_args = resolve_build_cargo_args(config, &options.cargo_args);
    let build_profile = resolve_build_profile(options.release, &build_cargo_args);
    let apple_targets = config.apple_targets();

    if !options.no_build {
        let step = reporter.step("Building Apple targets");
        build_apple_targets(
            config,
            &apple_targets,
            options.release,
            &build_cargo_args,
            &step,
        )?;
        step.finish_success();
    }

    let layout = options.layout.unwrap_or_else(|| config.apple_spm_layout());
    let package_root = config.apple_spm_output();

    if options.regenerate {
        let step = reporter.step("Generating Apple bindings");
        generate_apple_bindings(config, layout, &package_root)?;
        step.finish_success();
    }

    let libraries = discover_built_libraries_for_targets(
        &config.crate_artifact_name(),
        build_profile.output_directory_name(),
        &apple_targets,
    )?;
    let apple_libraries: Vec<_> = libraries
        .into_iter()
        .filter(|lib| lib.target.platform().is_apple())
        .collect();

    let missing_targets = missing_built_libraries(&apple_targets, &apple_libraries);
    if !missing_targets.is_empty() {
        return Err(CliError::MissingBuiltLibraries {
            platform: "Apple".to_string(),
            targets: missing_targets,
        });
    }

    let headers_dir = config.apple_header_output();
    if !headers_dir.exists() {
        return Err(CliError::FileNotFound(headers_dir));
    }

    let should_build_xcframework = !options.spm_only;
    let should_generate_spm = !options.xcframework_only;

    let xcframework_output = if should_build_xcframework {
        let step = reporter.step("Creating xcframework");
        let output = XcframeworkBuilder::new(config, apple_libraries.clone(), headers_dir.clone())
            .build_with_zip()?;
        step.finish_success();
        Some(output)
    } else {
        None
    };

    if should_generate_spm {
        let (checksum, version) = match config.apple_spm_distribution() {
            SpmDistribution::Local => (None, None),
            SpmDistribution::Remote => {
                let checksum = xcframework_output
                    .as_ref()
                    .and_then(|o| o.checksum.clone())
                    .map(Ok)
                    .unwrap_or_else(|| {
                        let step = reporter.step("Computing checksum");
                        let result = existing_xcframework_checksum(config);
                        step.finish_success();
                        result
                    })?;
                let version = options
                    .version
                    .or_else(detect_version)
                    .unwrap_or_else(|| "0.1.0".to_string());
                (Some(checksum), Some(version))
            }
        };

        if config.apple_spm_skip_package_swift() {
            reporter.warning("Skipping Package.swift (skip_package_swift = true)");
        } else {
            let generator = match config.apple_spm_distribution() {
                SpmDistribution::Local => SpmPackageGenerator::new_local(config, layout),
                SpmDistribution::Remote => {
                    let checksum = checksum.ok_or_else(|| CliError::CommandFailed {
                        command: "remote SPM requires checksum".to_string(),
                        status: None,
                    })?;
                    let version = version.ok_or_else(|| CliError::CommandFailed {
                        command: "remote SPM requires version".to_string(),
                        status: None,
                    })?;
                    SpmPackageGenerator::new_remote(config, checksum, version, layout)
                }
            };

            let step = reporter.step("Generating Package.swift");
            let package_path = generator.generate()?;
            step.finish_success_with(&format!("{}", package_path.display()));
        }
    }

    Ok(())
}

fn pack_wasm(config: &Config, options: PackWasmOptions, reporter: &Reporter) -> Result<()> {
    if !config.is_wasm_enabled() {
        return Err(CliError::CommandFailed {
            command: "targets.wasm.enabled = false".to_string(),
            status: None,
        });
    }
    if config.wasm_npm_generate_package_json() && config.wasm_npm_package_name().is_none() {
        return Err(CliError::CommandFailed {
            command: "targets.wasm.npm.package_name is required for pack wasm".to_string(),
            status: None,
        });
    }

    reporter.section("🌐", "Packing WASM");

    let requested_wasm_profile = if options.release {
        WasmProfile::Release
    } else {
        config.wasm_profile()
    };

    let build_cargo_args = resolve_build_cargo_args(config, &options.cargo_args);
    let build_profile = resolve_build_profile(
        matches!(requested_wasm_profile, WasmProfile::Release),
        &build_cargo_args,
    );

    let wasm_artifact_profile = match build_profile {
        CargoBuildProfile::Debug => WasmProfile::Debug,
        CargoBuildProfile::Release => WasmProfile::Release,
        CargoBuildProfile::Named(_) if config.wasm_has_artifact_path_override() => {
            requested_wasm_profile
        }
        CargoBuildProfile::Named(profile_name) => {
            return Err(CliError::CommandFailed {
                command: format!(
                    "custom cargo profile '{}' for wasm pack requires targets.wasm.artifact_path",
                    profile_name
                ),
                status: None,
            });
        }
    };

    if !options.no_build {
        let step = reporter.step("Building WASM target");
        build_wasm_target(config, requested_wasm_profile, &build_cargo_args, &step)?;
        step.finish_success();
    }

    let wasm_artifact_path = config.wasm_artifact_path(wasm_artifact_profile);
    if !wasm_artifact_path.exists() {
        return Err(CliError::FileNotFound(wasm_artifact_path));
    }

    if config.wasm_optimize_enabled(wasm_artifact_profile) {
        let step = reporter.step("Optimizing WASM binary");
        optimize_wasm_binary(config, &wasm_artifact_path)?;
        step.finish_success();
    }

    if options.regenerate {
        let step = reporter.step("Generating TypeScript bindings");
        run_generate_with_output(
            config,
            GenerateOptions {
                target: GenerateTarget::Typescript,
                output: Some(config.wasm_typescript_output()),
                experimental: false,
            },
        )?;
        step.finish_success();
    }

    let npm_output = config.wasm_npm_output();
    std::fs::create_dir_all(&npm_output).map_err(|source| CliError::CreateDirectoryFailed {
        path: npm_output.clone(),
        source,
    })?;

    let module_name = config.wasm_typescript_module_name();
    let packaged_wasm_path = npm_output.join(format!("{}_bg.wasm", module_name));
    std::fs::copy(&wasm_artifact_path, &packaged_wasm_path).map_err(|source| {
        CliError::CopyFailed {
            from: wasm_artifact_path.clone(),
            to: packaged_wasm_path.clone(),
            source,
        }
    })?;

    let generated_typescript_source = config
        .wasm_typescript_output()
        .join(format!("{}.ts", module_name));
    if !generated_typescript_source.exists() {
        return Err(CliError::FileNotFound(generated_typescript_source));
    }

    let step = reporter.step("Transpiling TypeScript bindings");
    transpile_typescript_bundle(config, &generated_typescript_source, &npm_output)?;
    step.finish_success();

    let generated_node_typescript_source = config
        .wasm_typescript_output()
        .join(format!("{}_node.ts", module_name));
    if generated_node_typescript_source.exists() {
        let step = reporter.step("Transpiling Node.js bindings");
        transpile_typescript_bundle(config, &generated_node_typescript_source, &npm_output)?;
        step.finish_success();
    }

    let enabled_targets = config.wasm_npm_targets();
    let step = reporter.step("Generating WASM loader entrypoints");
    generate_wasm_loader_entrypoints(&module_name, &enabled_targets, &npm_output)?;
    step.finish_success();

    if config.wasm_npm_generate_package_json() {
        let step = reporter.step("Generating package.json");
        let package_json_path =
            generate_wasm_package_json(config, &module_name, &enabled_targets, &npm_output)?;
        step.finish_success_with(&format!("{}", package_json_path.display()));
    }

    if config.wasm_npm_generate_readme() {
        let step = reporter.step("Generating README.md");
        let readme_path =
            generate_wasm_readme(config, &module_name, &enabled_targets, &npm_output)?;
        step.finish_success_with(&format!("{}", readme_path.display()));
    }

    Ok(())
}

fn pack_android(config: &Config, options: PackAndroidOptions, reporter: &Reporter) -> Result<()> {
    if !config.is_android_enabled() {
        return Err(CliError::CommandFailed {
            command: "targets.android.enabled = false".to_string(),
            status: None,
        });
    }

    reporter.section("🤖", "Packing Android");

    let build_cargo_args = resolve_build_cargo_args(config, &options.cargo_args);
    let build_profile = resolve_build_profile(options.release, &build_cargo_args);
    let android_targets = config.android_targets();

    if !options.no_build {
        let step = reporter.step("Building Android targets");
        build_android_targets(
            config,
            &android_targets,
            options.release,
            &build_cargo_args,
            &step,
        )?;
        step.finish_success();
    }

    if options.regenerate {
        let step = reporter.step("Generating Kotlin bindings");
        run_generate_with_output(
            config,
            GenerateOptions {
                target: GenerateTarget::Kotlin,
                output: Some(config.android_kotlin_output()),
                experimental: false,
            },
        )?;
        step.finish_success();

        let step = reporter.step("Generating C header");
        run_generate_with_output(
            config,
            GenerateOptions {
                target: GenerateTarget::Header,
                output: Some(config.android_header_output()),
                experimental: false,
            },
        )?;
        step.finish_success();
    }

    let libraries = discover_built_libraries_for_targets(
        &config.crate_artifact_name(),
        build_profile.output_directory_name(),
        &android_targets,
    )?;
    let android_libraries: Vec<_> = libraries
        .into_iter()
        .filter(|lib| lib.target.platform() == Platform::Android)
        .collect();

    let missing_targets = missing_built_libraries(&android_targets, &android_libraries);
    if !missing_targets.is_empty() {
        return Err(CliError::MissingBuiltLibraries {
            platform: "Android".to_string(),
            targets: missing_targets,
        });
    }

    let packager = AndroidPackager::new(config, android_libraries, build_profile.is_release_like());
    let step = reporter.step("Packaging jniLibs");
    packager.package()?;
    step.finish_success();

    Ok(())
}

fn pack_java(config: &Config, options: PackJavaOptions, reporter: &Reporter) -> Result<()> {
    if !config.is_java_jvm_enabled() {
        return Err(CliError::CommandFailed {
            command: "targets.java.jvm.enabled = false".to_string(),
            status: None,
        });
    }

    reporter.section("☕", "Packing Java");

    let build_cargo_args = resolve_build_cargo_args(config, &options.cargo_args);
    let build_profile = resolve_build_profile(options.release, &build_cargo_args);
    let java_host_targets = resolve_java_host_targets_for_packaging(config)?;

    ensure_java_no_build_supported(config, options.no_build, options.experimental, "pack java")?;

    if options.regenerate {
        let step = reporter.step("Generating C header");
        generate_java_header(config)?;
        step.finish_success();

        let step = reporter.step("Generating Java bindings");
        run_generate_with_output(
            config,
            GenerateOptions {
                target: GenerateTarget::Java,
                output: Some(config.java_jvm_output()),
                experimental: options.experimental,
            },
        )?;
        step.finish_success();
    }

    let step = reporter.step("Building Rust host library");
    let build_artifacts = build_jvm_native_library(
        config,
        options.release,
        &build_cargo_args,
        java_host_targets[0],
        &step,
    )?;
    step.finish_success();

    let step = reporter.step("Compiling JNI library");
    compile_jni_library(
        config,
        build_profile.output_directory_name(),
        java_host_targets[0],
        &build_artifacts.native_static_libraries,
        &build_artifacts.native_link_search_paths,
        build_artifacts.static_library_filename.as_deref(),
        &build_cargo_args,
    )?;
    step.finish_success();

    reporter.finish();
    Ok(())
}

fn ensure_java_no_build_supported(
    config: &Config,
    no_build: bool,
    experimental: bool,
    command_name: &str,
) -> Result<()> {
    if no_build && config.should_process(Target::Java, experimental) {
        return Err(CliError::CommandFailed {
            command: format!(
                "{command_name} --no-build is unsupported in Phase 3 when JVM packaging is enabled; rerun without --no-build"
            ),
            status: None,
        });
    }

    Ok(())
}

fn generate_java_header(config: &Config) -> Result<()> {
    use boltffi_bindgen::{CHeaderLowerer, ir, scan_crate_with_pointer_width};

    let output_dir = config.java_jvm_output().join("jni");
    let output_path = output_dir.join(format!("{}.h", config.library_name()));

    std::fs::create_dir_all(&output_dir).map_err(|source| CliError::CreateDirectoryFailed {
        path: output_dir.clone(),
        source,
    })?;

    let crate_dir = std::env::current_dir()
        .and_then(|p| p.canonicalize())
        .unwrap_or_else(|_| PathBuf::from("."));
    let crate_name = config.library_name();

    let host_pointer_width_bits = match usize::BITS {
        32 => Some(32),
        64 => Some(64),
        _ => None,
    };
    let mut module = scan_crate_with_pointer_width(&crate_dir, crate_name, host_pointer_width_bits)
        .map_err(|error| CliError::CommandFailed {
            command: format!("scan_crate: {}", error),
            status: None,
        })?;

    let contract = ir::build_contract(&mut module);
    let abi = ir::Lowerer::new(&contract).to_abi_contract();
    let header_code = CHeaderLowerer::new(&contract, &abi).generate();
    std::fs::write(&output_path, header_code).map_err(|source| CliError::WriteFailed {
        path: output_path,
        source,
    })?;

    Ok(())
}

fn compile_jni_library(
    config: &Config,
    profile_directory_name: &str,
    host_target: JavaHostTarget,
    native_static_libraries: &[String],
    native_link_search_paths: &[String],
    static_library_filename: Option<&str>,
    build_cargo_args: &[String],
) -> Result<()> {
    let java_output = config.java_jvm_output();
    let jni_dir = java_output.join("jni");
    let jni_glue = jni_dir.join("jni_glue.c");
    let header = jni_dir.join(format!("{}.h", config.library_name()));

    if !jni_glue.exists() {
        return Err(CliError::FileNotFound(jni_glue));
    }
    if !header.exists() {
        return Err(CliError::FileNotFound(header));
    }

    let artifact_name = config.crate_artifact_name();
    let target_directory = cargo_target_directory_with_args(build_cargo_args)?;
    let crate_outputs = current_jvm_crate_outputs(config, build_cargo_args)?;
    let link_input = resolve_jvm_native_link_input(
        &target_directory,
        profile_directory_name,
        host_target,
        &artifact_name,
        crate_outputs,
        static_library_filename,
    )?;
    let compatibility_shared_library = existing_jvm_shared_library_path(
        &target_directory,
        profile_directory_name,
        host_target,
        &artifact_name,
        crate_outputs,
    );

    let host_native_output = java_output
        .join("native")
        .join(host_target.canonical_name());
    std::fs::create_dir_all(&host_native_output).map_err(|source| {
        CliError::CreateDirectoryFailed {
            path: host_native_output.clone(),
            source,
        }
    })?;

    let output_lib = host_native_output.join(host_target.jni_library_filename(&artifact_name));

    let java_home = std::env::var("JAVA_HOME").map_err(|_| CliError::CommandFailed {
        command: "JAVA_HOME not set".to_string(),
        status: None,
    })?;

    let mut cmd = Command::new("clang");
    cmd.arg("-shared")
        .arg("-fPIC")
        .arg("-o")
        .arg(&output_lib)
        .arg(&jni_glue)
        .arg(link_input.path())
        .arg(format!("-I{}", jni_dir.display()))
        .arg(format!("-I{}/include", java_home))
        .arg(format!(
            "-I{}/include/{}",
            java_home,
            host_target.jni_platform()
        ))
        .args(link_search_path_flags(native_link_search_paths))
        .args(native_static_libraries);

    if let Some(rpath) = host_target.rpath_flag() {
        cmd.arg(rpath);
    }

    let status = cmd.status().map_err(|e| CliError::CommandFailed {
        command: format!("clang: {}", e),
        status: None,
    })?;

    if !status.success() {
        return Err(CliError::CommandFailed {
            command: "clang failed to compile JNI library".to_string(),
            status: status.code(),
        });
    }

    let compatibility_jni_copy = java_output.join(host_target.jni_library_filename(&artifact_name));
    std::fs::copy(&output_lib, &compatibility_jni_copy).map_err(|source| CliError::CopyFailed {
        from: output_lib.clone(),
        to: compatibility_jni_copy,
        source,
    })?;

    if let Some(shared_library) = compatibility_shared_library.as_deref() {
        let shared_library_name = shared_library
            .file_name()
            .expect("shared library path should have a file name");
        let structured_copy = host_native_output.join(shared_library_name);
        std::fs::copy(shared_library, &structured_copy).map_err(|source| CliError::CopyFailed {
            from: shared_library.to_path_buf(),
            to: structured_copy,
            source,
        })?;

        let flat_copy = java_output.join(shared_library_name);
        std::fs::copy(shared_library, &flat_copy).map_err(|source| CliError::CopyFailed {
            from: shared_library.to_path_buf(),
            to: flat_copy,
            source,
        })?;
    } else {
        let stale_shared_library_name = host_target.shared_library_filename(&artifact_name);
        remove_file_if_exists(&host_native_output.join(&stale_shared_library_name))?;
        remove_file_if_exists(&java_output.join(stale_shared_library_name))?;
    }

    Ok(())
}

fn build_jvm_native_library(
    config: &Config,
    release: bool,
    build_cargo_args: &[String],
    host_target: JavaHostTarget,
    step: &Step,
) -> Result<JvmBuildArtifacts> {
    let native_static_libraries = Arc::new(Mutex::new(Vec::<String>::new()));
    let captured_static_libraries = Arc::clone(&native_static_libraries);
    let verbose = step.is_verbose();
    let on_output: Option<OutputCallback> = Some(Box::new(move |line: &str| {
        if verbose {
            print_cargo_line(line);
        }

        if let Some(flags) = parse_native_static_libraries(line) {
            let mut libraries = captured_static_libraries
                .lock()
                .expect("native static libraries lock poisoned");
            *libraries = flags;
        }
    }));

    let metadata = cargo_metadata_with_args(build_cargo_args)?;
    let manifest_path = current_manifest_path_with_args(build_cargo_args)?;
    let options = BuildOptions {
        release,
        package: effective_cargo_package_selector(
            config,
            build_cargo_args,
            &metadata,
            &manifest_path,
        ),
        cargo_args: strip_cargo_package_selectors(build_cargo_args),
        on_output,
    };

    let builder = Builder::new(config, options);
    let result = builder.build_host()?;

    if !result.success {
        return Err(CliError::BuildFailed {
            targets: vec!["host".to_string()],
        });
    }

    let native_static_libraries = native_static_libraries
        .lock()
        .expect("native static libraries lock poisoned")
        .clone();
    let mut native_link_search_paths = Vec::new();

    let native_static_libraries = if native_static_libraries.is_empty() {
        let target_directory = cargo_target_directory_with_args(build_cargo_args)?;
        let crate_outputs = current_jvm_crate_outputs(config, build_cargo_args)?;
        let static_library_filename = if crate_outputs.builds_staticlib {
            resolve_static_library_filename(config, host_target, release, build_cargo_args)?
        } else {
            None
        };
        let staticlib_path = static_library_filename.as_ref().map(|filename| {
            target_directory
                .join(resolve_build_profile(release, build_cargo_args).output_directory_name())
                .join(filename)
        });

        if crate_outputs.builds_staticlib
            && staticlib_path
                .as_ref()
                .is_some_and(|staticlib_path| staticlib_path.exists())
        {
            let link_metadata = query_native_link_metadata(config, release, build_cargo_args)?;
            native_link_search_paths = link_metadata.native_link_search_paths;
            link_metadata.native_static_libraries
        } else {
            native_static_libraries
        }
    } else {
        let target_directory = cargo_target_directory_with_args(build_cargo_args)?;
        let crate_outputs = current_jvm_crate_outputs(config, build_cargo_args)?;
        let static_library_filename = if crate_outputs.builds_staticlib {
            resolve_static_library_filename(config, host_target, release, build_cargo_args)?
        } else {
            None
        };
        let staticlib_path = static_library_filename.as_ref().map(|filename| {
            target_directory
                .join(resolve_build_profile(release, build_cargo_args).output_directory_name())
                .join(filename)
        });

        if crate_outputs.builds_staticlib
            && staticlib_path
                .as_ref()
                .is_some_and(|staticlib_path| staticlib_path.exists())
        {
            native_link_search_paths =
                query_native_link_metadata(config, release, build_cargo_args)?
                    .native_link_search_paths;
        }

        native_static_libraries
    };

    let crate_outputs = current_jvm_crate_outputs(config, build_cargo_args)?;
    let static_library_filename = if crate_outputs.builds_staticlib {
        resolve_static_library_filename(config, host_target, release, build_cargo_args)?
    } else {
        None
    };

    Ok(JvmBuildArtifacts {
        native_static_libraries,
        native_link_search_paths,
        static_library_filename,
    })
}

fn resolve_java_host_targets_for_packaging(config: &Config) -> Result<Vec<JavaHostTarget>> {
    config
        .java_jvm_host_targets()
        .map_err(|message| CliError::CommandFailed {
            command: message,
            status: None,
        })
}

fn resolve_jvm_native_link_input(
    target_directory: &Path,
    profile_directory_name: &str,
    host_target: JavaHostTarget,
    artifact_name: &str,
    crate_outputs: JvmCrateOutputs,
    static_library_filename: Option<&str>,
) -> Result<JvmNativeLinkInput> {
    let staticlib_path = static_library_filename
        .map(|filename| target_directory.join(profile_directory_name).join(filename));
    if crate_outputs.builds_staticlib
        && staticlib_path
            .as_ref()
            .is_some_and(|staticlib_path| staticlib_path.exists())
    {
        return Ok(JvmNativeLinkInput::Staticlib(
            staticlib_path.expect("checked staticlib path existence"),
        ));
    }

    let cdylib_path = target_directory
        .join(profile_directory_name)
        .join(host_target.shared_library_filename(artifact_name));
    if crate_outputs.builds_cdylib && cdylib_path.exists() {
        return Ok(JvmNativeLinkInput::Cdylib(cdylib_path));
    }

    if crate_outputs.builds_staticlib {
        return Err(CliError::FileNotFound(staticlib_path.unwrap_or_else(
            || {
                target_directory
                    .join(profile_directory_name)
                    .join(host_target.static_library_filename(artifact_name))
            },
        )));
    }

    if crate_outputs.builds_cdylib {
        return Err(CliError::FileNotFound(cdylib_path));
    }

    Err(CliError::CommandFailed {
        command:
            "the current library target must enable either staticlib or cdylib for JVM packaging"
                .to_string(),
        status: None,
    })
}

fn existing_jvm_shared_library_path(
    target_directory: &Path,
    profile_directory_name: &str,
    host_target: JavaHostTarget,
    artifact_name: &str,
    crate_outputs: JvmCrateOutputs,
) -> Option<PathBuf> {
    if !crate_outputs.builds_cdylib {
        return None;
    }

    let shared_library_path = target_directory
        .join(profile_directory_name)
        .join(host_target.shared_library_filename(artifact_name));
    shared_library_path.exists().then_some(shared_library_path)
}

fn parse_native_static_libraries(line: &str) -> Option<Vec<String>> {
    let (_, flags) = line.split_once("native-static-libs:")?;
    let parsed: Vec<String> = flags
        .split_whitespace()
        .map(str::to_string)
        .filter(|flag| !flag.is_empty())
        .collect();

    (!parsed.is_empty()).then_some(parsed)
}

fn query_native_link_metadata(
    config: &Config,
    release: bool,
    build_cargo_args: &[String],
) -> Result<NativeLinkMetadata> {
    let crate_dir = std::env::current_dir().map_err(|source| CliError::CommandFailed {
        command: format!("current_dir: {source}"),
        status: None,
    })?;
    let probe_cargo_args = strip_cargo_package_selectors(build_cargo_args);
    let (toolchain_selector, cargo_command_args) = split_toolchain_selector(&probe_cargo_args);
    let package_id = current_cargo_package_id(config, build_cargo_args)?;

    let mut command = Command::new("cargo");
    command.current_dir(crate_dir);

    if let Some(toolchain_selector) = toolchain_selector {
        command.arg(toolchain_selector);
    }

    command.arg("rustc");

    if release {
        command.arg("--release");
    }

    command
        .arg("-p")
        .arg(package_id)
        .args(&cargo_command_args)
        .arg("--message-format=json-render-diagnostics")
        .arg("--lib")
        .arg("--")
        .arg("--print=native-static-libs");

    let output = command.output().map_err(|source| CliError::CommandFailed {
        command: format!("cargo rustc --print=native-static-libs: {source}"),
        status: None,
    })?;

    if !output.status.success() {
        return Err(CliError::CommandFailed {
            command: "cargo rustc --print=native-static-libs".to_string(),
            status: output.status.code(),
        });
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}\n{stderr}");
    let native_link_search_paths = extract_link_search_paths(&stdout);
    let native_static_libraries =
        extract_native_static_libraries(&combined).ok_or_else(|| CliError::CommandFailed {
            command: "cargo rustc --print=native-static-libs did not emit link metadata"
                .to_string(),
            status: None,
        })?;

    Ok(NativeLinkMetadata {
        native_static_libraries,
        native_link_search_paths,
    })
}

fn resolve_static_library_filename(
    config: &Config,
    host_target: JavaHostTarget,
    release: bool,
    build_cargo_args: &[String],
) -> Result<Option<String>> {
    let artifact_name = config.crate_artifact_name();

    if host_target != JavaHostTarget::WindowsX86_64 {
        return Ok(Some(host_target.static_library_filename(&artifact_name)));
    }

    let filenames = query_library_filenames(config, release, build_cargo_args)?;
    select_windows_static_library_filename(&artifact_name, &filenames)
        .map(Some)
        .ok_or_else(|| CliError::CommandFailed {
            command: format!(
                "cargo rustc --print=file-names did not report a Windows static library for '{}'",
                artifact_name
            ),
            status: None,
        })
}

fn query_library_filenames(
    config: &Config,
    release: bool,
    build_cargo_args: &[String],
) -> Result<Vec<String>> {
    let crate_dir = std::env::current_dir().map_err(|source| CliError::CommandFailed {
        command: format!("current_dir: {source}"),
        status: None,
    })?;
    let probe_cargo_args = strip_cargo_package_selectors(build_cargo_args);
    let (toolchain_selector, cargo_command_args) = split_toolchain_selector(&probe_cargo_args);
    let package_id = current_cargo_package_id(config, build_cargo_args)?;

    let mut command = Command::new("cargo");
    command.current_dir(crate_dir);

    if let Some(toolchain_selector) = toolchain_selector {
        command.arg(toolchain_selector);
    }

    command.arg("rustc");

    if release {
        command.arg("--release");
    }

    command
        .arg("-p")
        .arg(package_id)
        .args(&cargo_command_args)
        .arg("--lib")
        .arg("--")
        .arg("--print=file-names");

    let output = command.output().map_err(|source| CliError::CommandFailed {
        command: format!("cargo rustc --print=file-names: {source}"),
        status: None,
    })?;

    if !output.status.success() {
        return Err(CliError::CommandFailed {
            command: "cargo rustc --print=file-names".to_string(),
            status: output.status.code(),
        });
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}\n{stderr}");
    let filenames = extract_library_filenames(&combined);

    if filenames.is_empty() {
        return Err(CliError::CommandFailed {
            command: "cargo rustc --print=file-names did not emit any library filenames"
                .to_string(),
            status: None,
        });
    }

    Ok(filenames)
}

fn extract_library_filenames(output: &str) -> Vec<String> {
    output
        .lines()
        .map(str::trim)
        .filter(|line| {
            !line.is_empty()
                && !line.contains(' ')
                && [".a", ".lib", ".dylib", ".so", ".rlib", ".dll"]
                    .iter()
                    .any(|extension| line.ends_with(extension))
        })
        .map(str::to_string)
        .collect()
}

fn select_windows_static_library_filename(
    artifact_name: &str,
    filenames: &[String],
) -> Option<String> {
    let msvc_name = format!("{artifact_name}.lib");
    let gnu_name = format!("lib{artifact_name}.a");

    filenames
        .iter()
        .find(|filename| *filename == &msvc_name || *filename == &gnu_name)
        .cloned()
}

fn extract_native_static_libraries(output: &str) -> Option<Vec<String>> {
    output
        .lines()
        .filter_map(parse_native_static_libraries)
        .last()
}

fn extract_link_search_paths(output: &str) -> Vec<String> {
    #[derive(Deserialize)]
    struct BuildScriptExecutedMessage {
        reason: String,
        #[serde(default)]
        linked_paths: Vec<String>,
    }

    let mut linked_paths = Vec::new();

    for line in output
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with('{'))
    {
        let Ok(message) = serde_json::from_str::<BuildScriptExecutedMessage>(line) else {
            continue;
        };

        if message.reason != "build-script-executed" {
            continue;
        }

        for linked_path in message.linked_paths {
            if !linked_paths.contains(&linked_path) {
                linked_paths.push(linked_path);
            }
        }
    }

    linked_paths
}

fn link_search_path_flags(link_search_paths: &[String]) -> Vec<String> {
    let mut flags = Vec::new();

    for linked_path in link_search_paths {
        let flag = if let Some(path) = linked_path.strip_prefix("framework=") {
            format!("-F{path}")
        } else if let Some(path) = linked_path.strip_prefix("native=") {
            format!("-L{path}")
        } else if let Some(path) = linked_path.strip_prefix("dependency=") {
            format!("-L{path}")
        } else if let Some(path) = linked_path.strip_prefix("all=") {
            format!("-L{path}")
        } else if let Some(path) = linked_path.strip_prefix("crate=") {
            format!("-L{path}")
        } else {
            format!("-L{linked_path}")
        };

        if !flags.contains(&flag) {
            flags.push(flag);
        }
    }

    flags
}

fn split_toolchain_selector(cargo_args: &[String]) -> (Option<String>, Vec<String>) {
    let toolchain_selector_index = cargo_args
        .iter()
        .position(|argument| argument.starts_with('+') && argument.len() > 1);

    toolchain_selector_index
        .map(|index| {
            let toolchain_selector = cargo_args.get(index).cloned();
            let command_args = cargo_args
                .iter()
                .take(index)
                .chain(cargo_args.iter().skip(index + 1))
                .cloned()
                .collect();
            (toolchain_selector, command_args)
        })
        .unwrap_or_else(|| (None, cargo_args.to_vec()))
}

fn cargo_metadata_args(cargo_args: &[String]) -> Vec<String> {
    let mut metadata_args = Vec::new();
    let mut index = 0;

    while index < cargo_args.len() {
        let argument = &cargo_args[index];
        let takes_value = matches!(
            argument.as_str(),
            "--target-dir" | "--config" | "-Z" | "--manifest-path"
        );
        let keep_current = argument.starts_with('+')
            || takes_value
            || matches!(argument.as_str(), "--locked" | "--offline" | "--frozen")
            || argument.starts_with("--target-dir=")
            || argument.starts_with("--config=")
            || argument.starts_with("-Z")
            || argument.starts_with("--manifest-path=");

        if keep_current {
            metadata_args.push(argument.clone());
            if takes_value
                && !argument.contains('=')
                && let Some(value) = cargo_args.get(index + 1)
            {
                metadata_args.push(value.clone());
                index += 1;
            }
        }

        index += 1;
    }

    metadata_args
}

fn remove_file_if_exists(path: &Path) -> Result<()> {
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(source) => Err(CliError::WriteFailed {
            path: path.to_path_buf(),
            source,
        }),
    }
}

fn current_cargo_package_selector(cargo_args: &[String]) -> Option<String> {
    let mut package_selector = None;
    let mut index = 0;

    while index < cargo_args.len() {
        let argument = &cargo_args[index];

        if let Some(selector) = argument.strip_prefix("--package=") {
            package_selector = Some(selector.to_string());
        } else if let Some(selector) = argument.strip_prefix("-p") {
            if !selector.is_empty() {
                package_selector = Some(selector.to_string());
            } else if let Some(value) = cargo_args.get(index + 1) {
                package_selector = Some(value.clone());
                index += 1;
            }
        } else if argument == "--package"
            && let Some(value) = cargo_args.get(index + 1)
        {
            package_selector = Some(value.clone());
            index += 1;
        }

        index += 1;
    }

    package_selector
}

fn has_explicit_manifest_path(cargo_args: &[String]) -> bool {
    cargo_args
        .iter()
        .any(|argument| argument == "--manifest-path" || argument.starts_with("--manifest-path="))
}

fn effective_cargo_package_selector(
    config: &Config,
    cargo_args: &[String],
    metadata: &CargoMetadata,
    manifest_path: &Path,
) -> Option<String> {
    current_cargo_package_selector(cargo_args).or_else(|| {
        let manifest_selects_package = has_explicit_manifest_path(cargo_args)
            && metadata
                .packages
                .iter()
                .any(|package| package.manifest_path == manifest_path);

        (!manifest_selects_package).then(|| config.package.name.clone())
    })
}

fn strip_cargo_package_selectors(cargo_args: &[String]) -> Vec<String> {
    let mut filtered = Vec::new();
    let mut index = 0;

    while index < cargo_args.len() {
        let argument = &cargo_args[index];

        if argument == "--package" || argument == "-p" {
            index += 1;
            if cargo_args.get(index).is_some() {
                index += 1;
            }
            continue;
        }

        if argument.starts_with("--package=") || (argument.starts_with("-p") && argument.len() > 2)
        {
            index += 1;
            continue;
        }

        filtered.push(argument.clone());
        index += 1;
    }

    filtered
}

fn current_jvm_crate_outputs(config: &Config, cargo_args: &[String]) -> Result<JvmCrateOutputs> {
    let metadata = cargo_metadata_with_args(cargo_args)?;
    let manifest_path = current_manifest_path_with_args(cargo_args)?;
    let package_selector =
        effective_cargo_package_selector(config, cargo_args, &metadata, &manifest_path);
    parse_jvm_crate_outputs(
        &metadata,
        &config.crate_artifact_name(),
        &manifest_path,
        package_selector.as_deref(),
    )
}

fn current_cargo_package_id(config: &Config, cargo_args: &[String]) -> Result<String> {
    let metadata = cargo_metadata_with_args(cargo_args)?;
    let manifest_path = current_manifest_path_with_args(cargo_args)?;
    let package_selector =
        effective_cargo_package_selector(config, cargo_args, &metadata, &manifest_path);
    find_cargo_metadata_package(&metadata, &manifest_path, package_selector.as_deref())
        .map(|package| package.id.clone())
}

fn build_apple_targets(
    config: &Config,
    targets: &[RustTarget],
    release: bool,
    build_cargo_args: &[String],
    step: &Step,
) -> Result<()> {
    let on_output: Option<OutputCallback> = if step.is_verbose() {
        Some(Box::new(|line: &str| {
            print_cargo_line(line);
        }))
    } else {
        None
    };

    let build_options = BuildOptions {
        release,
        package: Some(config.library_name().to_string()),
        cargo_args: build_cargo_args.to_vec(),
        on_output,
    };
    let builder = Builder::new(config, build_options);
    let results = builder.build_targets(targets)?;

    if all_successful(&results) {
        return Ok(());
    }

    let failed = failed_targets(&results);
    Err(CliError::BuildFailed { targets: failed })
}

fn print_cargo_line(line: &str) {
    use console::style;
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with("Fresh") {
        return;
    }

    if trimmed.starts_with("Compiling") {
        println!("      {}", style(trimmed).green());
    } else if trimmed.starts_with("Finished") {
        println!("      {}", style(trimmed).green().bold());
    } else if trimmed.starts_with("warning:") {
        println!("      {}", style(trimmed).yellow());
    } else if trimmed.starts_with("error") {
        println!("      {}", style(trimmed).red().bold());
    } else if trimmed.starts_with("Checking") {
        println!("      {}", style(trimmed).green());
    } else if trimmed.starts_with("Building") {
        println!("      {}", style(trimmed).cyan());
    } else {
        println!("      {}", style(trimmed).dim());
    }
}

fn build_android_targets(
    config: &Config,
    targets: &[RustTarget],
    release: bool,
    build_cargo_args: &[String],
    step: &Step,
) -> Result<()> {
    let on_output: Option<OutputCallback> = if step.is_verbose() {
        Some(Box::new(|line: &str| print_cargo_line(line)))
    } else {
        None
    };

    let build_options = BuildOptions {
        release,
        package: Some(config.library_name().to_string()),
        cargo_args: build_cargo_args.to_vec(),
        on_output,
    };
    let builder = Builder::new(config, build_options);
    let results = builder.build_android(targets)?;

    if all_successful(&results) {
        return Ok(());
    }

    let failed = failed_targets(&results);
    Err(CliError::BuildFailed { targets: failed })
}

fn build_wasm_target(
    config: &Config,
    profile: WasmProfile,
    build_cargo_args: &[String],
    step: &Step,
) -> Result<()> {
    let on_output: Option<OutputCallback> = if step.is_verbose() {
        Some(Box::new(|line: &str| print_cargo_line(line)))
    } else {
        None
    };

    let build_options = BuildOptions {
        release: matches!(profile, WasmProfile::Release),
        package: Some(config.library_name().to_string()),
        cargo_args: build_cargo_args.to_vec(),
        on_output,
    };
    let builder = Builder::new(config, build_options);
    let results = builder.build_wasm_with_triple(config.wasm_triple())?;

    if all_successful(&results) {
        return Ok(());
    }

    let failed = failed_targets(&results);
    Err(CliError::BuildFailed { targets: failed })
}

fn resolve_build_cargo_args(config: &Config, cli_cargo_args: &[String]) -> Vec<String> {
    config
        .cargo_args_for_command("build")
        .into_iter()
        .chain(cli_cargo_args.iter().cloned())
        .collect()
}

fn generate_apple_bindings(config: &Config, layout: SpmLayout, package_root: &Path) -> Result<()> {
    let swift_output_dir = match layout {
        SpmLayout::Bundled => config
            .apple_spm_wrapper_sources()
            .map(|path| package_root.join(path).join("BoltFFI"))
            .unwrap_or_else(|| package_root.join("Sources").join("BoltFFI")),
        SpmLayout::FfiOnly => package_root.join("Sources").join("BoltFFI"),
        SpmLayout::Split => config.apple_swift_output().join("BoltFFI"),
    };

    run_generate_with_output(
        config,
        GenerateOptions {
            target: GenerateTarget::Swift,
            output: Some(swift_output_dir),
            experimental: false,
        },
    )?;

    run_generate_with_output(
        config,
        GenerateOptions {
            target: GenerateTarget::Header,
            output: Some(config.apple_header_output()),
            experimental: false,
        },
    )?;

    Ok(())
}

fn optimize_wasm_binary(config: &Config, wasm_path: &Path) -> Result<()> {
    let optimize_level_flag = match config.wasm_optimize_level() {
        WasmOptimizeLevel::O0 => "-O0",
        WasmOptimizeLevel::O1 => "-O1",
        WasmOptimizeLevel::O2 => "-O2",
        WasmOptimizeLevel::O3 => "-O3",
        WasmOptimizeLevel::O4 => "-O4",
        WasmOptimizeLevel::Size => "-Os",
        WasmOptimizeLevel::MinSize => "-Oz",
    };

    let wasm_opt_path = match which::which("wasm-opt") {
        Ok(path) => path,
        Err(_) => {
            return match config.wasm_optimize_on_missing() {
                WasmOptimizeOnMissing::Error => Err(CliError::CommandFailed {
                    command: "wasm-opt not found in PATH".to_string(),
                    status: None,
                }),
                WasmOptimizeOnMissing::Warn => {
                    println!("warning: wasm-opt not found, skipping optimization");
                    Ok(())
                }
                WasmOptimizeOnMissing::Skip => Ok(()),
            };
        }
    };

    let optimized_path = wasm_path.with_extension("optimized.wasm");
    let mut command = Command::new(wasm_opt_path);
    command
        .arg(optimize_level_flag)
        .arg(wasm_path)
        .arg("-o")
        .arg(&optimized_path);

    if !config.wasm_optimize_strip_debug() {
        command.arg("-g");
    }

    let status = command.status().map_err(|_| CliError::CommandFailed {
        command: "wasm-opt".to_string(),
        status: None,
    })?;

    if !status.success() {
        return Err(CliError::CommandFailed {
            command: "wasm-opt".to_string(),
            status: status.code(),
        });
    }

    std::fs::rename(&optimized_path, wasm_path).map_err(|source| CliError::WriteFailed {
        path: wasm_path.to_path_buf(),
        source,
    })
}

fn transpile_typescript_bundle(
    config: &Config,
    source_file: &Path,
    output_dir: &Path,
) -> Result<()> {
    let mut command = if cfg!(windows) {
        let mut cmd = Command::new("cmd");
        cmd.args(["/C", "npx", "tsc"]);
        cmd
    } else {
        Command::new("tsc")
    };
    command
        .arg(source_file)
        .arg("--target")
        .arg("ES2020")
        .arg("--module")
        .arg("ES2020")
        .arg("--moduleResolution")
        .arg("bundler")
        .arg("--declaration")
        .arg("--sourceMap")
        .arg(if config.wasm_source_map_enabled() {
            "true"
        } else {
            "false"
        })
        .arg("--skipLibCheck")
        .arg("--noEmitOnError")
        .arg("false")
        .arg("--outDir")
        .arg(output_dir);

    let output = command.output().map_err(|_| CliError::CommandFailed {
        command: "tsc".to_string(),
        status: None,
    })?;

    let module_name = config.wasm_typescript_module_name();
    let javascript_path = output_dir.join(format!("{}.js", module_name));
    let declarations_path = output_dir.join(format!("{}.d.ts", module_name));
    let emitted_outputs_exist = javascript_path.exists() && declarations_path.exists();

    if output.status.success() || emitted_outputs_exist {
        return Ok(());
    }

    Err(CliError::CommandFailed {
        command: format!("tsc failed: {}", String::from_utf8_lossy(&output.stderr)),
        status: output.status.code(),
    })
}

fn generate_wasm_loader_entrypoints(
    module_name: &str,
    enabled_targets: &[WasmNpmTarget],
    output_dir: &Path,
) -> Result<()> {
    enabled_targets
        .iter()
        .try_for_each(|target| {
            let (filename, content) = match target {
                WasmNpmTarget::Bundler => (
                    "bundler.js",
                    format!(
                        "import init from \"./{module}.js\";\nexport * from \"./{module}.js\";\nexport {{ default as init }} from \"./{module}.js\";\nexport const initialized = (async () => {{\n  const response = await fetch(new URL(\"./{module}_bg.wasm\", import.meta.url));\n  await init(response);\n}})();\n",
                        module = module_name
                    ),
                ),
                WasmNpmTarget::Web => (
                    "web.js",
                    format!(
                        "import init from \"./{module}.js\";\nexport * from \"./{module}.js\";\nexport {{ default as init }} from \"./{module}.js\";\nexport const initialized = (async () => {{\n  const response = await fetch(new URL(\"./{module}_bg.wasm\", import.meta.url));\n  await init(response);\n}})();\n",
                        module = module_name
                    ),
                ),
                WasmNpmTarget::Nodejs => (
                    "node.js",
                    format!(
                        "export * from \"./{module}_node.js\";\nexport {{ default, initialized }} from \"./{module}_node.js\";\n",
                        module = module_name
                    ),
                ),
            };

            let path = output_dir.join(filename);
            std::fs::write(&path, content).map_err(|source| CliError::WriteFailed {
                path,
                source,
            })
        })
}

fn generate_wasm_package_json(
    config: &Config,
    module_name: &str,
    enabled_targets: &[WasmNpmTarget],
    output_dir: &Path,
) -> Result<PathBuf> {
    let package_name = config
        .wasm_npm_package_name()
        .ok_or_else(|| CliError::CommandFailed {
            command: "targets.wasm.npm.package_name is required for pack wasm".to_string(),
            status: None,
        })?;
    let package_version = config
        .wasm_npm_version()
        .unwrap_or_else(|| "0.1.0".to_string());

    let has_bundler = enabled_targets.contains(&WasmNpmTarget::Bundler);
    let has_web = enabled_targets.contains(&WasmNpmTarget::Web);
    let has_node = enabled_targets.contains(&WasmNpmTarget::Nodejs);
    let default_entry = if has_bundler {
        "./bundler.js"
    } else if has_web {
        "./web.js"
    } else {
        "./node.js"
    };

    let runtime_package = config.wasm_runtime_package();
    let runtime_version = config.wasm_runtime_version();
    let mut dependencies = BTreeMap::new();
    dependencies.insert(runtime_package, runtime_version);

    let package_json = WasmPackageJson {
        name: package_name.to_string(),
        version: package_version,
        package_type: "module".to_string(),
        exports: WasmPackageExports {
            root: WasmPackageEntry {
                types: format!("./{}.d.ts", module_name),
                browser: has_web.then(|| "./web.js".to_string()),
                node: has_node.then(|| "./node.js".to_string()),
                default: default_entry.to_string(),
            },
        },
        types: format!("./{}.d.ts", module_name),
        files: vec![
            format!("{}.js", module_name),
            format!("{}.d.ts", module_name),
            format!("{}_bg.wasm", module_name),
            "bundler.js".to_string(),
            "web.js".to_string(),
            "node.js".to_string(),
        ],
        dependencies,
        license: config.wasm_npm_license(),
        repository: config.wasm_npm_repository(),
    };

    let rendered =
        serde_json::to_string_pretty(&package_json).map_err(|source| CliError::CommandFailed {
            command: format!("failed to serialize package.json: {}", source),
            status: None,
        })?;
    let package_json_path = output_dir.join("package.json");
    std::fs::write(&package_json_path, rendered).map_err(|source| CliError::WriteFailed {
        path: package_json_path.clone(),
        source,
    })?;

    Ok(package_json_path)
}

#[derive(Serialize)]
struct WasmPackageJson {
    name: String,
    version: String,
    #[serde(rename = "type")]
    package_type: String,
    exports: WasmPackageExports,
    types: String,
    files: Vec<String>,
    dependencies: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    license: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    repository: Option<String>,
}

#[derive(Serialize)]
struct WasmPackageExports {
    #[serde(rename = ".")]
    root: WasmPackageEntry,
}

#[derive(Serialize)]
struct WasmPackageEntry {
    types: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    browser: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    node: Option<String>,
    default: String,
}

fn generate_wasm_readme(
    config: &Config,
    module_name: &str,
    enabled_targets: &[WasmNpmTarget],
    output_dir: &Path,
) -> Result<PathBuf> {
    let package_name = config.wasm_npm_package_name().unwrap_or(module_name);
    let targets_text = enabled_targets
        .iter()
        .map(|target| match target {
            WasmNpmTarget::Bundler => "bundler",
            WasmNpmTarget::Web => "web",
            WasmNpmTarget::Nodejs => "nodejs",
        })
        .collect::<Vec<_>>()
        .join(", ");
    let content = format!(
        "# {package_name}\n\nGenerated by boltffi.\n\nEnabled wasm npm targets: {targets_text}\n\n```ts\nimport {{ initialized }} from \"{package_name}\";\nawait initialized;\n```\n"
    );

    let readme_path = output_dir.join("README.md");
    std::fs::write(&readme_path, content).map_err(|source| CliError::WriteFailed {
        path: readme_path.clone(),
        source,
    })?;

    Ok(readme_path)
}

#[derive(Deserialize)]
struct CargoMetadata {
    packages: Vec<CargoMetadataPackage>,
    target_directory: PathBuf,
}

#[derive(Deserialize)]
struct CargoMetadataPackage {
    id: String,
    name: String,
    manifest_path: PathBuf,
    targets: Vec<CargoMetadataPackageTarget>,
}

#[derive(Deserialize)]
struct CargoMetadataPackageTarget {
    name: String,
    crate_types: Vec<String>,
}

fn discover_built_libraries_for_targets(
    crate_artifact_name: &str,
    profile_directory_name: &str,
    targets: &[RustTarget],
) -> Result<Vec<BuiltLibrary>> {
    let target_directory = cargo_target_directory()?;
    Ok(BuiltLibrary::discover_for_targets(
        &target_directory,
        crate_artifact_name,
        profile_directory_name,
        targets,
    ))
}

fn missing_built_libraries(targets: &[RustTarget], libraries: &[BuiltLibrary]) -> Vec<String> {
    targets
        .iter()
        .filter(|target| libraries.iter().all(|library| library.target != **target))
        .map(|target| target.triple().to_string())
        .collect()
}

fn cargo_target_directory() -> Result<PathBuf> {
    cargo_target_directory_with_args(&[])
}

fn cargo_target_directory_with_args(cargo_args: &[String]) -> Result<PathBuf> {
    Ok(cargo_metadata_with_args(cargo_args)?.target_directory)
}

fn cargo_metadata_with_args(cargo_args: &[String]) -> Result<CargoMetadata> {
    let crate_dir = std::env::current_dir().map_err(|source| CliError::CommandFailed {
        command: format!("current_dir: {source}"),
        status: None,
    })?;
    let metadata_args = cargo_metadata_args(cargo_args);
    let (toolchain_selector, command_args) = split_toolchain_selector(&metadata_args);
    let mut command = Command::new("cargo");
    command.current_dir(&crate_dir);
    if let Some(toolchain_selector) = toolchain_selector {
        command.arg(toolchain_selector);
    }
    let output = command
        .args(["metadata", "--format-version", "1", "--no-deps"])
        .args(&command_args)
        .output()
        .map_err(|source| CliError::CommandFailed {
            command: format!("cargo metadata: {source}"),
            status: None,
        })?;

    if !output.status.success() {
        return Err(CliError::CommandFailed {
            command: "cargo metadata --format-version 1 --no-deps".to_string(),
            status: output.status.code(),
        });
    }

    parse_cargo_metadata(&output.stdout)
}

fn current_manifest_path_with_args(cargo_args: &[String]) -> Result<PathBuf> {
    if let Some(manifest_path) = cargo_args.iter().enumerate().find_map(|(index, argument)| {
        argument
            .strip_prefix("--manifest-path=")
            .map(PathBuf::from)
            .or_else(|| {
                (argument == "--manifest-path")
                    .then(|| cargo_args.get(index + 1).map(PathBuf::from))
                    .flatten()
            })
    }) {
        let crate_dir = std::env::current_dir().map_err(|source| CliError::CommandFailed {
            command: format!("current_dir: {source}"),
            status: None,
        })?;
        let manifest_path = if manifest_path.is_absolute() {
            manifest_path
        } else {
            crate_dir.join(manifest_path)
        };

        return manifest_path
            .canonicalize()
            .map_err(|source| CliError::CommandFailed {
                command: format!(
                    "canonicalize manifest path {}: {source}",
                    manifest_path.display()
                ),
                status: None,
            });
    }

    let crate_dir = std::env::current_dir().map_err(|source| CliError::CommandFailed {
        command: format!("current_dir: {source}"),
        status: None,
    })?;

    let manifest_path = crate_dir.join("Cargo.toml");
    manifest_path
        .canonicalize()
        .map_err(|source| CliError::CommandFailed {
            command: format!(
                "canonicalize manifest path {}: {source}",
                manifest_path.display()
            ),
            status: None,
        })
}

fn parse_cargo_target_directory(metadata: &[u8]) -> Result<PathBuf> {
    Ok(parse_cargo_metadata(metadata)?.target_directory)
}

fn parse_cargo_metadata(metadata: &[u8]) -> Result<CargoMetadata> {
    serde_json::from_slice::<CargoMetadata>(metadata).map_err(|source| CliError::CommandFailed {
        command: format!("parse cargo metadata: {source}"),
        status: None,
    })
}

fn parse_jvm_crate_outputs(
    metadata: &CargoMetadata,
    crate_artifact_name: &str,
    manifest_path: &Path,
    package_selector: Option<&str>,
) -> Result<JvmCrateOutputs> {
    let package = find_cargo_metadata_package(metadata, manifest_path, package_selector)?;
    let target = package
        .targets
        .iter()
        .find(|target| target.name == crate_artifact_name)
        .ok_or_else(|| CliError::CommandFailed {
            command: format!(
                "could not find library target '{}' in cargo metadata for '{}'",
                crate_artifact_name,
                manifest_path.display()
            ),
            status: None,
        })?;

    Ok(JvmCrateOutputs {
        builds_staticlib: target
            .crate_types
            .iter()
            .any(|crate_type| crate_type == "staticlib"),
        builds_cdylib: target
            .crate_types
            .iter()
            .any(|crate_type| crate_type == "cdylib"),
    })
}

fn find_cargo_metadata_package<'a>(
    metadata: &'a CargoMetadata,
    manifest_path: &Path,
    package_selector: Option<&str>,
) -> Result<&'a CargoMetadataPackage> {
    if let Some(package_selector) = package_selector {
        return metadata
            .packages
            .iter()
            .find(|package| package.name == package_selector || package.id == package_selector)
            .ok_or_else(|| CliError::CommandFailed {
                command: format!(
                    "could not find selected cargo package '{}' in cargo metadata",
                    package_selector
                ),
                status: None,
            });
    }

    metadata
        .packages
        .iter()
        .find(|package| package.manifest_path == manifest_path)
        .ok_or_else(|| CliError::CommandFailed {
            command: format!(
                "could not find current package manifest '{}' in cargo metadata",
                manifest_path.display()
            ),
            status: None,
        })
}

fn existing_xcframework_checksum(config: &Config) -> Result<String> {
    let xcframework_zip = config
        .apple_xcframework_output()
        .join(format!("{}.xcframework.zip", config.xcframework_name()));

    if xcframework_zip.exists() {
        return compute_checksum(&xcframework_zip);
    }

    Err(CliError::FileNotFound(xcframework_zip))
}

fn detect_version() -> Option<String> {
    std::fs::read_to_string("Cargo.toml")
        .ok()
        .and_then(|content| {
            content
                .lines()
                .find(|line| line.starts_with("version = "))
                .and_then(|line| {
                    line.split('=')
                        .nth(1)
                        .map(|s| s.trim().trim_matches('"').to_string())
                })
        })
}

#[cfg(test)]
mod tests {
    use super::{
        CargoMetadata, CargoMetadataPackage, CargoMetadataPackageTarget, JvmCrateOutputs,
        cargo_metadata_args, current_cargo_package_selector, current_manifest_path_with_args,
        effective_cargo_package_selector, ensure_java_no_build_supported,
        existing_jvm_shared_library_path, extract_library_filenames, extract_link_search_paths,
        extract_native_static_libraries, find_cargo_metadata_package, link_search_path_flags,
        missing_built_libraries, parse_cargo_target_directory, parse_jvm_crate_outputs,
        parse_native_static_libraries, remove_file_if_exists, resolve_jvm_native_link_input,
        select_windows_static_library_filename, split_toolchain_selector,
        strip_cargo_package_selectors,
    };
    use crate::config::{CargoConfig, Config, PackageConfig, TargetsConfig};
    use crate::error::CliError;
    use crate::target::{BuiltLibrary, JavaHostTarget, RustTarget};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn parses_target_directory_from_cargo_metadata() {
        let metadata = br#"{
            "packages": [],
            "workspace_members": [],
            "workspace_default_members": [],
            "resolve": null,
            "target_directory": "/tmp/boltffi-target",
            "version": 1,
            "workspace_root": "/tmp/demo"
        }"#;

        let target_directory =
            parse_cargo_target_directory(metadata).expect("expected target directory");

        assert_eq!(target_directory, PathBuf::from("/tmp/boltffi-target"));
    }

    #[test]
    fn reports_missing_built_libraries_for_unbuilt_configured_targets() {
        let libraries = vec![BuiltLibrary {
            target: RustTarget::ANDROID_ARM64,
            path: PathBuf::from("/tmp/libdemo.a"),
        }];

        let missing = missing_built_libraries(
            &[RustTarget::ANDROID_ARM64, RustTarget::ANDROID_X86_64],
            &libraries,
        );

        assert_eq!(missing, vec!["x86_64-linux-android".to_string()]);
    }

    #[test]
    fn parses_native_static_library_flags_from_cargo_output() {
        let parsed = parse_native_static_libraries(
            "note: native-static-libs: -framework Security -lresolv -lc++",
        )
        .expect("expected static library flags");

        assert_eq!(parsed, vec!["-framework", "Security", "-lresolv", "-lc++"]);
    }

    #[test]
    fn preserves_repeated_framework_prefixes_in_native_static_library_flags() {
        let parsed = parse_native_static_libraries(
            "note: native-static-libs: -framework Security -framework SystemConfiguration -lobjc",
        )
        .expect("expected static library flags");

        assert_eq!(
            parsed,
            vec![
                "-framework",
                "Security",
                "-framework",
                "SystemConfiguration",
                "-lobjc",
            ]
        );
    }

    #[test]
    fn extracts_last_native_static_library_line_from_combined_output() {
        let parsed = extract_native_static_libraries(
            "Compiling demo\nnote: native-static-libs: -lSystem\nFinished\nnote: native-static-libs: -framework CoreFoundation -lSystem\n",
        )
        .expect("expected static library flags");

        assert_eq!(parsed, vec!["-framework", "CoreFoundation", "-lSystem"]);
    }

    #[test]
    fn extracts_link_search_paths_from_build_script_messages() {
        let linked_paths = extract_link_search_paths(
            r#"{"reason":"compiler-artifact","package_id":"path+file:///tmp/demo#0.1.0"}
{"reason":"build-script-executed","package_id":"path+file:///tmp/dep#0.1.0","linked_paths":["native=/tmp/out","framework=/tmp/frameworks","native=/tmp/out"]}"#,
        );

        assert_eq!(
            linked_paths,
            vec![
                "native=/tmp/out".to_string(),
                "framework=/tmp/frameworks".to_string(),
            ]
        );
    }

    #[test]
    fn converts_link_search_paths_to_clang_flags() {
        let flags = link_search_path_flags(&[
            "native=/tmp/out".to_string(),
            "framework=/tmp/frameworks".to_string(),
            "dependency=/tmp/deps".to_string(),
            "/tmp/plain".to_string(),
            "native=/tmp/out".to_string(),
        ]);

        assert_eq!(
            flags,
            vec![
                "-L/tmp/out".to_string(),
                "-F/tmp/frameworks".to_string(),
                "-L/tmp/deps".to_string(),
                "-L/tmp/plain".to_string(),
            ]
        );
    }

    #[test]
    fn rejects_pack_all_no_build_when_java_is_enabled() {
        let config = Config {
            experimental: Vec::new(),
            cargo: CargoConfig::default(),
            package: PackageConfig {
                name: "workspace-member".to_string(),
                crate_name: None,
                version: None,
                description: None,
                license: None,
                repository: None,
            },
            targets: TargetsConfig {
                java: crate::config::JavaConfig {
                    jvm: crate::config::JavaJvmConfig {
                        enabled: true,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            },
        };

        let error = ensure_java_no_build_supported(&config, true, false, "pack all")
            .expect_err("expected no-build rejection");

        assert!(matches!(
            error,
            CliError::CommandFailed { command, status: None }
                if command.contains("pack all --no-build is unsupported in Phase 3")
        ));
    }

    #[test]
    fn allows_pack_all_no_build_when_java_is_disabled() {
        let config = Config {
            experimental: Vec::new(),
            cargo: CargoConfig::default(),
            package: PackageConfig {
                name: "workspace-member".to_string(),
                crate_name: None,
                version: None,
                description: None,
                license: None,
                repository: None,
            },
            targets: TargetsConfig::default(),
        };

        ensure_java_no_build_supported(&config, true, false, "pack all")
            .expect("expected no-build to be allowed");
    }

    #[test]
    fn extracts_library_filenames_from_print_file_names_output() {
        let filenames = extract_library_filenames(
            "Compiling demo\nlibdemo.a\nlibdemo.dylib\nlibdemo.rlib\nFinished\n",
        );

        assert_eq!(
            filenames,
            vec![
                "libdemo.a".to_string(),
                "libdemo.dylib".to_string(),
                "libdemo.rlib".to_string(),
            ]
        );
    }

    #[test]
    fn selects_windows_static_library_filename_from_reported_outputs() {
        let filename = select_windows_static_library_filename(
            "demo",
            &[
                "demo.lib".to_string(),
                "demo.dll".to_string(),
                "demo.rlib".to_string(),
            ],
        )
        .expect("expected windows staticlib filename");

        assert_eq!(filename, "demo.lib");
    }

    #[test]
    fn selects_windows_gnu_static_library_filename_from_reported_outputs() {
        let filename = select_windows_static_library_filename(
            "demo",
            &[
                "libdemo.a".to_string(),
                "demo.dll".to_string(),
                "demo.rlib".to_string(),
            ],
        )
        .expect("expected windows gnu staticlib filename");

        assert_eq!(filename, "libdemo.a");
    }

    #[test]
    fn splits_toolchain_selector_from_cargo_args() {
        let (toolchain_selector, command_args) = split_toolchain_selector(&[
            "--features".to_string(),
            "demo".to_string(),
            "+nightly".to_string(),
            "--locked".to_string(),
        ]);

        assert_eq!(toolchain_selector.as_deref(), Some("+nightly"));
        assert_eq!(
            command_args,
            vec![
                "--features".to_string(),
                "demo".to_string(),
                "--locked".to_string()
            ]
        );
    }

    #[test]
    fn keeps_metadata_relevant_cargo_args() {
        let metadata_args = cargo_metadata_args(&[
            "+nightly".to_string(),
            "--target-dir".to_string(),
            "out/target".to_string(),
            "--config=build.target-dir=\"other-target\"".to_string(),
            "--locked".to_string(),
            "--features".to_string(),
            "demo".to_string(),
            "--manifest-path".to_string(),
            "examples/demo/Cargo.toml".to_string(),
            "-Zunstable-options".to_string(),
        ]);

        assert_eq!(
            metadata_args,
            vec![
                "+nightly".to_string(),
                "--target-dir".to_string(),
                "out/target".to_string(),
                "--config=build.target-dir=\"other-target\"".to_string(),
                "--locked".to_string(),
                "--manifest-path".to_string(),
                "examples/demo/Cargo.toml".to_string(),
                "-Zunstable-options".to_string(),
            ]
        );
    }

    #[test]
    fn canonicalizes_manifest_path_from_split_cargo_args() {
        let expected = std::env::current_dir()
            .expect("current dir")
            .join("Cargo.toml")
            .canonicalize()
            .expect("canonical manifest path");

        let manifest_path = current_manifest_path_with_args(&[
            "--manifest-path".to_string(),
            "Cargo.toml".to_string(),
        ])
        .expect("manifest path");

        assert_eq!(manifest_path, expected);
    }

    #[test]
    fn canonicalizes_manifest_path_from_equals_cargo_arg() {
        let expected = std::env::current_dir()
            .expect("current dir")
            .join("Cargo.toml")
            .canonicalize()
            .expect("canonical manifest path");

        let manifest_path =
            current_manifest_path_with_args(&["--manifest-path=Cargo.toml".to_string()])
                .expect("manifest path");

        assert_eq!(manifest_path, expected);
    }

    #[test]
    fn canonicalizes_implicit_manifest_path() {
        let expected = std::env::current_dir()
            .expect("current dir")
            .join("Cargo.toml")
            .canonicalize()
            .expect("canonical manifest path");

        let manifest_path = current_manifest_path_with_args(&[]).expect("manifest path");

        assert_eq!(manifest_path, expected);
    }

    #[test]
    fn extracts_last_package_selector_from_cargo_args() {
        let package_selector = current_cargo_package_selector(&[
            "--manifest-path".to_string(),
            "Cargo.toml".to_string(),
            "-p".to_string(),
            "first".to_string(),
            "--package=second".to_string(),
        ]);

        assert_eq!(package_selector.as_deref(), Some("second"));
    }

    #[test]
    fn strips_package_selectors_from_probe_cargo_args() {
        let cargo_args = strip_cargo_package_selectors(&[
            "+nightly".to_string(),
            "--package".to_string(),
            "member-a".to_string(),
            "-pmember-b".to_string(),
            "--features".to_string(),
            "demo".to_string(),
            "--package=member-c".to_string(),
            "--release".to_string(),
        ]);

        assert_eq!(
            cargo_args,
            vec![
                "+nightly".to_string(),
                "--features".to_string(),
                "demo".to_string(),
                "--release".to_string(),
            ]
        );
    }

    #[test]
    fn falls_back_to_config_package_name_for_effective_package_selector() {
        let config = Config {
            experimental: Vec::new(),
            cargo: CargoConfig::default(),
            package: PackageConfig {
                name: "workspace-member".to_string(),
                crate_name: None,
                version: None,
                description: None,
                license: None,
                repository: None,
            },
            targets: TargetsConfig::default(),
        };

        let metadata = CargoMetadata {
            target_directory: PathBuf::from("/tmp/boltffi-target"),
            packages: vec![],
        };
        let package_selector = effective_cargo_package_selector(
            &config,
            &[],
            &metadata,
            Path::new("/tmp/workspace/Cargo.toml"),
        );

        assert_eq!(package_selector.as_deref(), Some("workspace-member"));
    }

    #[test]
    fn falls_back_to_cargo_package_name_when_crate_name_differs() {
        let config = Config {
            experimental: Vec::new(),
            cargo: CargoConfig::default(),
            package: PackageConfig {
                name: "workspace-member".to_string(),
                crate_name: Some("ffi_member".to_string()),
                version: None,
                description: None,
                license: None,
                repository: None,
            },
            targets: TargetsConfig::default(),
        };

        let metadata = CargoMetadata {
            target_directory: PathBuf::from("/tmp/boltffi-target"),
            packages: vec![],
        };
        let package_selector = effective_cargo_package_selector(
            &config,
            &[],
            &metadata,
            Path::new("/tmp/workspace/Cargo.toml"),
        );

        assert_eq!(package_selector.as_deref(), Some("workspace-member"));
    }

    #[test]
    fn returns_none_for_effective_package_selector_when_manifest_path_selects_package() {
        let config = Config {
            experimental: Vec::new(),
            cargo: CargoConfig::default(),
            package: PackageConfig {
                name: "workspace-member".to_string(),
                crate_name: None,
                version: None,
                description: None,
                license: None,
                repository: None,
            },
            targets: TargetsConfig::default(),
        };
        let metadata = CargoMetadata {
            target_directory: PathBuf::from("/tmp/boltffi-target"),
            packages: vec![CargoMetadataPackage {
                id: "path+file:///tmp/workspace/member#0.1.0".to_string(),
                name: "workspace-member".to_string(),
                manifest_path: PathBuf::from("/tmp/workspace/member/Cargo.toml"),
                targets: vec![],
            }],
        };

        let package_selector = effective_cargo_package_selector(
            &config,
            &[
                "--manifest-path".to_string(),
                "member/Cargo.toml".to_string(),
            ],
            &metadata,
            Path::new("/tmp/workspace/member/Cargo.toml"),
        );

        assert_eq!(package_selector, None);
    }

    #[test]
    fn falls_back_to_config_package_name_for_virtual_workspace_manifest_path() {
        let config = Config {
            experimental: Vec::new(),
            cargo: CargoConfig::default(),
            package: PackageConfig {
                name: "workspace-member".to_string(),
                crate_name: None,
                version: None,
                description: None,
                license: None,
                repository: None,
            },
            targets: TargetsConfig::default(),
        };
        let metadata = CargoMetadata {
            target_directory: PathBuf::from("/tmp/boltffi-target"),
            packages: vec![CargoMetadataPackage {
                id: "path+file:///tmp/workspace/member#0.1.0".to_string(),
                name: "workspace-member".to_string(),
                manifest_path: PathBuf::from("/tmp/workspace/member/Cargo.toml"),
                targets: vec![],
            }],
        };

        let package_selector = effective_cargo_package_selector(
            &config,
            &[
                "--manifest-path".to_string(),
                "/tmp/workspace/Cargo.toml".to_string(),
            ],
            &metadata,
            Path::new("/tmp/workspace/Cargo.toml"),
        );

        assert_eq!(package_selector.as_deref(), Some("workspace-member"));
    }

    #[test]
    fn prefers_explicit_package_selector_over_config_package_name() {
        let config = Config {
            experimental: Vec::new(),
            cargo: CargoConfig::default(),
            package: PackageConfig {
                name: "workspace-member".to_string(),
                crate_name: None,
                version: None,
                description: None,
                license: None,
                repository: None,
            },
            targets: TargetsConfig::default(),
        };
        let metadata = CargoMetadata {
            target_directory: PathBuf::from("/tmp/boltffi-target"),
            packages: vec![],
        };

        let package_selector = effective_cargo_package_selector(
            &config,
            &["--package=selected-member".to_string()],
            &metadata,
            Path::new("/tmp/workspace/Cargo.toml"),
        );

        assert_eq!(package_selector.as_deref(), Some("selected-member"));
    }

    #[test]
    fn remove_file_if_exists_deletes_existing_file() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time went backwards")
            .as_nanos();
        let temp_root = std::env::temp_dir().join(format!("boltffi-remove-file-test-{unique}"));
        fs::create_dir_all(&temp_root).expect("create temp dir");
        let file_path = temp_root.join("stale.dylib");
        fs::write(&file_path, []).expect("write temp file");

        remove_file_if_exists(&file_path).expect("remove stale file");

        assert!(!file_path.exists());

        fs::remove_dir_all(&temp_root).expect("cleanup temp dir");
    }

    #[test]
    fn prefers_staticlib_for_jvm_linking_when_available() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time went backwards")
            .as_nanos();
        let temp_root = std::env::temp_dir().join(format!("boltffi-jvm-link-test-{unique}"));
        let profile_dir = temp_root.join("release");
        fs::create_dir_all(&profile_dir).expect("create profile dir");

        let staticlib = profile_dir.join("libdemo.a");
        let cdylib = profile_dir.join("libdemo.dylib");
        fs::write(&staticlib, []).expect("write staticlib");
        fs::write(&cdylib, []).expect("write cdylib");

        let resolved = resolve_jvm_native_link_input(
            &temp_root,
            "release",
            JavaHostTarget::DarwinArm64,
            "demo",
            JvmCrateOutputs {
                builds_staticlib: true,
                builds_cdylib: true,
            },
            Some("libdemo.a"),
        )
        .expect("expected link input");

        assert_eq!(resolved.path(), staticlib.as_path());

        fs::remove_dir_all(&temp_root).expect("cleanup temp target dir");
    }

    #[test]
    fn finds_shared_library_compatibility_copy_even_when_staticlib_exists() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time went backwards")
            .as_nanos();
        let temp_root = std::env::temp_dir().join(format!("boltffi-jvm-copy-test-{unique}"));
        let profile_dir = temp_root.join("release");
        fs::create_dir_all(&profile_dir).expect("create profile dir");

        let staticlib = profile_dir.join("libdemo.a");
        let cdylib = profile_dir.join("libdemo.dylib");
        fs::write(&staticlib, []).expect("write staticlib");
        fs::write(&cdylib, []).expect("write cdylib");

        let resolved = resolve_jvm_native_link_input(
            &temp_root,
            "release",
            JavaHostTarget::DarwinArm64,
            "demo",
            JvmCrateOutputs {
                builds_staticlib: true,
                builds_cdylib: true,
            },
            Some("libdemo.a"),
        )
        .expect("expected link input");
        let compatibility_shared_library = existing_jvm_shared_library_path(
            &temp_root,
            "release",
            JavaHostTarget::DarwinArm64,
            "demo",
            JvmCrateOutputs {
                builds_staticlib: true,
                builds_cdylib: true,
            },
        )
        .expect("expected shared library compatibility copy");

        assert_eq!(resolved.path(), staticlib.as_path());
        assert_eq!(compatibility_shared_library, cdylib);

        fs::remove_dir_all(&temp_root).expect("cleanup temp target dir");
    }

    #[test]
    fn ignores_stale_staticlib_when_current_crate_is_cdylib_only() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time went backwards")
            .as_nanos();
        let temp_root = std::env::temp_dir().join(format!("boltffi-jvm-stale-static-{unique}"));
        let profile_dir = temp_root.join("release");
        fs::create_dir_all(&profile_dir).expect("create profile dir");

        let staticlib = profile_dir.join("libdemo.a");
        let cdylib = profile_dir.join("libdemo.dylib");
        fs::write(&staticlib, []).expect("write stale staticlib");
        fs::write(&cdylib, []).expect("write current cdylib");

        let resolved = resolve_jvm_native_link_input(
            &temp_root,
            "release",
            JavaHostTarget::DarwinArm64,
            "demo",
            JvmCrateOutputs {
                builds_staticlib: false,
                builds_cdylib: true,
            },
            None,
        )
        .expect("expected link input");

        assert_eq!(resolved.path(), cdylib.as_path());

        fs::remove_dir_all(&temp_root).expect("cleanup temp target dir");
    }

    #[test]
    fn ignores_stale_shared_library_when_current_crate_is_staticlib_only() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time went backwards")
            .as_nanos();
        let temp_root = std::env::temp_dir().join(format!("boltffi-jvm-stale-cdylib-{unique}"));
        let profile_dir = temp_root.join("release");
        fs::create_dir_all(&profile_dir).expect("create profile dir");

        let cdylib = profile_dir.join("libdemo.dylib");
        fs::write(&cdylib, []).expect("write stale shared library");

        let compatibility_shared_library = existing_jvm_shared_library_path(
            &temp_root,
            "release",
            JavaHostTarget::DarwinArm64,
            "demo",
            JvmCrateOutputs {
                builds_staticlib: true,
                builds_cdylib: false,
            },
        );

        assert!(compatibility_shared_library.is_none());

        fs::remove_dir_all(&temp_root).expect("cleanup temp target dir");
    }

    #[test]
    fn parses_current_jvm_crate_outputs_from_cargo_metadata() {
        let metadata = CargoMetadata {
            target_directory: PathBuf::from("/tmp/boltffi-target"),
            packages: vec![
                CargoMetadataPackage {
                    id: "path+file:///tmp/workspace/sibling#0.1.0".to_string(),
                    name: "sibling".to_string(),
                    manifest_path: PathBuf::from("/tmp/workspace/sibling/Cargo.toml"),
                    targets: vec![CargoMetadataPackageTarget {
                        name: "demo".to_string(),
                        crate_types: vec!["cdylib".to_string()],
                    }],
                },
                CargoMetadataPackage {
                    id: "path+file:///tmp/workspace/current#0.1.0".to_string(),
                    name: "current".to_string(),
                    manifest_path: PathBuf::from("/tmp/workspace/current/Cargo.toml"),
                    targets: vec![
                        CargoMetadataPackageTarget {
                            name: "demo".to_string(),
                            crate_types: vec![
                                "staticlib".to_string(),
                                "cdylib".to_string(),
                                "rlib".to_string(),
                            ],
                        },
                        CargoMetadataPackageTarget {
                            name: "demo_cli".to_string(),
                            crate_types: vec!["bin".to_string()],
                        },
                    ],
                },
            ],
        };

        let outputs = parse_jvm_crate_outputs(
            &metadata,
            "demo",
            Path::new("/tmp/workspace/current/Cargo.toml"),
            None,
        )
        .expect("crate outputs");

        assert_eq!(
            outputs,
            JvmCrateOutputs {
                builds_staticlib: true,
                builds_cdylib: true,
            }
        );
    }

    #[test]
    fn scopes_jvm_crate_outputs_to_selected_package_manifest() {
        let metadata = CargoMetadata {
            target_directory: PathBuf::from("/tmp/boltffi-target"),
            packages: vec![
                CargoMetadataPackage {
                    id: "path+file:///tmp/workspace/a#0.1.0".to_string(),
                    name: "workspace-a".to_string(),
                    manifest_path: PathBuf::from("/tmp/workspace/a/Cargo.toml"),
                    targets: vec![CargoMetadataPackageTarget {
                        name: "shared_name".to_string(),
                        crate_types: vec!["cdylib".to_string()],
                    }],
                },
                CargoMetadataPackage {
                    id: "path+file:///tmp/workspace/b#0.1.0".to_string(),
                    name: "workspace-b".to_string(),
                    manifest_path: PathBuf::from("/tmp/workspace/b/Cargo.toml"),
                    targets: vec![CargoMetadataPackageTarget {
                        name: "shared_name".to_string(),
                        crate_types: vec!["staticlib".to_string()],
                    }],
                },
            ],
        };

        let outputs = parse_jvm_crate_outputs(
            &metadata,
            "shared_name",
            Path::new("/tmp/workspace/b/Cargo.toml"),
            None,
        )
        .expect("crate outputs");

        assert_eq!(
            outputs,
            JvmCrateOutputs {
                builds_staticlib: true,
                builds_cdylib: false,
            }
        );
    }

    #[test]
    fn finds_current_cargo_metadata_package_by_manifest_path() {
        let metadata = CargoMetadata {
            target_directory: PathBuf::from("/tmp/boltffi-target"),
            packages: vec![
                CargoMetadataPackage {
                    id: "path+file:///tmp/workspace/a#0.1.0".to_string(),
                    name: "workspace-a".to_string(),
                    manifest_path: PathBuf::from("/tmp/workspace/a/Cargo.toml"),
                    targets: vec![],
                },
                CargoMetadataPackage {
                    id: "path+file:///tmp/workspace/b#0.1.0".to_string(),
                    name: "workspace-b".to_string(),
                    manifest_path: PathBuf::from("/tmp/workspace/b/Cargo.toml"),
                    targets: vec![],
                },
            ],
        };

        let package =
            find_cargo_metadata_package(&metadata, Path::new("/tmp/workspace/b/Cargo.toml"), None)
                .expect("package lookup");

        assert_eq!(package.id, "path+file:///tmp/workspace/b#0.1.0");
    }

    #[test]
    fn finds_selected_cargo_metadata_package_by_package_name() {
        let metadata = CargoMetadata {
            target_directory: PathBuf::from("/tmp/boltffi-target"),
            packages: vec![
                CargoMetadataPackage {
                    id: "path+file:///tmp/workspace#workspace-a@0.1.0".to_string(),
                    name: "workspace-a".to_string(),
                    manifest_path: PathBuf::from("/tmp/workspace/Cargo.toml"),
                    targets: vec![],
                },
                CargoMetadataPackage {
                    id: "path+file:///tmp/workspace#workspace-b@0.1.0".to_string(),
                    name: "workspace-b".to_string(),
                    manifest_path: PathBuf::from("/tmp/workspace/Cargo.toml"),
                    targets: vec![],
                },
            ],
        };

        let package = find_cargo_metadata_package(
            &metadata,
            Path::new("/tmp/workspace/Cargo.toml"),
            Some("workspace-b"),
        )
        .expect("package lookup");

        assert_eq!(package.id, "path+file:///tmp/workspace#workspace-b@0.1.0");
    }

    #[test]
    fn scopes_jvm_crate_outputs_to_selected_package_name() {
        let metadata = CargoMetadata {
            target_directory: PathBuf::from("/tmp/boltffi-target"),
            packages: vec![
                CargoMetadataPackage {
                    id: "path+file:///tmp/workspace#workspace-a@0.1.0".to_string(),
                    name: "workspace-a".to_string(),
                    manifest_path: PathBuf::from("/tmp/workspace/Cargo.toml"),
                    targets: vec![CargoMetadataPackageTarget {
                        name: "shared_name".to_string(),
                        crate_types: vec!["cdylib".to_string()],
                    }],
                },
                CargoMetadataPackage {
                    id: "path+file:///tmp/workspace#workspace-b@0.1.0".to_string(),
                    name: "workspace-b".to_string(),
                    manifest_path: PathBuf::from("/tmp/workspace/Cargo.toml"),
                    targets: vec![CargoMetadataPackageTarget {
                        name: "shared_name".to_string(),
                        crate_types: vec!["staticlib".to_string()],
                    }],
                },
            ],
        };

        let outputs = parse_jvm_crate_outputs(
            &metadata,
            "shared_name",
            Path::new("/tmp/workspace/Cargo.toml"),
            Some("workspace-b"),
        )
        .expect("crate outputs");

        assert_eq!(
            outputs,
            JvmCrateOutputs {
                builds_staticlib: true,
                builds_cdylib: false,
            }
        );
    }
}
