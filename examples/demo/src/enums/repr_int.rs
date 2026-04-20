use boltffi::*;

/// Task priority with explicit integer discriminants.
///
/// The `#[repr(i32)]` means these values are stable across
/// versions and safe to persist or send over the network.
#[data]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum Priority {
    Low = 0,
    Medium = 1,
    High = 2,
    Critical = 3,
}

#[export]
pub fn echo_priority(p: Priority) -> Priority {
    p
}

#[export]
pub fn priority_label(p: Priority) -> String {
    match p {
        Priority::Low => "low".to_string(),
        Priority::Medium => "medium".to_string(),
        Priority::High => "high".to_string(),
        Priority::Critical => "critical".to_string(),
    }
}

#[export]
pub fn is_high_priority(p: Priority) -> bool {
    matches!(p, Priority::High | Priority::Critical)
}

#[data]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum LogLevel {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
}

#[export]
pub fn echo_log_level(level: LogLevel) -> LogLevel {
    level
}

#[export]
pub fn should_log(level: LogLevel, min_level: LogLevel) -> bool {
    (level as u8) >= (min_level as u8)
}

#[export]
pub fn echo_vec_log_level(levels: Vec<LogLevel>) -> Vec<LogLevel> {
    levels
}

/// HTTP status codes with gapped, real-world discriminants.
///
/// Every variant's numeric value is meaningful on its own — `404` is a
/// wire-level protocol constant, not a label that could be renumbered.
/// A round-trip of `NotFound` must preserve `404`, not some positional
/// index that happens to name the same variant.
///
/// This ensures that in languages which expose numbered enum members
/// (C#, Kotlin, Swift, Java, etc.), the generated enum carries the Rust
/// discriminant — `Ok = 200, NotFound = 404, ServerError = 500` — so the
/// numeric value is usable directly in consuming code (e.g. comparing
/// against an HTTP response status) without routing through a separate
/// lookup table.
#[data]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u16)]
pub enum HttpCode {
    Ok = 200,
    NotFound = 404,
    ServerError = 500,
}

#[export]
pub fn echo_http_code(code: HttpCode) -> HttpCode {
    code
}

#[export]
pub fn http_code_not_found() -> HttpCode {
    HttpCode::NotFound
}

/// Signedness sentinel with a negative discriminant.
///
/// Rust allows negative values on any signed `#[repr(iN)]` enum, and
/// the numeric value must survive the crossing intact — flipping
/// `Negative` to `255` (an unsigned reinterpretation of the low byte)
/// changes the meaning of the value for every consumer.
///
/// This ensures that in languages which expose numbered enum members,
/// the backing type stays signed all the way through: the emitted C#
/// `enum : sbyte`, Swift `enum Sign: Int8`, Kotlin `value: Byte`,
/// Java `byte value`, etc. all preserve `-1` rather than truncating
/// it to its two's-complement unsigned form.
#[data]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i8)]
pub enum Sign {
    Negative = -1,
    Zero = 0,
    Positive = 1,
}

#[export]
pub fn echo_sign(s: Sign) -> Sign {
    s
}

#[export]
pub fn sign_negative() -> Sign {
    Sign::Negative
}
