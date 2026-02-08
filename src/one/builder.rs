use body_plz::variants::Body;
use bytes::BytesMut;
use header_plz::OneHeaderMap;

#[derive(Clone, Debug, Default)]
pub struct OneBuilder<T> {
    pub(super) info_line: T,
    pub(super) headers: Option<OneHeaderMap>,
    pub(super) body: Option<Body>,
}

impl<T> OneBuilder<T> {
    pub fn headers(mut self, headers: OneHeaderMap) -> Self {
        self.headers = Some(headers);
        self
    }

    pub fn body(mut self, body: BytesMut) -> Self {
        self.body = Some(Body::Raw(body));
        self
    }
}
