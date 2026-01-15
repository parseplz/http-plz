use crate::message::Message;
use body_plz::variants::Body;
use decompression_plz::DecompressTrait;
use header_plz::{
    Header, body_headers::BodyHeader, message_head::header_map::HMap,
};

// TODO: delimit
impl<T> DecompressTrait for Message<T> {
    type HmapType = Header;

    fn get_body(&mut self) -> Body {
        Body::Raw(self.body.take().unwrap())
    }

    fn get_extra_body(&mut self) -> Option<bytes::BytesMut> {
        None
    }

    fn set_body(&mut self, body: Body) {
        todo!()
    }

    fn body_headers(&self) -> Option<&BodyHeader> {
        todo!()
    }

    fn body_headers_as_mut(&mut self) -> Option<&mut BodyHeader> {
        todo!()
    }

    fn header_map(&self) -> &HMap<Self::HmapType> {
        &self.headers
    }

    fn header_map_as_mut(&mut self) -> &mut HMap<Self::HmapType> {
        &mut self.headers
    }
}
