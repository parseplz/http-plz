use header_plz::{HeaderMap, ResponseLine, status::StatusCode};

use crate::{
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

    pub fn build(mut self) -> Response {
        Response {
            info_line: self.info_line,
            headers: self.headers.unwrap_or(HeaderMap::new()),
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
        &self.info_line.status()
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
    use header_plz::const_headers::CONTENT_LENGTH;

    use super::*;

    #[test]
    fn test_one_to_two_response() {
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
}
