use body_plz::variants::Body;
use bytes::BytesMut;
use decompression_plz::DecompressTrait;
use header_plz::{
    OneHeader, OneInfoLine, body_headers::BodyHeader,
    message_head::header_map::HMap,
};

use crate::one::OneOne;

impl<T> DecompressTrait for OneOne<T>
where
    T: OneInfoLine,
{
    type HmapType = OneHeader;

    fn get_body(&mut self) -> Body {
        self.body.take().unwrap()
    }

    fn get_extra_body(&mut self) -> Option<BytesMut> {
        self.extra_body.take()
    }

    fn set_body(&mut self, body: Body) {
        self.body = Some(body);
    }

    fn body_headers(&self) -> Option<&BodyHeader> {
        self.body_headers.as_ref()
    }

    fn body_headers_as_mut(&mut self) -> Option<&mut BodyHeader> {
        self.body_headers.as_mut()
    }

    fn header_map(&self) -> &HMap<Self::HmapType> {
        self.message_head.header_map()
    }

    fn header_map_as_mut(&mut self) -> &mut HMap<Self::HmapType> {
        self.message_head.header_map_as_mut()
    }
}
