pub use riff_core::UnexpectedFfiCallbackError;
pub use riff_core::{CustomFfiConvertible, Data, FfiType, custom_ffi, data, error, export, name, skip};

#[doc(hidden)]
pub mod __private {
    pub use riff_core::{
        EventSubscription, FfiBuf, FfiStatus, RustFutureContinuationCallback, RustFutureHandle,
        StreamContinuationCallback, StreamPollResult, SubscriptionHandle, WaitResult, rustfuture,
        set_last_error, wire,
    };
}
