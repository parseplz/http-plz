use body_plz::variants::Body;
use bytes::BytesMut;
use decompression_plz::DecompressTrait;
use header_plz::{
    OneInfoLine, OneMessageHead, abnf::HEADER_DELIMITER,
    body_headers::parse::ParseBodyHeaders, const_headers::CONTENT_LENGTH,
};

use crate::one::{OneOne, parse::error::BuildMessageError};

/* Description:
 *      Build oneone from BytesMut.
 *      Used when request/response is modified in interceptor. No chunked body,
 *      as chunked is converted to Content-Length by convert_one_dot_one()
 *
 * Steps:
 *      1. Find HEADER_DELIMITER (2 * CRLF) in buf.
 *      2. Split buf at index.
 *      3. Build OneOne.
 *      4. if buf !empty, i.e. body is present.
 *          a. set body.
 *          b. If content-length header is present, update content-length by calling
 *          update_content_length().
 *          c. Else add, new content-length header.
 *
 * Error:
 *      BuildFrameError::UnableToFindCRLF  [1]
 *      BuildFrameError::HttpDecodeError   [3]
 */

impl<T> TryFrom<BytesMut> for OneOne<T>
where
    T: OneInfoLine + std::fmt::Debug,
    OneMessageHead<T>: ParseBodyHeaders,
{
    type Error = BuildMessageError;

    fn try_from(mut buf: BytesMut) -> Result<Self, Self::Error> {
        let index = buf
            .windows(4)
            .position(|window| window == HEADER_DELIMITER)
            .ok_or(BuildMessageError::UnableToFindCRLF)?;
        let message_head = buf.split_to(index + HEADER_DELIMITER.len());
        let mut one = OneOne::try_from_message_head_buf(message_head)?;
        if !buf.is_empty() {
            let len = buf.len().to_string();
            one.set_body(Body::Raw(buf));
            if !one.update_header_value_on_key(CONTENT_LENGTH, len.as_bytes())
            {
                one.insert_header(CONTENT_LENGTH, len.as_bytes());
            }
        }
        Ok(one)
    }
}

#[cfg(test)]
mod tests {
    use crate::{OneRequest, OneResponse};

    use super::*;

    #[test]
    fn test_request_try_from_bytes_content_length_no_body() {
        let req = "POST / HTTP/1.1\r\n\r\n";
        let input = BytesMut::from(req);
        let result = OneRequest::try_from(input).unwrap();
        assert_eq!(result.into_bytes(), req);
    }

    #[test]
    fn test_request_try_from_bytes_content_length_no_cl() {
        let input = BytesMut::from("POST / HTTP/1.1\r\n\r\nHello");
        let result = OneRequest::try_from(input).unwrap();
        let verify = BytesMut::from(
            "POST / HTTP/1.1\r\ncontent-length: 5\r\n\r\nHello",
        );
        assert_eq!(result.into_bytes(), verify);
    }

    #[test]
    fn test_request_try_from_bytes_content_length_less() {
        let input =
            BytesMut::from("POST / HTTP/1.1\r\nContent-Length: 10\r\n\r\na");
        let result = OneRequest::try_from(input).unwrap();
        let verify =
            BytesMut::from("POST / HTTP/1.1\r\nContent-Length: 1\r\n\r\na");
        assert_eq!(result.into_bytes(), verify);
    }

    #[test]
    fn test_request_try_from_bytes_content_length_more() {
        let input = BytesMut::from(
            "POST / HTTP/1.1\r\nContent-Length: 0\r\n\r\nHello",
        );
        let result = OneRequest::try_from(input).unwrap();
        let verify = BytesMut::from(
            "POST / HTTP/1.1\r\nContent-Length: 5\r\n\r\nHello",
        );
        assert_eq!(result.into_bytes(), verify);
    }

    #[test]
    fn test_response_try_from_bytes_content_length_no_cl() {
        let input = BytesMut::from("HTTP/1.1 200 OK\r\n\r\nHello");
        let result = OneResponse::try_from(input).unwrap();
        let verify = BytesMut::from(
            "HTTP/1.1 200 OK\r\ncontent-length: 5\r\n\r\nHello",
        );
        assert_eq!(result.into_bytes(), verify);
    }

    #[test]
    fn test_response_try_from_bytes_content_length_less() {
        let input =
            BytesMut::from("HTTP/1.1 200 OK\r\nContent-Length: 10\r\n\r\na");
        let result = OneResponse::try_from(input).unwrap();
        let verify =
            BytesMut::from("HTTP/1.1 200 OK\r\nContent-Length: 1\r\n\r\na");
        assert_eq!(result.into_bytes(), verify);
    }

    #[test]
    fn test_response_try_from_bytes_content_length_more() {
        let input = BytesMut::from(
            "HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\nHello",
        );
        let result = OneResponse::try_from(input).unwrap();
        let verify = BytesMut::from(
            "HTTP/1.1 200 OK\r\nContent-Length: 5\r\n\r\nHello",
        );
        assert_eq!(result.into_bytes(), verify);
    }
}
