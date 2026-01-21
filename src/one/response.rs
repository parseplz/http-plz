use body_plz::variants::Body;
use bytes::{BufMut, BytesMut};
use decompression_plz::DecompressTrait;
use header_plz::{
    MessageHead, OneResponseLine,
    abnf::CRLF,
    status::{InvalidStatusCode, StatusCode},
};

use crate::{
    Response, Version,
    one::{OneResponse, process_two_headers_and_body},
};

use super::OneOne;

impl OneOne<OneResponseLine> {
    pub fn status_code(&self) -> Result<StatusCode, InvalidStatusCode> {
        self.message_head.infoline().status()
    }
}

#[inline]
fn build_one_response_line_with_version(
    status: StatusCode,
    version: Version,
) -> OneResponseLine {
    let reason_str = StatusCode::canonical_reason(&status).unwrap_or_default();
    let mut reason = BytesMut::with_capacity(1 + reason_str.len() + 2);
    reason.put_u8(b' ');
    reason.extend_from_slice(reason_str.as_bytes());
    reason.extend_from_slice(CRLF.as_ref());
    OneResponseLine::new(
        version.for_response_line().into(),
        status.as_str().into(),
        reason,
    )
}

impl From<(Response, Version)> for OneResponse {
    fn from((mut res, version): (Response, Version)) -> Self {
        let body = res.take_body();
        let trailer = res.take_trailers();
        let header_map =
            process_two_headers_and_body(res.headers, body.as_ref(), trailer);

        let status = res.info_line.into_parts();
        let info_line = build_one_response_line_with_version(status, version);

        let message_head = MessageHead::new(info_line, header_map);
        let mut one = OneResponse::new(message_head, res.body_headers);

        if let Some(body) = body
            && !body.is_empty()
        {
            one.set_body(Body::Raw(body))
        }
        one
    }
}

impl From<Response> for OneResponse {
    fn from(mut res: Response) -> Self {
        OneResponse::from((res, Version::H11))
    }
}

#[cfg(test)]
mod tests {
    use bytes::BytesMut;
    use header_plz::{HeaderMap, OneInfoLine, const_headers::CONTENT_LENGTH};

    use super::*;

    #[test]
    fn test_build_one_response_line() {
        let line =
            build_one_response_line_with_version(StatusCode::OK, Version::H11);
        let input = "HTTP/1.1 200 OK\r\n";
        let verify =
            OneResponseLine::try_build_infoline(input.into()).unwrap();
        assert_eq!(line, verify);
    }

    #[test]
    fn test_build_one_response_line_custom_status_no_reason() {
        let line = build_one_response_line_with_version(
            StatusCode::from_u16(599).unwrap(),
            Version::H2,
        );
        let input = "HTTP/2 599 \r\n";
        let verify =
            OneResponseLine::try_build_infoline(input.into()).unwrap();
        assert_eq!(line, verify);
    }

    #[test]
    fn test_two_to_one_response_minimal() {
        let verify = Response::builder().status(StatusCode::OK).build();
        let input = "HTTP/1.1 200 OK\r\n\r\n";
        let mut one = OneResponse::try_from(BytesMut::from(input)).unwrap();
        one.body_headers = None;
        assert_eq!(one, verify.into());
    }

    #[test]
    fn test_two_to_one_response_body() {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_LENGTH, 5.to_string().as_str());
        let verify = Response::builder()
            .status(StatusCode::from_u16(205).unwrap())
            .headers(headers)
            .body(BytesMut::from("Hello"))
            .build();

        let input = "HTTP/1.1 205 Reset Content\r\n\
                     content-length: 5\r\n\r\n\
                     Hello";
        let mut one = OneResponse::try_from(BytesMut::from(input)).unwrap();
        one.body_headers = None;
        assert_eq!(one, verify.into());
    }

    #[test]
    fn test_two_to_one_response_custom_header() {
        let mut headers = HeaderMap::new();
        headers.insert("key", "value");
        let verify = Response::builder()
            .status(StatusCode::from_u16(205).unwrap())
            .headers(headers)
            .build();

        let input = "HTTP/1.1 205 Reset Content\r\n\
                     key: value\r\n\r\n";
        let mut one = OneResponse::try_from(BytesMut::from(input)).unwrap();
        one.body_headers = None;
        assert_eq!(one, verify.into());
    }

    #[test]
    fn test_two_to_one_response_multiple_headers() {
        let mut headers = HeaderMap::new();
        headers.insert("key1", "value1");
        headers.insert("key2", "value2");
        headers.insert("key3", "value3");
        headers.insert("key4", "value4");
        headers.insert("key5", "value5");
        let verify = Response::builder()
            .status(StatusCode::from_u16(205).unwrap())
            .headers(headers)
            .build();

        let input = "HTTP/1.1 205 Reset Content\r\n\
                     key1: value1\r\n\
                     key2: value2\r\n\
                     key3: value3\r\n\
                     key4: value4\r\n\
                     key5: value5\r\n\r\n";

        let mut one = OneResponse::try_from(BytesMut::from(input)).unwrap();
        one.body_headers = None;
        assert_eq!(one, verify.into());
    }

    #[test]
    fn test_two_to_one_response_zero_content_length() {
        let verify = Response::builder()
            .status(StatusCode::from_u16(200).unwrap())
            .headers(HeaderMap::new())
            .body(BytesMut::from(""))
            .build();

        let input = "HTTP/1.1 200 OK\r\n\
                     content-length: 0\r\n\r\n";
        let mut one = OneResponse::try_from(BytesMut::from(input)).unwrap();
        one.body_headers = None;
        assert_eq!(one, verify.into());
    }

    #[test]
    fn test_two_to_one_response_large_body() {
        let large_body = "x".repeat(10000);
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_LENGTH, "10000".to_string().as_str());
        let verify = Response::builder()
            .status(StatusCode::from_u16(200).unwrap())
            .headers(HeaderMap::new())
            .body(BytesMut::from(&large_body[..]))
            .build();

        let input = format!(
            "HTTP/1.1 200 OK\r\n\
            content-length: 10000\r\n\r\n\
            {}",
            large_body
        );
        let mut one =
            OneResponse::try_from(BytesMut::from(input.as_bytes())).unwrap();
        one.body_headers = None;
        assert_eq!(one, verify.into());
    }
}
