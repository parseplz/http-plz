use crate::message::Message;
use body_plz::variants::Body;
use decompression_plz::DecompressTrait;
use header_plz::{
    Header, body_headers::BodyHeader, message_head::header_map::HMap,
};

impl<T> DecompressTrait for Message<T> {
    type HmapType = Header;

    fn take_body(&mut self) -> Option<Body> {
        self.body.take().map(Body::Raw)
    }

    fn take_extra_body(&mut self) -> Option<bytes::BytesMut> {
        None
    }

    fn set_body(&mut self, body: Body) {
        if let Body::Raw(body) = body {
            self.body = Some(body)
        }
    }

    fn body_headers(&self) -> Option<&BodyHeader> {
        self.body_headers.as_ref()
    }

    fn body_headers_as_mut(&mut self) -> Option<&mut BodyHeader> {
        self.body_headers.as_mut()
    }

    fn header_map(&self) -> &HMap<Self::HmapType> {
        &self.headers
    }

    fn header_map_as_mut(&mut self) -> &mut HMap<Self::HmapType> {
        &mut self.headers
    }
}
