use std::marker::PhantomData;

use super::StreamFormat;

pub(crate) trait StreamingMode {}

pub struct AsyncStream<F: StreamFormat> {
    _p: PhantomData<F>,
}
impl<F: StreamFormat> StreamingMode for AsyncStream<F> {}

pub struct SyncStream<F: StreamFormat> {
    _p: PhantomData<F>,
}
impl<F: StreamFormat> StreamingMode for SyncStream<F> {}

pub struct NoStream {}
impl StreamingMode for NoStream {}
