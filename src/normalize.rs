use body_plz::variants::Body;
use bon::Builder;
use header_plz::Version;
use header_plz::{
    OneInfoLine, OneMessageHead,
    body_headers::parse::ParseBodyHeaders,
    const_headers::{CLOSE, CONNECTION, CONTENT_LENGTH, KEEP_ALIVE},
};

use crate::OneOne;

#[derive(Default, Builder, Debug)]
pub struct NormalizerOpts {
    #[builder(default = true)]
    close_connection: bool,
    #[builder(default = true)]
    fix_content_length: bool,
    #[builder(default = true)]
    proxy_connection: bool,
    #[builder(default = true)]
    sec_ws_ext: bool,
}

impl NormalizerOpts {
    pub fn normalize<T>(&self, msg: &mut OneOne<T>)
    where
        T: OneInfoLine + std::fmt::Debug,
        OneMessageHead<T>: ParseBodyHeaders,
    {
        if self.fix_content_length {
            Self::fix_content_length(msg)
        }
        if self.close_connection {
            Self::close_connection(msg)
        }
        if self.proxy_connection
            && let Some(pos) = msg.has_proxy_connection()
        {
            msg.remove_header_on_position(pos);
        }
        if self.sec_ws_ext {
            msg.remove_header_on_key(
                header_plz::const_headers::SEC_WEBSOCKET_EXTENSIONS,
            );
        }
    }

    fn fix_content_length<T>(msg: &mut OneOne<T>)
    where
        T: OneInfoLine + std::fmt::Debug,
        OneMessageHead<T>: ParseBodyHeaders,
    {
        let len = if let Some(Body::Raw(body)) = msg.body() {
            body.len()
        } else {
            // https://github.com/curl/curl/issues/13380
            // Adding "Content-Length: 0" is not mandatory.
            return;
        };
        let headers = msg.header_map_mut();
        if let Some(index) = headers.header_key_position(CONTENT_LENGTH) {
            headers.update_header_value_on_position(index, len.to_string());
        } else {
            headers.insert(CONTENT_LENGTH, len.to_string());
        }
    }

    fn close_connection<T>(msg: &mut OneOne<T>)
    where
        T: OneInfoLine + std::fmt::Debug,
        OneMessageHead<T>: ParseBodyHeaders,
    {
        use Version::*;
        if let Some(index) = msg
            .header_map_mut()
            .header_position((CONNECTION, KEEP_ALIVE.as_bytes()))
        {
            msg.header_map_mut().update_header_value_on_position(index, CLOSE);
        } else if let Some(version) = msg.version()
            && matches!(version, H09 | H10 | H11)
        {
            msg.header_map_mut().insert(CONNECTION, CLOSE)
        }
    }
}

#[cfg(test)]
mod test {
    use bytes::BytesMut;

    use crate::{NormalizerOpts, OneRequest};

    #[test]
    fn test_normalize_default() {
        let req = "POST / HTTP/1.1\r\n\
                   Connection: keep-alive\r\n\
                   Content-Length: 1\r\n\r\n\
                   hello world";
        let mut request = OneRequest::try_from(BytesMut::from(req)).unwrap();
        let normalizer = NormalizerOpts::builder().build();
        normalizer.normalize(&mut request);
        let verify = "POST / HTTP/1.1\r\n\
                   Connection: close\r\n\
                   Content-Length: 11\r\n\r\n\
                   hello world";
        assert_eq!(request.into_bytes(), verify);
    }

    #[test]
    fn test_normalize_close_connection() {
        let req = "POST / HTTP/1.1\r\n\
                   Connection: keep-alive\r\n\
                   Content-Length: 1\r\n\r\n\
                   hello world";
        let mut request = OneRequest::try_from(BytesMut::from(req)).unwrap();
        let normalizer =
            NormalizerOpts::builder().close_connection(false).build();
        normalizer.normalize(&mut request);
        let verify = "POST / HTTP/1.1\r\n\
                   Connection: keep-alive\r\n\
                   Content-Length: 11\r\n\r\n\
                   hello world";
        assert_eq!(request.into_bytes(), verify);
    }

    #[test]
    fn test_normalize_add_close_connection() {
        let req = "POST / HTTP/1.1\r\n\r\n";
        let mut request = OneRequest::try_from(BytesMut::from(req)).unwrap();
        let normalizer = NormalizerOpts::builder().build();
        normalizer.normalize(&mut request);
        let verify = "POST / HTTP/1.1\r\n\
                      connection: close\r\n\r\n";
        assert_eq!(request.into_bytes(), verify);
    }

    #[test]
    fn test_normalize_add_no_close_connection() {
        let req = "POST / HTTP/2\r\n\r\n";
        let mut request = OneRequest::try_from(BytesMut::from(req)).unwrap();
        let normalizer = NormalizerOpts::builder().build();
        normalizer.normalize(&mut request);
        let verify = "POST / HTTP/2\r\n\r\n";
        assert_eq!(request.into_bytes(), verify);
    }

    #[test]
    fn test_normalize_no_cl() {
        let req = "POST / HTTP/1.1\r\n\
                   Connection: keep-alive\r\n\
                   Content-Length: 1\r\n\r\n\
                   hello world";
        let mut request = OneRequest::try_from(BytesMut::from(req)).unwrap();
        let normalizer =
            NormalizerOpts::builder().fix_content_length(false).build();
        normalizer.normalize(&mut request);
        let verify = "POST / HTTP/1.1\r\n\
                   Connection: close\r\n\
                   Content-Length: 1\r\n\r\n\
                   hello world";
        assert_eq!(request.into_bytes(), verify);
    }

    #[test]
    fn test_normalize_add_cl() {
        let req = "POST / HTTP/1.1\r\n\
                   Connection: keep-alive\r\n\r\n\
                   hello world";
        let mut request = OneRequest::try_from(BytesMut::from(req)).unwrap();
        let normalizer = NormalizerOpts::builder().build();
        normalizer.normalize(&mut request);
        let verify = "POST / HTTP/1.1\r\n\
                   Connection: close\r\n\
                   content-length: 11\r\n\r\n\
                   hello world";
        assert_eq!(request.into_bytes(), verify);
    }

    #[test]
    fn test_normalize_no_proxy_connection() {
        let req = "POST / HTTP/1.1\r\n\
                   Proxy-Connection: keep-alive\r\n\
                   Content-Length: 1\r\n\r\n\
                   hello world";
        let mut request = OneRequest::try_from(BytesMut::from(req)).unwrap();
        let normalizer = NormalizerOpts::builder()
            .fix_content_length(false)
            .close_connection(false)
            .build();
        normalizer.normalize(&mut request);
        let verify = "POST / HTTP/1.1\r\n\
                      Content-Length: 1\r\n\r\n\
                      hello world";
        assert_eq!(request.into_bytes(), verify);
    }

    #[test]
    fn test_normalize_no_sec_ws_ext() {
        let req = "POST / HTTP/1.1\r\n\
                   sec-websocket-extensions: extensions\r\n\
                   Content-Length: 1\r\n\r\n\
                   hello world";
        let mut request = OneRequest::try_from(BytesMut::from(req)).unwrap();
        let normalizer = NormalizerOpts::builder()
            .fix_content_length(false)
            .close_connection(false)
            .build();
        normalizer.normalize(&mut request);
        let verify = "POST / HTTP/1.1\r\n\
                      Content-Length: 1\r\n\r\n\
                      hello world";
        assert_eq!(request.into_bytes(), verify);
    }
}
