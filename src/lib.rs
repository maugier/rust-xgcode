use std::io::{Read, Write};
use byteorder::{ReadBytesExt, LittleEndian, WriteBytesExt};
use thiserror::Error;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// XGCode header
pub struct Header {
    /// Print time, in seconds
    pub print_time: u32,
    /// Filament usage, extruder 0 (right), in mm
    pub filament_0_usage: u32,
    /// Filament usage, extruder 1 (left), in mm
    pub filament_1_usage: u32,

    /// Type of multi-extruder
    pub multi_extruder_type: u16,

    /// Layer height, microns
    pub layer_height: u16,

    /// Function unknown
    pub reserved0: u16,

    /// Perimeter shells, number
    pub perimeter_shells: u16,

    /// Print speed, mm/s
    pub print_speed: u16,

    /// Hotbed temperature, °C
    pub hotbed_temp: u16,

    /// Extruder 0 (right) temperature, °C
    pub extruder_0_temp: u16,

    /// Extruder 1 (left) temperature, °C
    pub extruder_1_temp: u16,

    /// Function unknown
    pub reserved1: u16,

}

#[derive(Clone,Debug,Eq,PartialEq)]
pub struct XGCode {
    /// XGcode header
    pub header: Header,
    /// Print thumbnail (BMP file, 80x60, 8-bit RGB)
    pub thumbnail: Vec<u8>,
    /// GCode file
    pub gcode: Vec<u8>
}

#[derive(Copy,Clone,Debug,Eq,PartialEq)]
pub struct XGCodeRef<'a> {
    /// XGcode header
    pub header: Header,
    /// Print thumbnail (BMP file, 80x60, 8-bit RGB)
    pub thumbnail: &'a [u8],
    /// Print thumbnail (BMP file, 80x60, 8-bit RGB)
    pub gcode: &'a [u8],
}



#[derive(Debug,Error)]
pub enum Error {
    #[error("Bad magic header")]    BadMagic(Box<[u8; 16]>),
    #[error("Bad header size")]     BadHeaderSize(u32),
    #[error("Thumb size negative")] ThumbSizeNegative(i32),
    #[error("GCode too big")]       ThumbnailTooLarge(usize),
    #[error("Second goffset not found")]  SecondGOffsetNotFound,
    #[error("Data in reserved field")]  DataInReservedField {offset: u16, value: u16},
    #[error("IO error")]            IO(#[from] std::io::Error),
}

const XGCODE_MAGIC: &'static [u8; 16] = b"xgcode 1.0\n\0\0\0\0\0";
const THUMB_OFFSET: u32 = 0x3A;

impl XGCode {

    pub fn read<R: Read>(mut source: R) -> Result<Self, Error> {
        let mut magic = [0; 16];
        source.read_exact(&mut magic)?;
        
        if &magic != XGCODE_MAGIC { return Err(Error::BadMagic(Box::new(magic))) }

        let thumb_offset = source.read_u32::<LittleEndian>()?;
        if thumb_offset != THUMB_OFFSET { return Err(Error::BadHeaderSize(thumb_offset)) }

        let gcode_offset = source.read_u32::<LittleEndian>()?;
        let thumb_size = (gcode_offset as usize).checked_sub(THUMB_OFFSET as usize)
            .ok_or(Error::ThumbSizeNegative(gcode_offset as i32 - THUMB_OFFSET as i32))?;


        let gcode_offset2 = source.read_u32::<LittleEndian>()?;
        if gcode_offset != gcode_offset2 { return Err(Error::SecondGOffsetNotFound)}

        let print_time = source.read_u32::<LittleEndian>()?;
        let filament_0_usage = source.read_u32::<LittleEndian>()?;
        let filament_1_usage = source.read_u32::<LittleEndian>()?;
        let multi_extruder_type = source.read_u16::<LittleEndian>()?;
        let layer_height = source.read_u16::<LittleEndian>()?;
        let reserved0 = source.read_u16::<LittleEndian>()?;
        let perimeter_shells = source.read_u16::<LittleEndian>()?;
        let print_speed = source.read_u16::<LittleEndian>()?;
        let hotbed_temp = source.read_u16::<LittleEndian>()?;
        let extruder_0_temp = source.read_u16::<LittleEndian>()?;
        let extruder_1_temp = source.read_u16::<LittleEndian>()?;
        let reserved1 = source.read_u16::<LittleEndian>()?;

        let header = Header { print_time, filament_0_usage, filament_1_usage, multi_extruder_type, layer_height, perimeter_shells, print_speed, hotbed_temp, extruder_0_temp, extruder_1_temp, reserved0, reserved1 };

        let mut thumbnail = vec![0; thumb_size];
        source.read_exact(&mut thumbnail)?;

        let mut gcode = vec![];
        source.read_to_end(&mut gcode)?;

        Ok(XGCode{ header, thumbnail, gcode })

    }

    pub fn as_ref(&self) -> XGCodeRef {
        XGCodeRef { header: self.header, thumbnail: &self.thumbnail[..], gcode: &self.gcode[..] }
    }

    pub fn write<W: Write>(&self, writer: W) -> Result<(), Error> {
        self.as_ref().write(writer)


    }

}

impl<'a> XGCodeRef<'a> {
    fn write<W: Write>(&self, mut writer: W) -> Result<(), Error> {

        let gcode_offset = THUMB_OFFSET as usize + self.thumbnail.len();
        if gcode_offset > (u32::MAX as usize) { return Err(Error::ThumbnailTooLarge(self.thumbnail.len()))}

        writer.write_all(XGCODE_MAGIC)?;
        writer.write_u32::<LittleEndian>(THUMB_OFFSET)?;
        writer.write_u32::<LittleEndian>(gcode_offset as u32)?;
        writer.write_u32::<LittleEndian>(gcode_offset as u32)?;  // Yes, there is a second field

        writer.write_u32::<LittleEndian>(self.header.print_time)?;
        writer.write_u32::<LittleEndian>(self.header.filament_0_usage)?;
        writer.write_u32::<LittleEndian>(self.header.filament_1_usage)?;
        writer.write_u16::<LittleEndian>(self.header.multi_extruder_type)?;
        writer.write_u16::<LittleEndian>(self.header.layer_height)?;
        writer.write_u16::<LittleEndian>(self.header.reserved0)?;
        writer.write_u16::<LittleEndian>(self.header.perimeter_shells)?;
        writer.write_u16::<LittleEndian>(self.header.print_speed)?;
        writer.write_u16::<LittleEndian>(self.header.hotbed_temp)?;
        writer.write_u16::<LittleEndian>(self.header.extruder_0_temp)?;
        writer.write_u16::<LittleEndian>(self.header.extruder_1_temp)?;
        writer.write_u16::<LittleEndian>(self.header.reserved1)?;

        writer.write_all(self.thumbnail)?;
        writer.write_all(self.gcode)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    

    use std::{fs::File, io::Write};

    use crate::XGCode;

    #[test]
    fn test_sample_file() {
        let file = include_bytes!("../test/20mm_Box.gx");
        let parsed = XGCode::read(&mut &file[..]).unwrap();

        // Check .bmp magic in thumbnail
        assert_eq!(&parsed.thumbnail[..2], b"BM");

        let mut file2 = vec![];
        parsed.write(&mut file2).unwrap();


        assert_eq!(file, &file2[..]);

    }

}
