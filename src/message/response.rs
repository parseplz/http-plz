use header_plz::{ResponseLine, status::StatusCode};

use crate::{
    Version,
    message::{
        Message, builder::MessageBuilder, process_one_headers_and_body,
    },
    one::OneResponse,
};

pub type Response = Message<ResponseLine>;
pub type ResponseBuilder = MessageBuilder<ResponseLine>;

impl ResponseBuilder {
    pub fn status(mut self, status: StatusCode) -> Self {
        self.info_line = ResponseLine::new(status);
        self
    }

    pub fn build(self) -> Response {
        Response {
            info_line: self.info_line,
            headers: self.headers.unwrap_or_default(),
            body: self.body,
            trailers: self.trailer,
            body_headers: None,
        }
    }
}

impl Response {
    pub fn builder() -> ResponseBuilder {
        ResponseBuilder::default()
    }

    pub fn status(&self) -> &StatusCode {
        self.info_line.status()
    }

    pub fn into_one_rep(self) -> OneResponse {
        OneResponse::from((self, Version::H2))
    }
}

impl From<OneResponse> for Response {
    fn from(mut res: OneResponse) -> Self {
        let body = res.take_body();
        let (info_line, headers) = res.message_head.into_parts();
        let (headers, body) = process_one_headers_and_body(headers, body);
        let (_, raw_status, _) = info_line.into_parts();
        let status = StatusCode::from_bytes(raw_status.as_ref()).unwrap();
        let info_line = ResponseLine::new(status);
        Response::new(info_line, headers, body, None)
    }
}

#[cfg(test)]
mod tests {
    use bytes::BytesMut;
    use header_plz::HeaderMap;
    use header_plz::const_headers::CONTENT_LENGTH;

    use super::*;

    #[test]
    fn test_one_to_two_response_minimal() {
        let input = "HTTP/1.1 200 OK\r\n\r\n";
        let one = OneResponse::try_from(BytesMut::from(input)).unwrap();
        let result = Response::from(one);
        let verify = Response::builder()
            .status(StatusCode::from_u16(200).unwrap())
            .build();
        assert_eq!(result, verify);
    }

    #[test]
    fn test_one_to_two_response_custom_header() {
        let input = "HTTP/1.1 200 OK\r\n\
                     key: value\r\n\r\n";
        let one = OneResponse::try_from(BytesMut::from(input)).unwrap();
        let result = Response::from(one);
        let mut headers = HeaderMap::new();
        headers.insert("key", "value");
        let verify = Response::builder()
            .status(StatusCode::from_u16(200).unwrap())
            .headers(headers)
            .build();
        assert_eq!(result, verify);
    }

    #[test]
    fn test_one_to_two_response_multiple_headers() {
        let input = "HTTP/1.1 200 OK\r\n\
                     key1: value1\r\n\
                     key2: value2\r\n\
                     key3: value3\r\n\
                     key4: value4\r\n\
                     key5: value5\r\n\r\n";
        let one = OneResponse::try_from(BytesMut::from(input)).unwrap();
        let result = Response::from(one);
        let mut headers = HeaderMap::new();
        headers.insert("key1", "value1");
        headers.insert("key2", "value2");
        headers.insert("key3", "value3");
        headers.insert("key4", "value4");
        headers.insert("key5", "value5");
        let verify = Response::builder()
            .status(StatusCode::from_u16(200).unwrap())
            .headers(headers)
            .build();
        assert_eq!(result, verify);
    }

    #[test]
    fn test_one_to_two_response_body() {
        let input = "HTTP/1.1 205 OK\r\n\
                     Content-Length: 5\r\n\r\n\
                     Hello";
        let one = OneResponse::try_from(BytesMut::from(input)).unwrap();
        let result = Response::from(one);

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_LENGTH, 5.to_string().as_str());
        let verify = Response::builder()
            .status(StatusCode::from_u16(205).unwrap())
            .headers(headers)
            .body(BytesMut::from("Hello"))
            .build();
        assert_eq!(result, verify);
    }

    #[test]
    fn test_one_to_two_response_zero_content_length() {
        let input = "HTTP/1.1 205 OK\r\n\
                     Content-Length: 0\r\n\r\n";
        let one = OneResponse::try_from(BytesMut::from(input)).unwrap();
        let result = Response::from(one);
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_LENGTH, "0");
        let verify = Response::builder()
            .status(StatusCode::from_u16(205).unwrap())
            .headers(headers)
            .build();
        assert_eq!(result, verify);
    }

    #[test]
    fn test_one_to_two_response_large_body() {
        let large_body = "x".repeat(10000);
        let input = format!(
            "HTTP/1.1 205 OK\r\n\
             Content-Length: {}\r\n\r\n\
             {}",
            large_body.len(),
            large_body
        );
        let one =
            OneResponse::try_from(BytesMut::from(input.as_bytes())).unwrap();
        let result = Response::from(one);
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_LENGTH, large_body.len().to_string().as_str());
        let verify = Response::builder()
            .status(StatusCode::from_u16(205).unwrap())
            .headers(headers)
            .body(BytesMut::from(&large_body[..]))
            .build();
        assert_eq!(result, verify);
    }

    #[test]
    fn test_two_into_one_rep_response() {
        let expected = "HTTP/2 200 OK\r\n\
                     content-length: 5\r\n\r\n\
                     Hello";

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_LENGTH, "5");
        let resp = Response::builder()
            .status(StatusCode::from_u16(200).unwrap())
            .headers(headers)
            .body(BytesMut::from("Hello"))
            .build();
        assert_eq!(resp.into_one_rep().into_bytes(), expected);
    }
}
