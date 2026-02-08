# `boltffi.toml` specification

`boltffi.toml` configures `boltffi` code generation and packaging. The CLI reads it from the current working directory.

## Minimal example

```toml
[package]
name = "mylib"
```

Everything else is optional with defaults.

## Top-level

### `[package]` (required)

- `name` (string): logical name used for default module/class naming.
- `crate` (string, optional): Rust crate name to scan/build if different from `name`.

## Apple

### `[apple]` (optional)

- `output` (path): Apple artifact root directory.
  - Default: `dist/apple`
- `deployment_target` (string): iOS deployment target (major.minor).
  - Default: `16.0`
- `include_macos` (bool): whether `boltffi pack apple` also builds macOS targets.
  - Default: `false`

### `[apple.swift]` (optional)

- `module_name` (string, optional): Swift module name for generated bindings.
  - Default: `PascalCase(package.name)`
- `output` (path, optional): where Swift bindings are generated.
  - Default: `{apple.output}/Sources`
- `tools_version` (string, optional): SwiftPM tools version emitted in `Package.swift`.
  - Default: `5.9`
- `error_style` (`throwing` | `result`): error surface style in generated Swift.
  - Default: `throwing`

### `[apple.swift.type_mappings]` (optional)

Maps custom types to native Swift types. When a custom type has a mapping, the generated Swift code uses the native type instead of a typealias, with automatic conversion at the wire boundary.

Each mapping is a table with:
- `type` (string, required): The native Swift type to use (e.g., `UUID`, `URL`).
- `conversion` (string, required): The conversion strategy. One of:
  - `uuid_string`: String ↔ UUID (`UUID(uuidString:)` / `.uuidString`)
  - `url_string`: String ↔ URL (`URL(string:)` / `.absoluteString`)

Example:
```toml
[apple.swift.type_mappings]
Uuid = { type = "UUID", conversion = "uuid_string" }
```

### `[apple.header]` (optional)

- `output` (path, optional): where the generated C header is written.
  - Default: `{apple.output}/include`

### `[apple.xcframework]` (optional)

- `output` (path, optional): where `{Name}.xcframework` and `{Name}.xcframework.zip` are written.
  - Default: `{apple.output}`
- `name` (string, optional): xcframework base name.
  - Default: `{apple.swift.module_name}`

### `[apple.spm]` (optional)

- `output` (path, optional): directory where `Package.swift` is written.
  - Default: `{apple.output}`
- `distribution` (`local` | `remote`): whether `Package.swift` points at a local `.xcframework` or a remote release `.zip`.
  - Default: `local`
- `repo_url` (string, optional): base URL for remote releases (only used for `distribution = "remote"`).
- `layout` (`bundled` | `split` | `ffi-only`): SwiftPM layout.
  - Default: `ffi-only`
- `package_name` (string, optional): SwiftPM package name override.
  - Default:
    - `layout = "split"`: `{apple.swift.module_name}FFI`
    - otherwise: `{apple.swift.module_name}`
- `wrapper_sources` (path, optional): Swift target sources path used by `layout = "bundled"`.
  - Default: `Sources`

## Android

### `[android]` (optional)

- `output` (path): Android artifact root directory.
  - Default: `dist/android`
- `min_sdk` (integer): Android minSdkVersion used for packaging.
  - Default: `24`
- `ndk_version` (string, optional): NDK version hint (used by environment checks).

### `[android.kotlin]` (optional)

- `package` (string, optional): Kotlin package for generated sources.
  - Default: `com.example.{package.name}` (with `-` normalized to `_`)
- `output` (path, optional): output directory for Kotlin sources and JNI glue.
  - Default: `{android.output}/kotlin`
- `error_style` (`throwing` | `result`): error surface style in generated Kotlin.
  - Default: `throwing`
- `factory_style` (`constructors` | `companion_methods`): how factory constructors are exposed.
  - Default: `constructors`

### `[android.kotlin.type_mappings]` (optional)

Maps custom types to native Kotlin/Java types. Same structure as `[apple.swift.type_mappings]`.

Example:
```toml
[android.kotlin.type_mappings]
Uuid = { type = "java.util.UUID", conversion = "uuid_string" }
```

Note: Kotlin type mappings are parsed but not yet implemented in codegen.

### `[android.header]` (optional)

- `output` (path, optional): where the generated C header is written (used by Android JNI builds).
  - Default: `{android.output}/include`

### `[android.pack]` (optional)

- `output` (path, optional): where `boltffi pack android` writes the `jniLibs/` folder.
  - Default: `{android.output}/jniLibs`

## Apple SwiftPM layouts

`boltffi pack apple` always produces an xcframework (unless `--spm-only`) and can generate `Package.swift` (unless `--xcframework-only`).

### `layout = "ffi-only"`

Generates a standalone SwiftPM package containing:

- a binary target `{XcframeworkName}FFI`
- a Swift target `{apple.swift.module_name}` that depends on that binary target
- generated bindings in `{apple.spm.output}/Sources/BoltFFIGenerated/{apple.swift.module_name}.swift`

### `layout = "bundled"`

Generates `Package.swift` that points the Swift target at your existing wrapper sources directory.

- Set `apple.spm.wrapper_sources` to the wrapper target’s source directory.
- Generated bindings go into `{apple.spm.output}/{apple.spm.wrapper_sources}/BoltFFIGenerated/{apple.swift.module_name}.swift`.

### `layout = "split"`

Generates a binary-only SwiftPM package intended to be depended on by a separate wrapper package.

- `Package.swift` exposes only the binary target `{XcframeworkName}FFI`.
- Generated Swift bindings are written to `{apple.swift.output}/BoltFFIGenerated/{apple.swift.module_name}.swift` so you can include them in your wrapper target.
