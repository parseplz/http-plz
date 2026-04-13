use decompression_plz::DecompressTrait;
use header_plz::abnf::CRLF;
use header_plz::status::InvalidStatusCode;
use header_plz::{MessageHead, OneResponseLine, StatusCode, Version};

use crate::one::builder::OneBuilder;
use crate::{Response, one::process_two_headers_and_body};

use super::OneOne;
use super::*;

pub type OneResponse = OneOne<OneResponseLine>;
pub type OneResponseBuilder = OneBuilder<u16>;

impl OneResponseBuilder {
    pub fn status(mut self, status: u16) -> Self {
        self.info_line = status;
        self
    }

    pub fn build(self) -> Result<OneResponse, InvalidStatusCode> {
        let scode = StatusCode::from_u16(self.info_line)?;
        let info_line = OneResponseLine::from(scode);
        let mut response = OneResponse::new(
            MessageHead::new(
                info_line,
                self.headers.unwrap_or_default(),
                CRLF.into(),
            ),
            None,
        );
        if let Some(body) = self.body {
            response.set_body(body)
        }

        Ok(response)
    }
}

impl OneResponse {
    pub fn builder() -> OneResponseBuilder {
        OneResponseBuilder::default()
    }

    pub fn status_code(&self) -> Result<StatusCode, InvalidStatusCode> {
        self.message_head.info_line().status()
    }

    pub fn set_status(&mut self, status: u16) {
        self.message_head.info_line_mut().set_status(status);
    }
}

impl From<(Response, Version)> for OneResponse {
    fn from((mut res, version): (Response, Version)) -> Self {
        let body = res.take_body();
        let trailer = res.take_trailers();
        let header_map =
            process_two_headers_and_body(res.headers, body.as_ref(), trailer);

        let status = res.info_line.into_parts();
        let info_line = OneResponseLine::from((status, version));

        let message_head =
            MessageHead::new(info_line, header_map, CRLF.into());
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
    fn from(res: Response) -> Self {
        OneResponse::from((res, Version::H11))
    }
}

#[cfg(test)]
mod tests {
    use bytes::BytesMut;
    use header_plz::{HeaderMap, const_headers::CONTENT_LENGTH};

    use super::*;

    #[test]
    fn test_one_response_builder() {
        let mut headers = OneHeaderMap::new();
        headers.insert(CONTENT_LENGTH, "5".to_string());
        headers.insert("key", "value".to_string());
        let result = OneResponseBuilder::default()
            .status(200)
            .headers(headers)
            .body(BytesMut::from("dead body"))
            .build()
            .unwrap()
            .into_bytes();
        let expected = "HTTP/1.1 200 OK\r\n\
                        content-length: 5\r\n\
                        key: value\r\n\r\n\
                        dead body";
        assert_eq!(result, BytesMut::from(expected));
    }

    #[test]
    fn test_one_response_builder_wrong_status() {
        let result = OneResponseBuilder::default().status(1000).build();
        assert!(result.is_err());
    }

    #[test]
    fn test_two_to_one_response_minimal() {
        let verify = Response::builder().status(200).build().unwrap();
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
            .status(205)
            .headers(headers)
            .body(BytesMut::from("Hello"))
            .build()
            .unwrap();

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
        let verify =
            Response::builder().status(205).headers(headers).build().unwrap();

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
        let verify =
            Response::builder().status(205).headers(headers).build().unwrap();

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
            .status(200)
            .headers(HeaderMap::new())
            .body(BytesMut::from(""))
            .build()
            .unwrap();

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
            .status(200)
            .headers(HeaderMap::new())
            .body(BytesMut::from(&large_body[..]))
            .build()
            .unwrap();

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
