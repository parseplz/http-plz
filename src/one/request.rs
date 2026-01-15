use crate::{
    Request,
    one::{OneRequest, process_two_headers_and_body},
};
use body_plz::variants::Body;
use bytes::{BufMut, BytesMut};
use decompression_plz::DecompressTrait;
use header_plz::{
    MessageHead, OneHeaderMap, RequestLine,
    const_headers::{CONTENT_LENGTH, HOST},
};
use std::borrow::Cow;

use header_plz::{
    OneRequestLine,
    methods::{CONNECT, Method},
};

use super::OneOne;

// TODO: remove
pub const HTTP_1_1: &str = "HTTP/1.1";

// OneOne request methods
impl OneOne<OneRequestLine> {
    pub fn is_connect_request(&self) -> bool {
        matches!(self.message_head.infoline().method(), CONNECT)
    }

    pub fn method_as_string(&self) -> Cow<str> {
        String::from_utf8_lossy(self.message_head.infoline().method())
    }

    pub fn method_as_enum(&self) -> Method {
        self.message_head.infoline().method().into()
    }

    pub fn uri_as_string(&self) -> Cow<str> {
        self.message_head.infoline().uri_as_string()
    }
}

impl From<Request> for OneRequest {
    fn from(mut req: Request) -> OneRequest {
        let body = req.take_body();
        let trailer = req.take_trailers();

        let mut header_map =
            process_two_headers_and_body(req.headers, body.as_ref(), trailer);

        let (method, uri) = req.info_line.into_parts();

        let mut method_bytes = BytesMut::with_capacity(method.len() + 1);
        method_bytes.extend_from_slice(method.as_ref());
        method_bytes.put_u8(b' ');

        let info_line = OneRequestLine::new(
            method_bytes,
            uri.path_and_query().as_str().into(),
            HTTP_1_1.into(),
        );

        if let Some(host) = uri.authority()
            && !header_map.has_key(HOST)
        {
            header_map.insert(HOST, host);
        }

        let message_head = MessageHead::new(info_line, header_map);
        let mut one = OneRequest::new(message_head, req.body_headers);

        if body.is_some() {
            one.set_body(Body::Raw(body))
        }

        one
    }
}
