use crate::one::builder::OneBuilder;
use std::borrow::Cow;

use bytes::BufMut;
use decompression_plz::DecompressTrait;
use header_plz::{
    MessageHead, OneRequestLine, Uri, Version, abnf::SP, const_headers::HOST,
    method::Method, uri::InvalidUri,
};

use crate::Request;

use super::*;

pub type OneRequest = OneOne<OneRequestLine>;
pub type OneRequestBuilder = OneBuilder<(Option<Method>, Option<BytesMut>)>;

impl OneRequestBuilder {
    pub fn method(mut self, m: Method) -> Self {
        self.info_line.0 = Some(m);
        self
    }

    pub fn uri<U: AsRef<[u8]>>(mut self, u: U) -> Self {
        let mut b = BytesMut::new();
        b.extend_from_slice(u.as_ref());
        self.info_line.1 = Some(b);
        self
    }

    pub fn build(self) -> OneRequest {
        // method + space
        let mut method =
            BytesMut::from(self.info_line.0.unwrap_or(Method::GET).as_ref());
        method.put_u8(SP);
        let uri = self.info_line.1.unwrap_or_default();
        // space + version + CRLF
        let version = BytesMut::from(Version::H11.for_request_line());
        let info_line = OneRequestLine::new(method, uri, version);
        let mut request = OneRequest::new(
            MessageHead::new(info_line, self.headers.unwrap_or_default()),
            None,
        );
        if let Some(body) = self.body {
            request.set_body(body)
        }
        request
    }
}

impl OneRequest {
    pub fn builder() -> OneRequestBuilder {
        OneRequestBuilder::default()
    }
    pub fn is_connect_request(&self) -> bool {
        matches!(self.method_enum(), Method::CONNECT)
    }

    pub fn method_enum(&self) -> Method {
        self.message_head.infoline().method_enum()
    }

    pub fn uri_as_string(&self) -> Cow<'_, str> {
        self.message_head.infoline().uri_as_string()
    }

    pub fn uri(&self) -> Result<Uri, InvalidUri> {
        self.message_head.infoline().uri()
    }
}

impl From<(Request, Version)> for OneRequest {
    fn from((mut req, version): (Request, Version)) -> Self {
        let body = req.take_body();
        let trailer = req.take_trailers();

        let mut header_map =
            process_two_headers_and_body(req.headers, body.as_ref(), trailer);

        let (method, uri, _) = req.info_line.into_parts();

        let info_line = OneRequestLine::from((method, &uri, version));

        if let Some(host) = uri.authority()
            && !header_map.has_key(HOST)
        {
            header_map.insert(HOST, host);
        }

        let message_head = MessageHead::new(info_line, header_map);
        let mut one = OneRequest::new(message_head, req.body_headers);

        if let Some(body) = body
            && !body.is_empty()
        {
            one.set_body(Body::Raw(body))
        }

        one
    }
}

impl From<Request> for OneRequest {
    fn from(req: Request) -> OneRequest {
        OneRequest::from((req, Version::H11))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;
    use header_plz::{HeaderMap, const_headers::CONTENT_LENGTH};

    #[test]
    fn test_one_request_builder() {
        let mut headers = OneHeaderMap::new();
        headers.insert(CONTENT_LENGTH, "5".to_string());
        headers.insert("key", "value".to_string());
        let result = OneRequestBuilder::default()
            .method(Method::POST)
            .uri("/foo")
            .headers(headers)
            .body("dead body".into())
            .build()
            .into_bytes();
        let expected = "POST /foo HTTP/1.1\r\n\
                        content-length: 5\r\n\
                        key: value\r\n\r\n\
                        dead body";
        assert_eq!(result, BytesMut::from(expected));
    }

    #[test]
    fn test_two_to_one_request_minimal() {
        let verify =
            Request::builder().method(Method::GET).uri(Uri::default()).build();
        let input = "GET / HTTP/1.1\r\n\r\n";
        let one = OneRequest::try_from(BytesMut::from(input)).unwrap();
        assert_eq!(one, verify.into());
    }

    #[test]
    fn test_two_to_one_request_custom_header() {
        let mut headers = HeaderMap::new();
        headers.insert("key", "value");
        let verify =
            Request::builder().method(Method::GET).headers(headers).build();

        let input = "GET / HTTP/1.1\r\n\
                   key: value\r\n\r\n";
        let one = OneRequest::try_from(BytesMut::from(input)).unwrap();
        assert_eq!(one, verify.into());
    }

    #[test]
    fn test_two_to_one_request_multiple_headers() {
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
        assert_eq!(one, verify.into());
    }

    #[test]
    fn test_two_to_one_request_body() {
        let verify = Request::builder()
            .method(Method::POST)
            .uri(Uri::default())
            .body(BytesMut::from("Hello"))
            .build();
        let input = "POST / HTTP/1.1\r\n\
                   content-length: 5\r\n\r\n\
                   Hello";
        let mut one = OneRequest::try_from(BytesMut::from(input)).unwrap();
        one.body_headers = None;
        assert_eq!(one, verify.into());
    }

    #[test]
    fn test_two_to_one_request_zero_content_length() {
        let verify = Request::builder()
            .method(Method::POST)
            .uri(Uri::default())
            .body(BytesMut::from(""))
            .build();
        let input = "POST / HTTP/1.1\r\n\
                   content-length: 0\r\n\r\n";
        let mut one = OneRequest::try_from(BytesMut::from(input)).unwrap();
        one.body_headers = None;
        assert_eq!(one, verify.into());
    }

    #[test]
    fn test_two_to_one_request_large_body() {
        let large_body = "x".repeat(10000);
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_LENGTH, "10000".to_string().as_str());
        let verify = Request::builder()
            .method(Method::POST)
            .uri(Uri::default())
            .headers(headers)
            .body(BytesMut::from(&large_body[..]))
            .build();

        let input = format!(
            "POST / HTTP/1.1\r\n\
            content-length: 10000\r\n\r\n\
            {}",
            large_body
        );
        let mut one =
            OneRequest::try_from(BytesMut::from(input.as_bytes())).unwrap();
        one.body_headers = None;
        assert_eq!(one, verify.into());
    }
}
