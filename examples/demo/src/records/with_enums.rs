use boltffi::*;

use crate::enums::data_enum::Shape;
use crate::enums::repr_int::{LogLevel, Priority};

#[data]
#[derive(Clone, Debug, PartialEq)]
pub struct Task {
    pub title: String,
    pub priority: Priority,
    pub completed: bool,
}

#[export]
pub fn echo_task(task: Task) -> Task {
    task
}

#[export]
pub fn make_task(title: String, priority: Priority) -> Task {
    Task {
        title,
        priority,
        completed: false,
    }
}

#[export]
pub fn is_urgent(task: Task) -> bool {
    matches!(task.priority, Priority::High | Priority::Critical)
}

#[data]
#[derive(Clone, Debug, PartialEq)]
pub struct Notification {
    pub message: String,
    pub priority: Priority,
    pub read: bool,
}

#[export]
pub fn echo_notification(notification: Notification) -> Notification {
    notification
}

/// A `#[repr(C)]` wrapper around a data enum field.
///
/// Data enums have a variable-width on-the-wire representation — a
/// discriminant tag followed by the active variant's payload. A
/// record embedding one cannot be laid out as a flat C struct and
/// marshalled direct; it must ride the wire codec end to end.
///
/// This ensures that even when the host struct wears `#[repr(C)]`,
/// backends that have a "blittable if all fields are primitive"
/// fast path don't incorrectly admit a data-enum field into that
/// fast path.
#[data]
#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
pub struct Holder {
    pub shape: Shape,
}

#[export]
pub fn echo_holder(h: Holder) -> Holder {
    h
}

#[export]
pub fn make_triangle_holder() -> Holder {
    Holder {
        shape: Shape::Triangle {
            a: crate::records::blittable::Point { x: 0.0, y: 0.0 },
            b: crate::records::blittable::Point { x: 4.0, y: 0.0 },
            c: crate::records::blittable::Point { x: 0.0, y: 3.0 },
        },
    }
}

/// A compact header whose every field is a primitive or a C-style enum.
///
/// Rides the wire codec today because the `#[export]` macro's
/// blittability check admits only literal primitive fields, so a
/// `Priority` field (a C-style enum, same bit layout as its backing
/// `i32` but not a literal primitive from the macro's point of view)
/// bumps the struct onto the encoded path. A future change coordinating
/// the macro and each backend's blittable classifier can promote this
/// shape to direct P/Invoke with zero encode/decode cost.
#[data]
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TaskHeader {
    pub id: i64,
    pub priority: Priority,
    pub completed: bool,
}

#[export]
pub fn echo_task_header(header: TaskHeader) -> TaskHeader {
    header
}

#[export]
pub fn make_critical_task_header(id: i64) -> TaskHeader {
    TaskHeader {
        id,
        priority: Priority::Critical,
        completed: false,
    }
}

/// A `#[repr(C)]` struct mixing a `u8`-backed C-style enum with wider
/// primitives — same family as `TaskHeader`, but the enum's backing
/// type forces non-trivial alignment padding between fields.
///
/// Rides the wire codec today for the same reason `TaskHeader` does
/// (see its doc): the `#[export]` macro won't admit a C-style enum
/// field as a layout-compatible primitive. When that changes, this
/// struct is a useful shape to verify: padding between the `u8` enum
/// and the `u16` / `i64` fields has to line up on both sides of the
/// boundary, and non-`i32` enum backing types have historically been
/// the first place a new blittable path breaks.
#[data]
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LogEntry {
    pub timestamp: i64,
    pub level: LogLevel,
    pub code: u16,
}

#[export]
pub fn echo_log_entry(entry: LogEntry) -> LogEntry {
    entry
}

#[export]
pub fn make_error_log_entry(timestamp: i64, code: u16) -> LogEntry {
    LogEntry {
        timestamp,
        level: LogLevel::Error,
        code,
    }
}
