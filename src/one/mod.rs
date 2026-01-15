use body_plz::variants::Body;
use bytes::BytesMut;
use decompression_plz::{MultiDecompressErrorReason, decompress};
use header_plz::{
    HeaderMap, OneHeader, OneHeaderMap, OneInfoLine, OneMessageHead,
    OneRequestLine, OneResponseLine,
    body_headers::{
        BodyHeader, parse::ParseBodyHeaders, transfer_types::TransferType,
    },
    const_headers::{
        CLOSE, CONNECTION, CONTENT_LENGTH, KEEP_ALIVE, PROXY_CONNECTION,
        TRAILER, WS_EXT,
    },
    error::HeaderReadError,
};
pub mod impl_decompress;

pub mod build;
pub mod impl_try_from_bytes;
mod request;
mod response;

pub type OneRequest = OneOne<OneRequestLine>;
pub type OneResponse = OneOne<OneResponseLine>;

#[derive(Debug, PartialEq)]
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
    ) -> Result<Self, HeaderReadError> {
        let message_head = OneMessageHead::<T>::try_from(buf)?;
        let body_headers = message_head.parse_body_headers();
        Ok(OneOne::<T>::new(message_head, body_headers))
    }

    // Header Related methods
    pub fn message_head(&self) -> &OneMessageHead<T> {
        &self.message_head
    }

    pub fn has_header_key(&self, key: &str) -> Option<usize> {
        self.message_head.header_map().header_key_position(key)
    }

    pub fn add_header(&mut self, key: &str, value: &str) {
        self.message_head.header_map_as_mut().insert(key, value);
    }

    pub fn update_header_value_on_position(
        &mut self,
        pos: usize,
        value: &str,
    ) {
        self.message_head
            .header_map_as_mut()
            .update_header_value_on_position(pos, value);
    }

    pub fn update_header_value_on_key(
        &mut self,
        key: &str,
        value: &str,
    ) -> bool {
        self.message_head
            .header_map_as_mut()
            .update_header_value_on_key(key, value)
    }

    pub fn remove_header_on_position(&mut self, pos: usize) {
        self.message_head.header_map_as_mut().remove_header_on_position(pos);
    }

    pub fn remove_header_on_key(&mut self, key: &str) -> bool {
        self.message_head.header_map_as_mut().remove_header_on_key(key)
    }

    pub fn has_trailers(&self) -> bool {
        self.message_head.header_map().header_key_position(TRAILER).is_some()
    }

    pub fn set_transfer_type_close(&mut self) {
        self.body_headers.get_or_insert_with(Default::default).transfer_type =
            Some(TransferType::Close);
    }

    // Body Headers Related
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
            .header_position((CONNECTION, KEEP_ALIVE))
    }

    pub fn has_proxy_connection(&self) -> Option<usize> {
        self.message_head.header_map().header_key_position(PROXY_CONNECTION)
    }

    // Normalize
    pub fn normalize(&mut self) {
        if let Some(pos) = self.has_connection_keep_alive() {
            self.update_header_value_on_position(pos, CLOSE);
        }
        if let Some(pos) = self.has_proxy_connection() {
            self.remove_header_on_position(pos);
        }
        self.remove_header_on_key(WS_EXT);
    }

    pub fn try_decompress(
        &mut self,
        buf: &mut BytesMut,
    ) -> Result<(), MultiDecompressErrorReason> {
        decompress(self, buf)
    }
}

pub(crate) fn process_two_headers_and_body(
    mut headers: HeaderMap,
    body: Option<&BytesMut>,
    trailer: Option<HeaderMap>,
) -> OneHeaderMap {
    let mut header_map: OneHeaderMap = headers.into();

    // Merge trailers
    if let Some(trailers) = trailer {
        header_map.extend(OneHeaderMap::from(trailers).into_iter());
    }

    // Add content-length if needed
    if let Some(body) = body {
        if !header_map.has_key(CONTENT_LENGTH) {
            header_map.insert(CONTENT_LENGTH, body.len().to_string().as_str());
        }
    }

    header_map
}
