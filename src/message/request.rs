use bytes::Bytes;
use header_plz::{
    Method, RequestLine,
    uri::{Uri, path::PathAndQuery, scheme::Scheme},
};

use crate::{
    Version,
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

    pub fn build(self) -> Request {
        Request {
            info_line: self.info_line,
            headers: self.headers.unwrap_or_default(),
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
        self.info_line.method()
    }

    pub fn scheme(&self) -> Option<&Scheme> {
        self.info_line.uri().scheme()
    }

    pub fn path_and_query(&self) -> &PathAndQuery {
        self.info_line.uri().path_and_query()
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

    pub fn into_one_rep(self) -> OneRequest {
        OneRequest::from((self, Version::H2))
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

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;
    use decompression_plz::DecompressTrait;
    use header_plz::{HeaderMap, const_headers::CONTENT_LENGTH};

    #[test]
    fn test_one_to_two_request_minimal() {
        let verify =
            Request::builder().method(Method::GET).uri(Uri::default()).build();
        let input = "GET / HTTP/1.1\r\n\r\n";
        let one = OneRequest::try_from(BytesMut::from(input)).unwrap();
        assert_eq!(Request::from(one), verify);
    }

    #[test]
    fn test_one_to_two_request_custom_header() {
        let mut headers = HeaderMap::new();
        headers.insert("key", "value");
        let verify =
            Request::builder().method(Method::GET).headers(headers).build();
        let input = "GET / HTTP/1.1\r\n\
                     key: value\r\n\r\n";
        let one = OneRequest::try_from(BytesMut::from(input)).unwrap();
        assert_eq!(Request::from(one), verify);
    }

    #[test]
    fn test_one_to_two_request_multiple_headers() {
        let mut headers = HeaderMap::new();
        headers.insert("key1", "value1");
        headers.insert("key2", "value2");
        headers.insert("key3", "value3");
        headers.insert("key4", "value4");
        headers.insert("key5", "value5");
        let verify =
            Request::builder().method(Method::GET).headers(headers).build();
        let input = "GET / HTTP/1.1\r\n\
                     key1: value1\r\n\
                     key2: value2\r\n\
                     key3: value3\r\n\
                     key4: value4\r\n\
                     key5: value5\r\n\r\n";
        let one = OneRequest::try_from(BytesMut::from(input)).unwrap();
        assert_eq!(Request::from(one), verify);
    }

    #[test]
    fn test_one_to_two_request_body() {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_LENGTH, b"5");
        let verify = Request::builder()
            .method(Method::POST)
            .uri(Uri::default())
            .headers(headers)
            .body(BytesMut::from("Hello"))
            .build();
        let input = "POST / HTTP/1.1\r\n\
                     Content-Length: 5\r\n\r\n\
                     Hello";
        let one = OneRequest::try_from(BytesMut::from(input)).unwrap();
        assert_eq!(Request::from(one), verify);
    }

    #[test]
    fn test_one_to_two_request_zero_content_length() {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_LENGTH, b"0");
        let mut verify = Request::builder()
            .method(Method::POST)
            .uri(Uri::default())
            .headers(headers)
            .body(BytesMut::from(""))
            .build();
        verify.take_body();
        let input = "POST / HTTP/1.1\r\n\
                     Content-Length: 0\r\n\r\n";
        let mut one = OneRequest::try_from(BytesMut::from(input)).unwrap();
        assert_eq!(Request::from(one), verify);
    }

    #[test]
    fn test_one_to_two_request_large_body() {
        let large_body = "x".repeat(10000);
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_LENGTH, "10000");
        let verify = Request::builder()
            .method(Method::POST)
            .uri(Uri::default())
            .headers(headers)
            .body(BytesMut::from(&large_body[..]))
            .build();

        let input = format!(
            "POST / HTTP/1.1\r\n\
            Content-Length: 10000\r\n\r\n\
            {}",
            large_body
        );
        let one =
            OneRequest::try_from(BytesMut::from(input.as_bytes())).unwrap();
        assert_eq!(Request::from(one), verify);
    }
    #[test]
    fn test_two_into_one_rep_request() {
        let expected = "POST / HTTP/2\r\n\
                        content-length: 5\r\n\r\n\
                        Hello";

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_LENGTH, "5");
        let req = Request::builder()
            .method(Method::POST)
            .uri(Uri::default())
            .headers(headers)
            .body(BytesMut::from("Hello"))
            .build();
        assert_eq!(req.into_one_rep().into_bytes(), expected);
    }
}
