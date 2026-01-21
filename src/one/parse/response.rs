use header_plz::OneResponseLine;

use super::*;

impl ParseMessage for OneOne<OneResponseLine> {
    fn parse(buf: BytesMut) -> Result<Self, BuildMessageError> {
        OneOne::<OneResponseLine>::try_from(buf)
    }
}
