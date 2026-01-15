use std::borrow::Cow;

use body_plz::variants::Body;
use bytes::{BufMut, BytesMut};
use decompression_plz::DecompressTrait;
use header_plz::{
    MessageHead, OneHeaderMap, OneResponseLine,
    abnf::CRLF,
    const_headers::CONTENT_LENGTH,
    status::{InvalidStatusCode, StatusCode},
};

use crate::{
    Response,
    one::{OneResponse, process_two_headers_and_body, request::HTTP_1_1},
};

use super::OneOne;

// OneOne response methods
impl OneOne<OneResponseLine> {
    pub fn status_code(&self) -> Result<StatusCode, InvalidStatusCode> {
        self.message_head.infoline().status()
    }
}

impl From<Response> for OneResponse {
    fn from(mut res: Response) -> Self {
        let body = res.take_body();
        let trailer = res.take_trailers();
        let mut header_map =
            process_two_headers_and_body(res.headers, body.as_ref(), trailer);

        let mut version: BytesMut = HTTP_1_1.into();
        version.put_u8(b' ');

        let status = res.info_line.into_parts();

        let mut reason = BytesMut::from(" ");
        let mut reason_str =
            StatusCode::canonical_reason(&status).unwrap_or_default();
        reason.extend_from_slice(reason_str.as_bytes());
        reason.extend_from_slice(CRLF.as_ref());

        let info_line =
            OneResponseLine::new(version, status.as_str().into(), reason);

        let message_head = MessageHead::new(info_line, header_map);
        let mut one = OneResponse::new(message_head, res.body_headers);

        if let Some(body) = body {
            one.set_body(Body::Raw(body))
        }
        one
    }
}

#[cfg(test)]
mod tests {
    use bytes::BytesMut;
    use header_plz::{HeaderMap, const_headers::CONTENT_LENGTH};

    use super::*;

    #[test]
    fn test_two_to_one_response() {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_LENGTH, 5.to_string().as_str());
        let verify = Response::builder()
            .status(StatusCode::from_u16(205).unwrap())
            .headers(headers)
            .body(BytesMut::from("Hello"))
            .build();

        let input = "HTTP/1.1 205 Reset Content\r\n\
                     Content-Length: 5\r\n\r\n\
                     Hello";
        let one = OneResponse::try_from(BytesMut::from(input)).unwrap();

        assert_eq!(one, verify.into());
    }
}
