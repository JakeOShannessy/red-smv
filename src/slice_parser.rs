use std::io::BufReader;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;

#[derive(Clone, Debug, PartialEq)]
pub struct SliceFile {
    pub header: SliceHeader,
    pub frames: Vec<Frame>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SliceHeader {
    pub quantity: String,
    pub short_name: String,
    pub units: String,
    pub dimensions: Dimensions,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Frame {
    pub time: f32,
    pub values: Vec<f32>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Dimensions {
    pub i_min: u32,
    pub i_max: u32,
    pub j_min: u32,
    pub j_max: u32,
    pub k_min: u32,
    pub k_max: u32,
}

#[derive(Debug)]
pub enum ParseSliceError {
    IOError(std::io::Error),
    RecLengthError,
}

impl std::fmt::Display for ParseSliceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "A parsing error occurred.")
    }
}
impl std::error::Error for ParseSliceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Self::IOError(ref e) => e.source(),
            Self::RecLengthError => None,
        }
    }
}

#[derive(Debug)]
pub struct SliceParser<R> {
    reader: BufReader<R>,
    pub header: SliceHeader,
    current_frame: usize,
    header_length: u64,
}

impl<R: Read> SliceParser<R> {
    pub fn parse_frame(&mut self) -> Result<Frame, ParseSliceError> {
        let i_dim = self.header.dimensions.i_max - self.header.dimensions.i_min + 1;
        let j_dim = self.header.dimensions.j_max - self.header.dimensions.j_min + 1;
        let k_dim = self.header.dimensions.k_max - self.header.dimensions.k_min + 1;
        let frame = parse_data_set(i_dim, j_dim, k_dim, &mut self.reader);
        if frame.is_ok() {
            // TODO: needs better error handling
            self.current_frame += 1;
        }
        frame
    }
    pub fn header_length(&self) -> u64 {
        self.header_length
    }
    pub fn frame_length(&self) -> u64 {
        // Time record 4+4+4
        let time_length = 4 + 4 + 4;
        let i_dim = self.header.dimensions.i_max - self.header.dimensions.i_min + 1;
        let j_dim = self.header.dimensions.j_max - self.header.dimensions.j_min + 1;
        let k_dim = self.header.dimensions.k_max - self.header.dimensions.k_min + 1;
        let n_values = (i_dim * j_dim * k_dim) as usize;
        let data_length = 4 + n_values * 4 + 4;
        (time_length + data_length) as u64
    }
}
impl<R: Read + std::io::Seek> SliceParser<R> {
    pub fn new(input: R) -> Result<Self, ParseSliceError> {
        let mut reader = BufReader::new(input);
        let header = parse_slice_header(&mut reader)?;
        let header_length = reader.stream_position().map_err(ParseSliceError::IOError)?;
        Ok(SliceParser {
            reader,
            header,
            current_frame: 0,
            header_length,
        })
    }
}
impl<R: Read + std::io::Seek> SliceParser<R> {
    pub fn seek_next_frame(&mut self) -> std::io::Result<()> {
        self.reader.seek_relative(self.frame_length() as i64)
    }
    pub fn seek_frame(&mut self, frame: usize) -> std::io::Result<u64> {
        self.reader.seek(SeekFrom::Start(
            self.header_length() + self.frame_length() * (frame as u64),
        ))
    }
    pub fn get_frame(&mut self, frame: usize) -> Result<Frame, ParseSliceError> {
        self.seek_frame(frame).map_err(ParseSliceError::IOError)?;
        self.current_frame = frame;
        self.parse_frame()
    }
}
impl<R: Read> Iterator for SliceParser<R> {
    type Item = Result<Frame, ParseSliceError>;

    fn next(&mut self) -> Option<Self::Item> {
        let frame = self.parse_frame();
        Some(frame)
    }
}

pub fn parse_slice_file<R: Read + Seek>(i: &mut R) -> Result<SliceFile, ParseSliceError> {
    let parser = SliceParser::new(i)?;
    eprintln!("header: {:?}", parser.header);
    let header = parser.header.clone();
    let mut frames = Vec::new();
    for (i, frame) in parser.enumerate() {
        eprintln!("frame[{}]: {:?}", i, frame.as_ref().map(|x| x.time));
        if let Ok(frame) = frame {
            frames.push(frame);
        } else {
            break;
        }
    }
    Ok(SliceFile { header, frames })
}

pub fn parse_data_set<R: Read>(
    i_dim: u32,
    j_dim: u32,
    k_dim: u32,
    mut i: R,
) -> Result<Frame, ParseSliceError> {
    let rec_length = {
        let mut buf = [0u8; 4];
        i.read_exact(&mut buf).map_err(ParseSliceError::IOError)?;
        u32::from_le_bytes(buf)
    };
    let time = {
        let mut buf = [0u8; 4];
        i.read_exact(&mut buf).map_err(ParseSliceError::IOError)?;
        f32::from_le_bytes(buf)
    };
    let check_length = {
        let mut buf = [0u8; 4];
        i.read_exact(&mut buf).map_err(ParseSliceError::IOError)?;
        u32::from_le_bytes(buf)
    };
    if check_length != rec_length {
        return Err(ParseSliceError::RecLengthError);
    }
    let values = parse_slice_data(i_dim, j_dim, k_dim, i)?;
    Ok(Frame { time, values })
}

