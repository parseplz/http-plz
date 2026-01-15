use body_plz::variants::Body;
use bytes::Bytes;
use header_plz::{
    HeaderMap, RequestLine,
    methods::Method,
    uri::{Uri, path::PathAndQuery, scheme::Scheme},
};

use crate::{
    message::{
        Message, builder::MessageBuilder, process_one_headers_and_body,
    },
    one::OneRequest,
};

pub type Request = Message<RequestLine>;
pub type RequestBuilder = MessageBuilder<RequestLine>;

impl RequestBuilder {
    pub fn method(mut self, m: Method) -> Self {
        self.info_line.set_method(m);
        self
    }

    pub fn uri(mut self, u: Uri) -> Self {
        self.info_line.set_uri(u);
        self
    }

    pub fn extension(mut self, ext: Bytes) -> Self {
        self.info_line.set_extension(ext);
        self
    }

    pub fn build(mut self) -> Request {
        Request {
            info_line: self.info_line,
            headers: self.headers.unwrap_or(HeaderMap::new()),
            body: self.body,
            trailers: self.trailer,
            body_headers: None,
        }
    }
}

impl Request {
    pub fn builder() -> RequestBuilder {
        RequestBuilder::default()
    }

    pub fn method(&self) -> &Method {
        &self.info_line.method()
    }

    pub fn scheme(&self) -> Option<&Scheme> {
        self.info_line.uri().scheme()
    }

    pub fn path_and_query(&self) -> &PathAndQuery {
        &self.info_line.uri().path_and_query()
    }

    pub fn path(&self) -> &str {
        self.info_line.uri().path()
    }

    pub fn query(&self) -> Option<&str> {
        self.info_line.uri().query()
    }

    pub fn authority(&self) -> Option<&str> {
        self.info_line.uri().authority()
    }
}

impl From<OneRequest> for Request {
    fn from(mut req: OneRequest) -> Self {
        let body = req.take_body();
        let (info_line, headers) = req.message_head.into_parts();
        let (headers, body) = process_one_headers_and_body(headers, body);
        let (raw_method, raw_uri, _) = info_line.into_parts();

        let method = Method::from(raw_method.trim_ascii_end());
        let uri = Uri::builder().path(raw_uri.as_ref()).build().unwrap();
        let info_line = RequestLine::new(method, uri);
        Request::new(info_line, headers, body, None)
    }
}
