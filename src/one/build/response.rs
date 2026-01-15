use header_plz::OneResponseLine;

use super::*;

impl BuildMessage for OneOne<OneResponseLine> {
    fn build(buf: BytesMut) -> Result<Self, BuildMessageError> {
        OneOne::<OneResponseLine>::try_from(buf)
    }
}