pub fn parse_slice_data<R: Read>(
    i_dim: u32,
    j_dim: u32,
    k_dim: u32,
    mut i: R,
) -> Result<Vec<f32>, ParseSliceError> {
    let rec_length = {
        let mut buf = [0u8; 4];
        i.read_exact(&mut buf).map_err(ParseSliceError::IOError)?;
        u32::from_le_bytes(buf)
    };
    let n_values = (i_dim * j_dim * k_dim) as usize;
    let mut values = Vec::with_capacity(n_values);
    let mut buf = [0u8; 4];
    for _ in 0..n_values {
        let value = {
            i.read_exact(&mut buf).map_err(ParseSliceError::IOError)?;
            f32::from_le_bytes(buf)
        };
        values.push(value);
    }
    let check_length = {
        let mut buf = [0u8; 4];
        i.read_exact(&mut buf).map_err(ParseSliceError::IOError)?;
        u32::from_le_bytes(buf)
    };
    if check_length != rec_length {
        return Err(ParseSliceError::RecLengthError);
    }
    Ok(values)
}

fn parse_slice_header<R: Read>(i: &mut R) -> Result<SliceHeader, ParseSliceError> {
    let quantity = parse_record(i)?;
    let short_name = parse_record(i)?;
    let units = parse_record(i)?;
    let dimensions = parse_dimensions(i)?;
    Ok(SliceHeader {
        quantity: String::from_utf8(quantity).unwrap(),
        short_name: String::from_utf8(short_name).unwrap(),
        units: String::from_utf8(units).unwrap(),
        dimensions,
    })
}

fn parse_dimensions<R: Read>(mut i: R) -> Result<Dimensions, ParseSliceError> {
    let rec_length = {
        let mut buf = [0u8; 4];
        i.read_exact(&mut buf).map_err(ParseSliceError::IOError)?;
        u32::from_le_bytes(buf)
    };
    if rec_length != 24 {
        return Err(ParseSliceError::RecLengthError);
    }
    let i1 = {
        let mut buf = [0u8; 4];
        i.read_exact(&mut buf).map_err(ParseSliceError::IOError)?;
        u32::from_le_bytes(buf)
    };
    let i2 = {
        let mut buf = [0u8; 4];
        i.read_exact(&mut buf).map_err(ParseSliceError::IOError)?;
        u32::from_le_bytes(buf)
    };
    let j1 = {
        let mut buf = [0u8; 4];
        i.read_exact(&mut buf).map_err(ParseSliceError::IOError)?;
        u32::from_le_bytes(buf)
    };
    let j2 = {
        let mut buf = [0u8; 4];
        i.read_exact(&mut buf).map_err(ParseSliceError::IOError)?;
        u32::from_le_bytes(buf)
    };
    let k1 = {
        let mut buf = [0u8; 4];
        i.read_exact(&mut buf).map_err(ParseSliceError::IOError)?;
        u32::from_le_bytes(buf)
    };
    let k2 = {
        let mut buf = [0u8; 4];
        i.read_exact(&mut buf).map_err(ParseSliceError::IOError)?;
        u32::from_le_bytes(buf)
    };
    let check_length = {
        let mut buf = [0u8; 4];
        i.read_exact(&mut buf).map_err(ParseSliceError::IOError)?;
        u32::from_le_bytes(buf)
    };
    if check_length != rec_length {
        return Err(ParseSliceError::RecLengthError);
    }
    Ok(Dimensions {
        i_min: i1,
        i_max: i2,
        j_min: j1,
        j_max: j2,
        k_min: k1,
        k_max: k2,
    })
}

/// Parse the data from a record, ensuring the record length tags at the start
/// and finish match.
fn parse_record<R: Read>(i: &mut R) -> Result<Vec<u8>, ParseSliceError> {
    // Take the length of the record, which is the first 4 bytes of the record
    // as a 32-bit as an integer. The length is in bytes.
    let rec_length = {
        let mut buf = [0u8; 4];
        i.read_exact(&mut buf).map_err(ParseSliceError::IOError)?;
        u32::from_le_bytes(buf)
    };
    // Take the number of bytes specified by rec_length.
    let rec_bytes = {
        let mut buf = vec![0u8; rec_length as usize];
        i.read_exact(&mut buf).map_err(ParseSliceError::IOError)?;
        buf
    };
    let check_length = {
        let mut buf = [0u8; 4];
        i.read_exact(&mut buf).map_err(ParseSliceError::IOError)?;
        u32::from_le_bytes(buf)
    };
    if check_length != rec_length {
        panic!("bad rec_length start: {} end: {}", rec_length, check_length);
    }
    Ok(rec_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    // In these tests Ok(remaining, result) is used to make sure that we have
    // consumed the input we expect to consume.
    #[test]
    fn parse_slice_simple() {
        let result =
            parse_slice_file(&mut std::io::Cursor::new(include_bytes!("room_fire_01.sf"))).unwrap();
        assert_eq!(result.header.quantity.trim(), "TEMPERATURE".to_string());
        assert_eq!(result.header.units.trim(), "C".to_string());
        assert_eq!(result.header.short_name.trim(), "temp".to_string());
        assert_eq!(
            result.header.dimensions,
            Dimensions {
                i_min: 14,
                i_max: 14,
                j_min: 0,
                j_max: 10,
                k_min: 0,
                k_max: 24,
            }
        );
        assert_eq!(result.frames.len(), 945);
        let mut frames = Vec::new();
        let reader = &mut std::io::Cursor::new(include_bytes!("room_fire_01.sf"));
        let mut parser = SliceParser::new(reader).unwrap();
        for i in 0..945 {
            let frame = parser.get_frame(i).unwrap();
            frames.push(frame);
        }
        assert_eq!(result.frames, frames);
    }

    #[test]
    fn parse_slice_simple_bad01() {
        let result = parse_slice_file(&mut std::io::Cursor::new(include_bytes!(
            "room_fire_01_bad01.sf"
        )));
        assert!(result.is_err())
    }
}
