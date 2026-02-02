use header_plz::{OneRequestLine, method::METHODS_WITH_BODY};

use super::*;

/* Steps:
 *      If method is in METHODS_WITH_BODY and no content length header is
 *      present, add Content-Length of zero.
 *
 *      https://github.com/curl/curl/issues/13380
 *      Adding "Content-Length: 0" is not mandatory.
 */

impl ParseMessage for OneOne<OneRequestLine> {
    fn parse(buf: BytesMut) -> Result<Self, BuildMessageError> {
        let mut req = OneOne::<OneRequestLine>::try_from(buf)?;
        if METHODS_WITH_BODY.contains(&req.method_enum()) {
            // If No content length header is present
            if req.has_header_key(CONTENT_LENGTH).is_none() {
                // Add Content-Length of zero
                req.insert_header(CONTENT_LENGTH, b"0");
            }
        }
        Ok(req)
    }
}

#[cfg(test)]
mod tests {
    use crate::OneRequest;

    use super::*;

    #[test]
    fn request_build_post_no_body_post() {
        let buf = BytesMut::from("POST / HTTP/1.1\r\n\r\n");
        let req = OneRequest::parse(buf).unwrap();
        let verify = "POST / HTTP/1.1\r\ncontent-length: 0\r\n\r\n";
        assert_eq!(req.into_bytes(), verify);
    }

    #[test]
    fn request_build_with_content_length_less() {
        let buf =
            BytesMut::from("POST / HTTP/1.1\r\ncontent-length: 10\r\n\r\na");
        let req = OneRequest::parse(buf).unwrap();
        let verify = "POST / HTTP/1.1\r\ncontent-length: 1\r\n\r\na";
        assert_eq!(req.into_bytes(), verify);
    }
}
