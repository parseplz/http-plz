use body_plz::variants::{Body, chunked::ChunkType};
use bytes::{Buf, BytesMut};
use decompression_plz::{MultiDecompressErrorReason, decompress};
use header_plz::{
    HeaderMap, OneHeaderMap, OneInfoLine, OneMessageHead, Version,
    body_headers::{
        BodyHeader, parse::ParseBodyHeaders, transfer_types::TransferType,
    },
    const_headers::{
        CONNECTION, CONTENT_LENGTH, KEEP_ALIVE, PROXY_CONNECTION, TRAILER,
    },
    message_head::error::MessageHeadError,
};
pub mod impl_decompress;

pub mod builder;
pub mod impl_try_from_bytes;
pub mod request;
pub mod response;

#[derive(Clone, Debug, PartialEq)]
pub struct OneOne<T>
where
    T: OneInfoLine,
{
    pub(crate) message_head: OneMessageHead<T>,
    body_headers: Option<BodyHeader>,
    body: Option<Body>,
    extra_body: Option<BytesMut>,
}

impl<T> OneOne<T>
where
    T: OneInfoLine + std::fmt::Debug,
    OneMessageHead<T>: ParseBodyHeaders,
{
    pub fn new(
        message_head: OneMessageHead<T>,
        body_headers: Option<BodyHeader>,
    ) -> Self {
        OneOne {
            message_head,
            body_headers,
            body: None,
            extra_body: None,
        }
    }

    // parse from message_head
    pub fn try_from_message_head_buf(
        buf: BytesMut,
    ) -> Result<Self, MessageHeadError> {
        let message_head = OneMessageHead::<T>::try_from(buf)?;
        let body_headers = message_head.parse_body_headers();
        Ok(OneOne::<T>::new(message_head, body_headers))
    }

    // version
    pub fn version(&self) -> Option<Version> {
        self.message_head.version()
    }

    // Header Related methods
    pub fn message_head(&self) -> &OneMessageHead<T> {
        &self.message_head
    }

    pub fn message_head_mut(&mut self) -> &mut OneMessageHead<T> {
        &mut self.message_head
    }

    pub fn header_map(&self) -> &OneHeaderMap {
        self.message_head.header_map()
    }

    pub fn header_map_mut(&mut self) -> &mut OneHeaderMap {
        self.message_head.header_map_mut()
    }

    pub fn has_header_key(&self, key: &[u8]) -> Option<usize> {
        self.message_head.header_map().header_key_position(key)
    }

    pub fn insert_header(&mut self, key: &[u8], value: &[u8]) {
        self.message_head.header_map_mut().insert(key, value);
    }

    pub fn update_header_value_on_position(
        &mut self,
        pos: usize,
        value: &str,
    ) {
        self.message_head
            .header_map_mut()
            .update_header_value_on_position(pos, value);
    }

    pub fn update_header_value_on_key(
        &mut self,
        key: &[u8],
        value: &[u8],
    ) -> bool {
        self.message_head
            .header_map_mut()
            .update_header_value_on_key(key, value)
    }

    pub fn remove_header_on_position(&mut self, pos: usize) {
        self.message_head.header_map_mut().remove_header_on_position(pos);
    }

    pub fn remove_header_on_key(&mut self, key: &[u8]) -> bool {
        self.message_head.header_map_mut().remove_header_on_key(key)
    }

    pub fn has_trailers(&self) -> bool {
        self.message_head.header_map().header_key_position(TRAILER).is_some()
    }

    pub fn set_transfer_type_close(&mut self) {
        self.body_headers.get_or_insert_with(Default::default).transfer_type =
            Some(TransferType::Close);
    }

    // Body Headers Related
    pub fn parse_body_headers(&mut self) {
        self.body_headers = self.message_head.parse_body_headers();
    }

    pub fn body(&self) -> &Option<Body> {
        &self.body
    }

    pub fn body_headers(&self) -> &Option<BodyHeader> {
        &self.body_headers
    }

    pub fn body_as_mut(&mut self) -> Option<&mut Body> {
        self.body.as_mut()
    }

    pub fn set_extra_body(&mut self, extra_body: BytesMut) {
        self.extra_body = Some(extra_body);
    }

    pub fn take_body(&mut self) -> Option<Body> {
        self.body.take()
    }

    // checkers
    pub fn has_connection_keep_alive(&self) -> Option<usize> {
        self.message_head
            .header_map()
            .header_position((CONNECTION, KEEP_ALIVE.as_bytes()))
    }

    pub fn has_proxy_connection(&self) -> Option<usize> {
        self.message_head.header_map().header_key_position(PROXY_CONNECTION)
    }

    pub fn try_decompress(
        &mut self,
        buf: &mut BytesMut,
    ) -> Result<(), MultiDecompressErrorReason> {
        decompress(self, buf)
    }

    pub fn into_bytes(self) -> BytesMut {
        let mut header = self.message_head.into_bytes();
        if let Some(body) = self.body {
            let body = match body {
                Body::Raw(body) => body,
                Body::Chunked(items) => {
                    partial_chunked_to_raw(items).unwrap_or_default()
                }
            };
            header.unsplit(body);
        }
        header
    }

    pub fn as_chain(&self) -> impl Buf {
        let body = match self.body.as_ref() {
            Some(Body::Raw(b)) => b.as_ref(),
            _ => &[],
        };
        self.message_head.as_chain().chain(body)
    }
}

