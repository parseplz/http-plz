use crate::{
    Request, Version,
    one::{OneRequest, process_two_headers_and_body},
};
use body_plz::variants::Body;
use bytes::{BufMut, BytesMut};
use decompression_plz::DecompressTrait;
use header_plz::{MessageHead, const_headers::HOST, uri::Uri};
use std::borrow::Cow;

use header_plz::{
    OneRequestLine,
    method::{CONNECT, Method},
};

use super::OneOne;

impl OneOne<OneRequestLine> {
    pub fn is_connect_request(&self) -> bool {
        matches!(self.message_head.infoline().method(), CONNECT)
    }

    pub fn method_as_string(&self) -> Cow<'_, str> {
        String::from_utf8_lossy(self.message_head.infoline().method())
    }

    pub fn method_as_enum(&self) -> Method {
        self.message_head.infoline().method().into()
    }

    pub fn uri_as_string(&self) -> Cow<'_, str> {
        self.message_head.infoline().uri_as_string()
    }
}

#[inline]
fn build_one_request_line_with_version(
    method: Method,
    uri: &Uri,
    version: Version,
) -> OneRequestLine {
    let mut method_bytes = BytesMut::with_capacity(method.len() + 1);
    method_bytes.extend_from_slice(method.as_ref());
    method_bytes.put_u8(b' ');
    OneRequestLine::new(
        method_bytes,
        uri.path_and_query().as_str().into(),
        version.for_request_line().into(),
    )
}

impl From<(Request, Version)> for OneRequest {
    fn from((mut req, version): (Request, Version)) -> Self {
        let body = req.take_body();
        let trailer = req.take_trailers();

        let mut header_map =
            process_two_headers_and_body(req.headers, body.as_ref(), trailer);

        let (method, uri, _) = req.info_line.into_parts();

        let info_line =
            build_one_request_line_with_version(method, &uri, version);

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
    use bytes::BytesMut;
    use header_plz::uri::path::PathAndQuery;
    use header_plz::{HeaderMap, OneInfoLine, const_headers::CONTENT_LENGTH};

    use super::*;

    #[test]
    fn test_build_one_request_line() {
        let uri = Uri::builder().path("/foo?a=1&b=2#23").build().unwrap();
        let line = build_one_request_line_with_version(
            Method::GET,
            &uri,
            Version::H11,
        );
        let input = "GET /foo?a=1&b=2#23 HTTP/1.1\r\n";
        let verify = OneRequestLine::try_build_infoline(input.into()).unwrap();
        assert_eq!(line, verify);
    }

    #[test]
    fn test_build_one_request_line_minimal() {
        let uri = Uri::default();
        let line = build_one_request_line_with_version(
            Method::GET,
            &uri,
            Version::H2,
        );
        let input = "GET / HTTP/2\r\n";
        let verify = OneRequestLine::try_build_infoline(input.into()).unwrap();
        assert_eq!(line, verify);
    }

    #[test]
    fn test_build_one_request_line_encoded_query() {
        let method = Method::GET;
        let path = PathAndQuery::from_shared(
            "/search?q=hello%20world&lang=en".into(),
        )
        .unwrap();
        let uri = Uri::builder().path(path).build().unwrap();
        let line =
            build_one_request_line_with_version(method, &uri, Version::H3);

        let input = "GET /search?q=hello%20world&lang=en HTTP/3\r\n";
        let verify = OneRequestLine::try_build_infoline(input.into()).unwrap();
        assert_eq!(line, verify);
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
