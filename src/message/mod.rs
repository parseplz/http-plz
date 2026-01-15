use body_plz::variants::Body;
use bytes::BytesMut;
use decompression_plz::MultiDecompressErrorReason;
use decompression_plz::decompress;
use header_plz::OneHeaderMap;
use header_plz::{HeaderMap, body_headers::BodyHeader};
mod builder;
mod impl_decompress;
pub(crate) mod request;
pub(crate) mod response;

#[derive(Debug, PartialEq)]
pub struct Message<T> {
    pub(crate) info_line: T,
    pub(crate) headers: HeaderMap,
    pub(crate) body_headers: Option<BodyHeader>,
    body: Option<BytesMut>,
    trailers: Option<HeaderMap>,
}

impl<T> Message<T>
where
    T: std::fmt::Debug,
{
    pub fn new(
        info_line: T,
        headers: HeaderMap,
        body: Option<BytesMut>,
        trailer: Option<HeaderMap>,
    ) -> Message<T> {
        Self {
            info_line,
            headers,
            body,
            trailers: trailer,
            body_headers: None,
        }
    }

    // setters
    pub fn set_headers(&mut self, headers: HeaderMap) {
        self.headers = headers;
    }

    pub fn set_body(&mut self, body: BytesMut) {
        self.body = Some(body);
    }

    pub fn set_trailers(&mut self, trailer: HeaderMap) {
        self.trailers = Some(trailer);
    }

    // getters
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    pub fn body_as_ref(&self) -> Option<&BytesMut> {
        self.body.as_ref()
    }

    pub fn trailers(&self) -> Option<&HeaderMap> {
        self.trailers.as_ref()
    }

    // takers
    pub fn take_body(&mut self) -> Option<BytesMut> {
        self.body.take()
    }

    pub fn take_trailers(&mut self) -> Option<HeaderMap> {
        self.trailers.take()
    }

    pub fn try_decompress(
        &mut self,
        buf: &mut BytesMut,
    ) -> Result<(), MultiDecompressErrorReason> {
        decompress(self, buf)
    }
}

pub(crate) fn process_one_headers_and_body(
    headers: OneHeaderMap,
    body: Option<Body>,
) -> (HeaderMap, Option<BytesMut>) {
    let body = match body {
        Some(Body::Chunked(_)) => panic!(),
        Some(Body::Raw(b)) => Some(b),
        None => None,
    };
    (HeaderMap::from(headers), body)
}

#[cfg(test)]
mod tests {
    use crate::message::{request::Request, response::Response};
    use bytes::Bytes;
    use header_plz::{
        methods::Method,
        uri::{Uri, scheme::Scheme},
    };

    use super::*;

    fn build_headers() -> HeaderMap {
        let mut map = HeaderMap::new();
        for i in 0..2 {
            map.insert(
                Bytes::from(format!("header-{i}")),
                Bytes::from(format!("value-{i}")),
            );
        }
        map
    }

    #[test]
    fn test_request_builder_default() {
        let request = Request::builder().build();
        assert_eq!(*request.method(), Method::GET);
        assert!(request.scheme().is_none());
        assert_eq!(*request.path_and_query(), "/");
        assert_eq!(request.path(), "/");
        assert!(request.query().is_none());
        assert!(request.authority().is_none());
        assert!(request.headers().is_empty());
        assert!(request.body_as_ref().is_none());
        assert!(request.trailers().is_none());
    }

    #[test]
    fn test_request_builder() {
        let headers = build_headers();
        let body = BytesMut::from("dead body");
        let path = "/dead/end?user=sqli'--+-";
        let authority = "<script>alert(1)</script>";
        let uri = Uri::builder()
            .scheme("http")
            .authority(authority)
            .path(path)
            .build()
            .unwrap();
        let request = Request::builder()
            .method(Method::POST)
            .uri(uri)
            .headers(headers.clone())
            .body(body.clone())
            .trailer(headers.clone())
            .build();

        assert_eq!(*request.method(), Method::POST);
        assert_eq!(*request.scheme().unwrap(), Scheme::HTTP);
        assert_eq!(*request.path_and_query(), path);
        assert_eq!(request.path(), &path[..9]);
        assert_eq!(request.query().unwrap(), &path[10..]);
        assert_eq!(request.authority().unwrap(), authority);
        assert_eq!(request.headers(), &headers);
        assert_eq!(*request.body_as_ref().unwrap(), body);
        assert_eq!(request.trailers().unwrap(), &headers);
    }

    #[test]
    fn test_response_builder_default() {
        let response = Response::builder().build();
        assert_eq!(*response.status(), 200);
        assert!(response.headers().is_empty());
        assert!(response.body_as_ref().is_none());
        assert!(response.trailers().is_none());
    }

    #[test]
    fn test_response_builder() {
        let headers = build_headers();
        let body = BytesMut::from("dead body");
        let response = Response::builder()
            .status(100.try_into().unwrap())
            .headers(headers.clone())
            .body(body.clone())
            .trailer(headers.clone())
            .build();
        assert_eq!(*response.status(), 100);
        assert_eq!(response.headers(), &headers);
        assert_eq!(*response.body_as_ref().unwrap(), body);
        assert_eq!(response.trailers().unwrap(), &headers);
    }
}
