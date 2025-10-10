use askama::Template;

use crate::model::{Class, Method, Module, StreamMethod};

use super::templates::{
    AsyncMethodBodyTemplate, AsyncThrowingMethodBodyTemplate, StreamBodyTemplate,
    SyncMethodBodyTemplate, ThrowingMethodBodyTemplate,
};

pub struct BodyRenderer;

impl BodyRenderer {
    pub fn method(method: &Method, class: &Class, module: &Module) -> String {
        match (method.is_async, method.throws()) {
            (true, true) => AsyncThrowingMethodBodyTemplate::from_method(method, class, module)
                .render()
                .expect("async throwing method template failed"),
            (true, false) => AsyncMethodBodyTemplate::from_method(method, class, module)
                .render()
                .expect("async method template failed"),
            (false, true) => ThrowingMethodBodyTemplate::from_method(method, class, module)
                .render()
                .expect("throwing method template failed"),
            (false, false) => SyncMethodBodyTemplate::from_method(method, class, module)
                .render()
                .expect("sync method template failed"),
        }
    }

    pub fn stream(stream: &StreamMethod, class: &Class, module: &Module) -> String {
        StreamBodyTemplate::from_stream(stream, class, module)
            .render()
            .expect("stream body template failed")
    }
}
