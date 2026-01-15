use bytes::BytesMut;
use header_plz::HeaderMap;

#[derive(Default)]
pub struct MessageBuilder<T> {
    pub(super) info_line: T,
    pub(super) headers: Option<HeaderMap>,
    pub(super) body: Option<BytesMut>,
    pub(super) trailer: Option<HeaderMap>,
}

impl<T> MessageBuilder<T> {
    pub fn headers(mut self, headers: HeaderMap) -> Self {
        self.headers = Some(headers);
        self
    }

    pub fn body(mut self, body: BytesMut) -> Self {
        self.body = Some(body);
        self
    }

    pub fn trailer(mut self, trailer: HeaderMap) -> Self {
        self.trailer = Some(trailer);
        self
    }
}
