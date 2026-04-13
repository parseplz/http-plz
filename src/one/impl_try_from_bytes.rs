use body_plz::variants::Body;
use bytes::BytesMut;
use decompression_plz::DecompressTrait;
use header_plz::{
    OneInfoLine, OneMessageHead, abnf::HEADER_DELIMITER,
    body_headers::parse::ParseBodyHeaders,
    message_head::error::MessageHeadError,
};
use thiserror::Error;

use crate::one::OneOne;

#[derive(Debug, Error)]
#[error("message parse err| {}", self.kind)]
pub struct MsgParseErr {
    bytes: BytesMut,
    kind: MsgParseErrKind,
}

impl MsgParseErr {
    pub fn crlf(bytes: BytesMut) -> Self {
        Self {
            bytes,
            kind: MsgParseErrKind::UnableToFindCRLF,
        }
    }

    pub fn message_head(bytes: BytesMut, err: MessageHeadError) -> Self {
        Self {
            bytes,
            kind: MsgParseErrKind::MessageHead(err),
        }
    }

    pub fn into_bytes(self) -> BytesMut {
        use MsgParseErrKind::*;
        match self.kind {
            UnableToFindCRLF => self.bytes,
            MessageHead(err) => {
                let mut head = err.into_bytes();
                head.unsplit(self.bytes);
                head
            }
        }
    }
}

#[derive(Debug, Error)]
pub enum MsgParseErrKind {
    #[error("Failed to FindCRLF")]
    UnableToFindCRLF,
    #[error("Failed to DecodeHTTP| {0}")]
    MessageHead(MessageHeadError),
}

impl<T> TryFrom<BytesMut> for OneOne<T>
where
    T: OneInfoLine + std::fmt::Debug,
    OneMessageHead<T>: ParseBodyHeaders,
{
    type Error = MsgParseErr;

    fn try_from(mut buf: BytesMut) -> Result<Self, Self::Error> {
        let Some(index) =
            buf.windows(4).position(|window| window == HEADER_DELIMITER)
        else {
            return Err(MsgParseErr::crlf(buf));
        };
        let message_head = buf.split_to(index + HEADER_DELIMITER.len());
        let mut one = match OneOne::try_from_message_head_buf(message_head) {
            Ok(one) => one,
            Err(e) => return Err(MsgParseErr::message_head(buf, e)),
        };
        if !buf.is_empty() {
            one.set_body(Body::Raw(buf));
        }
        Ok(one)
    }
}

impl<T> TryFrom<&[u8]> for OneOne<T>
where
    T: OneInfoLine + std::fmt::Debug,
    OneMessageHead<T>: ParseBodyHeaders,
{
    type Error = MsgParseErr;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Self::try_from(BytesMut::from(value))
    }
}

#[cfg(test)]
mod tests {
    use crate::{OneRequest, OneResponse};
    use rstest::rstest;

    use super::*;

    // request
    #[test]
    fn test_request_try_from_bytes_no_body() {
        let req = "POST / HTTP/1.1\r\n\r\n";
        let input = BytesMut::from(req);
        let result = OneRequest::try_from(input).unwrap();
        assert_eq!(result.into_bytes(), req);
    }

    #[test]
    fn test_request_try_from_bytes_no_cl_body() {
        let req = "POST / HTTP/1.1\r\n\r\nHello";
        let input = BytesMut::from(req);
        let result = OneRequest::try_from(input).unwrap();
        assert_eq!(result.into_bytes(), req);
    }

    #[test]
    fn test_request_try_from_bytes_cl_body() {
        let req = "POST / HTTP/1.1\r\nContent-Length: 10\r\n\r\na";
        let input = BytesMut::from(req);
        let result = OneRequest::try_from(input).unwrap();
        assert_eq!(result.into_bytes(), req);
    }

    // response
    #[test]
    fn test_response_try_from_bytes_no_body() {
        let res = "HTTP/1.1 200 OK\r\n\r\n";
        let input = BytesMut::from(res);
        let result = OneResponse::try_from(input).unwrap();
        assert_eq!(result.into_bytes(), res);
    }

    #[test]
    fn test_response_try_from_bytes_no_cl_body() {
        let res = "HTTP/1.1 200 OK\r\n\r\nHello";
        let input = BytesMut::from(res);
        let result = OneResponse::try_from(input).unwrap();
        assert_eq!(result.into_bytes(), res);
    }

    #[test]
    fn test_response_try_from_bytes_cl_body() {
        let res = "HTTP/1.1 200 OK\r\nContent-Length: 10\r\n\r\na";
        let input = BytesMut::from(res);
        let result = OneResponse::try_from(input).unwrap();
        assert_eq!(result.into_bytes(), res);
    }

    // error
    #[rstest]
    #[case("nocrlf")]
    #[case("nofirstows\r\n\r\n")] // no first ows
    #[case("no secondows\r\n\r\n")] // no second ows
    #[case("no info line")] // no info line
    fn test_try_from_bytes_err(#[case] input: &str) {
        let result = OneRequest::try_from(BytesMut::from(input)).unwrap_err();
        dbg!(&result);
        assert_eq!(result.into_bytes(), input);
    }
}