fn partial_chunked_to_raw(vec_body: Vec<ChunkType>) -> Option<BytesMut> {
    let mut iter = vec_body.into_iter().map(|c| c.into_bytes());
    let mut body = iter.next()?;
    for chunk in iter {
        body.unsplit(chunk);
    }
    Some(body)
}

pub(crate) fn process_two_headers_and_body(
    headers: HeaderMap,
    body: Option<&BytesMut>,
    trailer: Option<HeaderMap>,
) -> OneHeaderMap {
    let mut header_map: OneHeaderMap = headers.into();

    // Merge trailers
    if let Some(trailers) = trailer {
        header_map.extend(OneHeaderMap::from(trailers));
    }

    // Add content-length if needed
    if let Some(body) = body
        && !header_map.has_key(CONTENT_LENGTH)
    {
        header_map.insert(CONTENT_LENGTH, body.len().to_string().as_bytes());
    }

    header_map
}

#[cfg(test)]
mod tests {

    use header_plz::Method;

    use crate::one::{
        request::OneRequestBuilder, response::OneResponseBuilder,
    };

    use super::*;

    fn build_header_map() -> OneHeaderMap {
        let mut headers = OneHeaderMap::new();
        headers.insert("key1", "value1");
        headers.insert("key2", "value2");
        headers.insert("key3", "value3");
        headers.insert("key4", "value4");
        headers.insert("key5", "value5");
        headers
    }

    #[test]
    fn test_one_req_as_chain_no_body() {
        let req = OneRequestBuilder::default()
            .method(Method::GET)
            .uri("/foo")
            .headers(build_header_map())
            .build();
        let mut chain = req.as_chain();
        let result = chain.copy_to_bytes(chain.remaining());
        let expected = "GET /foo HTTP/1.1\r\n\
                        key1: value1\r\n\
                        key2: value2\r\n\
                        key3: value3\r\n\
                        key4: value4\r\n\
                        key5: value5\r\n\r\n";
        assert_eq!(result, expected);
        drop(chain);
        assert_eq!(req.into_bytes(), expected);
    }

    #[test]
    fn test_one_req_as_chain_with_body() {
        let req = OneRequestBuilder::default()
            .method(Method::POST)
            .uri("/foo")
            .headers(build_header_map())
            .body("dead body".into())
            .build();
        let mut chain = req.as_chain();
        let result = chain.copy_to_bytes(chain.remaining());
        let expected = "POST /foo HTTP/1.1\r\n\
                        key1: value1\r\n\
                        key2: value2\r\n\
                        key3: value3\r\n\
                        key4: value4\r\n\
                        key5: value5\r\n\r\n\
                        dead body";
        assert_eq!(result, expected);
        drop(chain);
        assert_eq!(req.into_bytes(), expected);
    }

    #[test]
    fn test_one_res_as_chain_no_body() {
        let res = OneResponseBuilder::default()
            .status(200)
            .headers(build_header_map())
            .build()
            .unwrap();
        let mut chain = res.as_chain();
        let result = chain.copy_to_bytes(chain.remaining());
        let expected = "HTTP/1.1 200 OK\r\n\
                        key1: value1\r\n\
                        key2: value2\r\n\
                        key3: value3\r\n\
                        key4: value4\r\n\
                        key5: value5\r\n\r\n";
        assert_eq!(result, expected);
        drop(chain);
        assert_eq!(res.into_bytes(), expected);
    }

    #[test]
    fn test_one_res_as_chain_with_body() {
        let res = OneResponseBuilder::default()
            .status(200)
            .headers(build_header_map())
            .body("dead body".into())
            .build()
            .unwrap();
        let mut chain = res.as_chain();
        let result = chain.copy_to_bytes(chain.remaining());
        let expected = "HTTP/1.1 200 OK\r\n\
                        key1: value1\r\n\
                        key2: value2\r\n\
                        key3: value3\r\n\
                        key4: value4\r\n\
                        key5: value5\r\n\r\n\
                        dead body";
        assert_eq!(result, expected);
        drop(chain);
        assert_eq!(res.into_bytes(), expected);
    }
}
