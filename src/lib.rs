//! Elder Scrolls Master file parser.

#![allow(non_snake_case)]

use chunk_parser::prelude::*;
pub use chunk_parser::Result;
use esm_bindings::fo3::*;

use std::ffi::CString;
use std::io::Read;

use flate2::read::ZlibDecoder;

//------------------------------------------------------------------------------

#[chunk_parser(custom,depth)]
pub struct ESMParser {
    localised: bool
}

type RecordParser<P> = fn(parser: &mut P, header: &RecordHeader) -> Result<()>;
type FieldParser<P> = fn(parser: &mut P, header: &FieldHeader) -> Result<()>;

macro_rules! indent {
    ($parser:expr, $($arg:tt)*) => {
        let indent = " ".repeat($parser.depth() as usize * 2);
        print!("{}{}", indent, format!($($arg)*));
    };
}
macro_rules! indentln {
    ($parser:expr, $($arg:tt)*) => {
        let indent = " ".repeat($parser.depth() as usize * 2);
        println!("{}{}", indent, format!($($arg)*));
    };
}

/// Elder Scrolls Master parser implementation.
impl<R> ESMParser<R> where R: std::io::Read + std::io::Seek {
    /// Read a fixed sized string.
    fn read_zstring(&mut self, length: u16) -> Result<CString> {
        let mut v = Vec::with_capacity(length as usize);
        unsafe {
            let ptr = v.as_mut_ptr();
            self.reader().read_exact(std::slice::from_raw_parts_mut(ptr, length as usize))?;
            v.set_len(length as usize);
        }
        Ok(unsafe { CString::from_vec_unchecked(v) })
    }

    /// Read a potentially localised string.
    fn read_lstring(&mut self, length: u16) -> Result<CString> {
        if self.localised { panic!("unimplemented lstring");  }
        else { self.read_zstring(length) }
    }

    /// Decompress a Zlib buffer.
    fn deflate(&mut self, size: usize) -> Result<Vec<u8>> {
        let mut v = Vec::with_capacity(size);
        unsafe {
            let ptr = v.as_mut_ptr();
            self.reader().read_exact(std::slice::from_raw_parts_mut(ptr, size))?;
            v.set_len(size);
        }
        let mut decoder = ZlibDecoder::new(&v[..]);
        let mut decompressed_data = Vec::new();
        decoder.read_to_end(&mut decompressed_data)?;
        Ok(decompressed_data)
    }

    pub fn GRUP(&mut self, header: &RecordHeader) -> Result<()> {
        let RecordHeader { size, type_id, flags, .. } = *header;

//        if self.depth() >= 3 {
//            self.skip(size as u64)?;
//            return Ok(())
//        }

        if type_id == b"GRUP" {
            let GRUP: GroupHeader = unsafe { std::mem::transmute(*header) };
            indentln!(self, "{:?}", GRUP);
        } else {
            if type_id != b"REFR" {
                indentln!(self, "{:?}", header);
            }

            if (flags & 0x00040000) != 0 {
                let _uncompressed_size: u32 = self.read()?;
                let decompressed = &self.deflate(size as usize - 4)?;
                let reader = std::io::Cursor::new(decompressed);
                let mut parser = ESMParser::new(reader);
                *parser.inner_depth() = self.depth();
                parser.localised = self.localised;
                parser.push();
                // this block is for the first compressed record, NPC_
                parser.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"FULL" => {
                            let FULL = parser.read_lstring(header.size)?;
                            println!("{:?}", FULL);
                        },
                        b"OBND" => {
                            let OBND: OBND = parser.read()?;
                            println!("{:?}", OBND);
                        },
                        b"MODL" => {
                            let MODL = parser.read_zstring(header.size)?;
                            println!("{:?}", MODL);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, decompressed.len() as u32)?;
                parser.pop();
                return Ok(())
            }
        }

        match &type_id.0 {
            b"GLOB" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"FNAM" => {
                            let FNAM: u8 = parser.read()?;
                            println!("{:?}", FNAM);
                        },
                        b"FLTV" => {
                            let FLTV: f32 = parser.read()?;
                            println!("{:?}", FLTV);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"FACT" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"FULL" => {
                            let FULL = parser.read_lstring(header.size)?;
                            println!("{:?}", FULL);
                        },
                        b"XNAM" => {
                            let XNAM: XNAM = parser.read()?;
                            println!("{:?}", XNAM);
                        },
                        b"DATA" => {
                            let DATA: u32 = parser.read()?;
                            println!("{:#010x}", DATA);
                        },
                        b"RNAM" => {
                            let RNAM: u32 = parser.read()?;
                            println!("{:#010x}", RNAM);
                        },
                        b"MNAM" => {
                            let MNAM = parser.read_lstring(header.size)?;
                            println!("{:?}", MNAM);
                        },
                        b"FNAM" => {
                            let FNAM = parser.read_lstring(header.size)?;
                            println!("{:?}", FNAM);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"TXST" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"OBND" => {
                            let OBND: OBND = parser.read()?;
                            println!("{:?}", OBND);
                        },
                        tx if tx >= b"TX00" && tx <= b"TX07" => {
                            let TX = parser.read_zstring(header.size)?;
                            println!("{:?}", TX);
                        },
                        b"DNAM" => {
                            let DNAM: u16 = parser.read()?;
                            println!("{:?}", DNAM);
                        },
                        /*b"DODT" => {
                            let DODT: DODT = parser.read()?;
                            println!("{:?}", DODT);
                        },*/
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!(" Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"CLAS" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, " {:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"FULL" => {
                            let FULL = parser.read_lstring(header.size)?;
                            println!("{:?}", FULL);
                        },
                        b"DESC" => {
                            let DESC = parser.read_lstring(header.size)?;
                            println!("{:?}", DESC);
                        },
                        /*b"ICON" => {
                            let ICON = parser.read_zstring(header.size)?;
                            println!("{:?}", ICON);
                        },*/
                        b"DATA" => {
                            let CLAS: CLAS = parser.read()?;
                            println!("{:?}", CLAS);
                        },
                        b"ATTR" => {
                            let ATTR: ATTR = parser.read()?;
                            println!("{:?}", ATTR);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"SOUN" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"OBND" => {
                            let OBND: OBND = parser.read()?;
                            println!("{:?}", OBND);
                        },
                        b"FNAM" => {
                            let FNAM = parser.read_zstring(header.size)?;
                            println!("{:?}", FNAM);
                        },
                        b"SNDD" => {
                            let SNDD: SNDD = parser.read()?;
                            println!("{:?}", SNDD);
                        },
                        b"SDSC" => {
                            let SDSC: formid_t = parser.read()?;
                            println!("{:?}", SDSC);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"ASPC" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"OBND" => {
                            let OBND: OBND = parser.read()?;
                            println!("{:?}", OBND);
                        },
                        b"SNAM" => {
                            let SNAM: formid_t = parser.read()?;
                            println!("{:?}", SNAM);
                        },
                        b"RDAT" => {
                            let RDAT: formid_t = parser.read()?;
                            println!("{:?}", RDAT);
                        },
                        b"BNAM" => {
                            let BNAM: formid_t = parser.read()?;
                            println!("{:?}", BNAM);
                        },
                        b"ANAM" => {
                            let ANAM: [u8;4] = parser.read()?;
                            println!("ANAM {{ unknown: {:?} }}", ANAM);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"MGEF" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        //b"VMAD" => {},
                        b"FULL" => {
                            let FULL = parser.read_lstring(header.size)?;
                            println!("{:?}", FULL);
                        },
                        b"DESC" => {
                            let DESC = parser.read_lstring(header.size)?;
                            println!("{:?}", DESC);
                        },
                        /*b"MDOB" => {
                            let MDOB: formid_t = parser.read()?;
                            println!("{:?}", MDOB);
                        },
                        b"KSIZ" => {
                            let KSIZ: u32 = parser.read()?;
                            println!("{:?}", KSIZ);
                        },
                        b"KWDA" => {},*/
                        b"DATA" => {
                            let MGEF: MGEF = parser.read()?;
                            println!("{:?}", MGEF);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"ENCH" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"FULL" => {
                            let FULL = parser.read_lstring(header.size)?;
                            println!("{:?}", FULL);
                        },
                        b"ENIT" => {
                            let ENIT: ENIT = parser.read()?;
                            println!("{:?}", ENIT);
                        },
                        b"EFID" => {
                            let EFID: formid_t = parser.read()?;
                            println!("{:?}", EFID);
                        },
                        b"EFIT" => {
                            let EFIT: EFIT = parser.read()?;
                            println!("{:?}", EFIT);
                        },
                        /*b"OBND" => {
                            let OBND: OBND = parser.read()?;
                            println!("{:?}", OBND);
                        },*/
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"SCPT" => {
                let SCPT = self.read_lstring(size as u16)?;
                println!("{:?}", SCPT);
            },
            b"SPEL" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"FULL" => {
                            let FULL = parser.read_lstring(header.size)?;
                            println!("{:?}", FULL);
                        },
                        b"SPIT" => {
                            let SPIT: SPIT = parser.read()?;
                            println!("{:?}", SPIT);
                        },
                        b"EFID" => {
                            let EFID: formid_t = parser.read()?;
                            println!("{:?}", EFID);
                        },
                        b"EFIT" => {
                            let EFIT: EFIT = parser.read()?;
                            println!("{:?}", EFIT);
                        },
                        b"CTDA" => {
                            let CTDA: CTDA = parser.read()?;
                            println!("{:?}", CTDA);
                        },
                        /*b"OBND" => {
                            let OBND: OBND = parser.read()?;
                            println!("{:?}", OBND);
                        },*/
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"ACTI" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"FULL" => {
                            let FULL = parser.read_lstring(header.size)?;
                            println!("{:?}", FULL);
                        },
                        b"OBND" => {
                            let OBND: OBND = parser.read()?;
                            println!("{:?}", OBND);
                        },
                        b"SCRI" => {
                            let SCRI: formid_t = parser.read()?;
                            println!("{:?}", SCRI);
                        },
                        b"VNAM" => {
                            let VNAM: formid_t = parser.read()?;
                            println!("{:?}", VNAM);
                        },
                        b"MODL" => {
                            let MODL = parser.read_zstring(header.size)?;
                            println!("{:?}", MODL);
                        },
                        b"SNAM" => {
                            let SNAM: formid_t = parser.read()?;
                            println!("{:?}", SNAM);
                        },
                        b"DEST" => {
                            let DEST: [u8;8] = parser.read()?;
                            println!("DEST {{ unknown: {:?} }}", DEST);
                        },
                        b"DSTD" => {
                            let DSTD: DSTD = parser.read()?;
                            println!("{:?}", DSTD);
                        },
                        b"DSTF" => {
                            println!();
                        },
                        b"DMDL" => {
                            let DMDL = parser.read_zstring(header.size)?;
                            println!("{:?}", DMDL);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"TERM" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"FULL" => {
                            let FULL = parser.read_lstring(header.size)?;
                            println!("{:?}", FULL);
                        },
                        b"OBND" => {
                            let OBND: OBND = parser.read()?;
                            println!("{:?}", OBND);
                        },
                        b"MODL" => {
                            let MODL = parser.read_zstring(header.size)?;
                            println!("{:?}", MODL);
                        },
                        b"DESC" => {
                            let DESC = parser.read_lstring(header.size)?;
                            println!("{:?}", DESC);
                        },
                        b"CTDA" => {
                            let CTDA: CTDA = parser.read()?;
                            println!("{:?}", CTDA);
                        },
                        b"RNAM" => {
                            let RNAM = parser.read_zstring(header.size)?;
                            println!("{:?}", RNAM);
                        },
                        b"ITXT" => {
                            let ITXT = parser.read_zstring(header.size)?;
                            println!("{:?}", ITXT);
                        },
                        b"SNAM" => {
                            let SNAM: formid_t = parser.read()?;
                            println!("{:?}", SNAM);
                        },
                        b"SCHR" => {
                            let SCHR: SCHR = parser.read()?;
                            println!("{:?}", SCHR);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"CONT" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"FULL" => {
                            let FULL = parser.read_lstring(header.size)?;
                            println!("{:?}", FULL);
                        },
                        b"OBND" => {
                            let OBND: OBND = parser.read()?;
                            println!("{:?}", OBND);
                        },
                        b"MODL" => {
                            let MODL = parser.read_zstring(header.size)?;
                            println!("{:?}", MODL);
                        },
                        b"SCRI" => {
                            let SCRI: formid_t = parser.read()?;
                            println!("{:?}", SCRI);
                        },
                        b"DATA" => {
                            let DATA: [u8;5] = parser.read()?;
                            println!("DATA {{ unknown: {:?} }}", DATA);
                        },
                        b"CNTO" => {
                            let CNTO: CNTO = parser.read()?;
                            println!("{:?}", CNTO);
                        },
                        b"COED" => {
                            let COED: COED = parser.read()?;
                            println!("{:?}", COED);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"LIGH" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"OBND" => {
                            let OBND: OBND = parser.read()?;
                            println!("{:?}", OBND);
                        },
                        b"MODL" => {
                            let MODL = parser.read_zstring(header.size)?;
                            println!("{:?}", MODL);
                        },
                        b"SCRI" => {
                            let SCRI: formid_t = parser.read()?;
                            println!("{:?}", SCRI);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"MISC" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"OBND" => {
                            let OBND: OBND = parser.read()?;
                            println!("{:?}", OBND);
                        },
                        b"MODL" => {
                            let MODL = parser.read_zstring(header.size)?;
                            println!("{:?}", MODL);
                        },
                        b"ICON" => {
                            let ICON = parser.read_zstring(header.size)?;
                            println!("{:?}", ICON);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"STAT" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"FULL" => {
                            let FULL = parser.read_lstring(header.size)?;
                            println!("{:?}", FULL);
                        },
                        b"OBND" => {
                            let OBND: OBND = parser.read()?;
                            println!("{:?}", OBND);
                        },
                        b"MODL" => {
                            let MODL = parser.read_zstring(header.size)?;
                            println!("{:?}", MODL);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"MSTT" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"FULL" => {
                            let FULL = parser.read_lstring(header.size)?;
                            println!("{:?}", FULL);
                        },
                        b"OBND" => {
                            let OBND: OBND = parser.read()?;
                            println!("{:?}", OBND);
                        },
                        b"MODL" => {
                            let MODL = parser.read_zstring(header.size)?;
                            println!("{:?}", MODL);
                        },
                        b"DATA" => {
                            let DATA: u8 = parser.read()?;
                            println!("{:?}", DATA);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"PWAT" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"OBND" => {
                            let OBND: OBND = parser.read()?;
                            println!("{:?}", OBND);
                        },
                        b"MODL" => {
                            let MODL = parser.read_zstring(header.size)?;
                            println!("{:?}", MODL);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"FURN" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"FULL" => {
                            let FULL = parser.read_lstring(header.size)?;
                            println!("{:?}", FULL);
                        },
                        b"OBND" => {
                            let OBND: OBND = parser.read()?;
                            println!("{:?}", OBND);
                        },
                        b"MODL" => {
                            let MODL = parser.read_zstring(header.size)?;
                            println!("{:?}", MODL);
                        },
                        b"MNAM" => {
                            let MNAM: u32 = parser.read()?;
                            println!("{:?}", MNAM);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"WEAP" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"FULL" => {
                            let FULL = parser.read_lstring(header.size)?;
                            println!("{:?}", FULL);
                        },
                        b"OBND" => {
                            let OBND: OBND = parser.read()?;
                            println!("{:?}", OBND);
                        },
                        b"MODL" => {
                            let MODL = parser.read_zstring(header.size)?;
                            println!("{:?}", MODL);
                        },
                        b"MOD2" => {
                            let MOD2 = parser.read_zstring(header.size)?;
                            println!("{:?}", MOD2);
                        },
                        b"MOD3" => {
                            let MOD3 = parser.read_zstring(header.size)?;
                            println!("{:?}", MOD3);
                        },
                        b"MOD4" => {
                            let MOD4 = parser.read_zstring(header.size)?;
                            println!("{:?}", MOD4);
                        },
                        b"ICON" => {
                            let ICON = parser.read_zstring(header.size)?;
                            println!("{:?}", ICON);
                        },
                        b"CRDT" => {
                            let CRDT: CRDT = parser.read()?;
                            println!("{:?}", CRDT);
                        },
                        b"EITM" => {
                            let EITM: formid_t = parser.read()?;
                            println!("{:?}", EITM);
                        },
                        b"ETYP" => {
                            let ETYP: u32 = parser.read()?;
                            println!("{:?}", ETYP);
                        },
                        b"DATA" => {
                            let DATA: DATA = parser.read()?;
                            println!("{:?}", DATA);
                        },
                        b"REPL" => {
                            let REPL: formid_t = parser.read()?;
                            println!("{:?}", REPL);
                        },
                        b"SCRI" => {
                            let SCRI: formid_t = parser.read()?;
                            println!("{:?}", SCRI);
                        },
                        b"NAM0" => {
                            let NAM0: formid_t = parser.read()?;
                            println!("{:?}", NAM0);
                        },
                        b"NAM6" => {
                            let NAM6: formid_t = parser.read()?;
                            println!("{:?}", NAM6);
                        },
                        b"NAM8" => {
                            let NAM8: formid_t = parser.read()?;
                            println!("{:?}", NAM8);
                        },
                        b"NAM9" => {
                            let NAM9: formid_t = parser.read()?;
                            println!("{:?}", NAM9);
                        },
                        b"DNAM" => {
                            let DNAM: DNAM = parser.read()?;
                            println!("{:?}", DNAM);
                        },
                        b"INAM" => {
                            let INAM: formid_t = parser.read()?;
                            println!("{:?}", INAM);
                        },
                        b"NNAM" => {
                            let NNAM = parser.read_zstring(header.size)?;
                            println!("{:?}", NNAM);
                        },
                        b"SNAM" => {
                            let SNAM: formid_t = parser.read()?;
                            println!("{:?}", SNAM);
                        },
                        b"TNAM" => {
                            let TNAM: formid_t = parser.read()?;
                            println!("{:?}", TNAM);
                        },
                        b"UNAM" => {
                            let UNAM: formid_t = parser.read()?;
                            println!("{:?}", UNAM);
                        },
                        b"VNAM" => {
                            let VNAM: u32 = parser.read()?;
                            println!("{:?}", VNAM);
                        },
                        b"WNAM" => {
                            let WNAM: formid_t = parser.read()?;
                            println!("{:?}", WNAM);
                        },
                        b"XNAM" => {
                            let XNAM: formid_t = parser.read()?;
                            println!("{:?}", XNAM);
                        },
                        b"YNAM" => {
                            let YNAM: formid_t = parser.read()?;
                            println!("{:?}", YNAM);
                        },
                        b"ZNAM" => {
                            let ZNAM: formid_t = parser.read()?;
                            println!("{:?}", ZNAM);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"AMMO" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"FULL" => {
                            let FULL = parser.read_lstring(header.size)?;
                            println!("{:?}", FULL);
                        },
                        b"OBND" => {
                            let OBND: OBND = parser.read()?;
                            println!("{:?}", OBND);
                        },
                        b"MODL" => {
                            let MODL = parser.read_zstring(header.size)?;
                            println!("{:?}", MODL);
                        },
                        b"ICON" => {
                            let ICON = parser.read_zstring(header.size)?;
                            println!("{:?}", ICON);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"CREA" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"FULL" => {
                            let FULL = parser.read_lstring(header.size)?;
                            println!("{:?}", FULL);
                        },
                        b"OBND" => {
                            let OBND: OBND = parser.read()?;
                            println!("{:?}", OBND);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"LVLC" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"OBND" => {
                            let OBND: OBND = parser.read()?;
                            println!("{:?}", OBND);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"ALCH" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"FULL" => {
                            let FULL = parser.read_lstring(header.size)?;
                            println!("{:?}", FULL);
                        },
                        b"OBND" => {
                            let OBND: OBND = parser.read()?;
                            println!("{:?}", OBND);
                        },
                        b"MODL" => {
                            let MODL = parser.read_zstring(header.size)?;
                            println!("{:?}", MODL);
                        },
                        b"ICON" => {
                            let ICON = parser.read_zstring(header.size)?;
                            println!("{:?}", ICON);
                        },
                        b"EFID" => {
                            let EFID: formid_t = parser.read()?;
                            println!("{:?}", EFID);
                        },
                        b"EFIT" => {
                            let EFIT: EFIT = parser.read()?;
                            println!("{:?}", EFIT);
                        },
                        b"CTDA" => {
                            let CTDA: CTDA = parser.read()?;
                            println!("{:?}", CTDA);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"NOTE" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"FULL" => {
                            let FULL = parser.read_lstring(header.size)?;
                            println!("{:?}", FULL);
                        },
                        b"OBND" => {
                            let OBND: OBND = parser.read()?;
                            println!("{:?}", OBND);
                        },
                        b"MODL" => {
                            let MODL = parser.read_zstring(header.size)?;
                            println!("{:?}", MODL);
                        },
                        b"ICON" => {
                            let ICON = parser.read_zstring(header.size)?;
                            println!("{:?}", ICON);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"PROJ" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"FULL" => {
                            let FULL = parser.read_lstring(header.size)?;
                            println!("{:?}", FULL);
                        },
                        b"OBND" => {
                            let OBND: OBND = parser.read()?;
                            println!("{:?}", OBND);
                        },
                        b"MODL" => {
                            let MODL = parser.read_zstring(header.size)?;
                            println!("{:?}", MODL);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"LVLI" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"OBND" => {
                            let OBND: OBND = parser.read()?;
                            println!("{:?}", OBND);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"REGN" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"NAVI" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"DIAL" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"QUST" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"IDLE" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"PACK" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"CSTY" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"ANIO" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"MODL" => {
                            let MODL = parser.read_zstring(header.size)?;
                            println!("{:?}", MODL);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"WATR" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"FULL" => {
                            let FULL = parser.read_lstring(header.size)?;
                            println!("{:?}", FULL);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"EFSH" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"ICON" => {
                            let ICON = parser.read_zstring(header.size)?;
                            println!("{:?}", ICON);
                        },
                        b"ICO2" => {
                            let ICO2 = parser.read_zstring(header.size)?;
                            println!("{:?}", ICO2);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"EXPL" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"FULL" => {
                            let FULL = parser.read_lstring(header.size)?;
                            println!("{:?}", FULL);
                        },
                        b"OBND" => {
                            let OBND: OBND = parser.read()?;
                            println!("{:?}", OBND);
                        },
                        b"MODL" => {
                            let MODL = parser.read_zstring(header.size)?;
                            println!("{:?}", MODL);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"DEBR" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"IMGS" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"FLST" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"PERK" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"FULL" => {
                            let FULL = parser.read_lstring(header.size)?;
                            println!("{:?}", FULL);
                        },
                        b"DESC" => {
                            let DESC = parser.read_lstring(header.size)?;
                            println!("{:?}", DESC);
                        },
                        b"ICON" => {
                            let ICON = parser.read_zstring(header.size)?;
                            println!("{:?}", ICON);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"BPTD" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"MODL" => {
                            let MODL = parser.read_zstring(header.size)?;
                            println!("{:?}", MODL);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"ADDN" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"OBND" => {
                            let OBND: OBND = parser.read()?;
                            println!("{:?}", OBND);
                        },
                        b"MODL" => {
                            let MODL = parser.read_zstring(header.size)?;
                            println!("{:?}", MODL);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"CAMS" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"CPTH" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"MODL" => {
                            let MODL = parser.read_zstring(header.size)?;
                            println!("{:?}", MODL);
                        },
                        b"CTDA" => {
                            let CTDA: CTDA = parser.read()?;
                            println!("{:?}", CTDA);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"VTYP" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"IPCT" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"MODL" => {
                            let MODL = parser.read_zstring(header.size)?;
                            println!("{:?}", MODL);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"IPDS" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"ARMA" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"FULL" => {
                            let FULL = parser.read_lstring(header.size)?;
                            println!("{:?}", FULL);
                        },
                        b"OBND" => {
                            let OBND: OBND = parser.read()?;
                            println!("{:?}", OBND);
                        },
                        b"MODL" => {
                            let MODL = parser.read_zstring(header.size)?;
                            println!("{:?}", MODL);
                        },
                        b"MOD3" => {
                            let MOD3 = parser.read_zstring(header.size)?;
                            println!("{:?}", MOD3);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"ECZN" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"MESG" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"FULL" => {
                            let FULL = parser.read_lstring(header.size)?;
                            println!("{:?}", FULL);
                        },
                        b"DESC" => {
                            let DESC = parser.read_lstring(header.size)?;
                            println!("{:?}", DESC);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"RGDL" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"NPC_" => { self.skip(size as u64)?; },
            b"WRLD" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"FULL" => {
                            let FULL = parser.read_lstring(header.size)?;
                            println!("{:?}", FULL);
                        },
                        b"CNAM" => {
                            let CNAM: formid_t = parser.read()?;
                            println!("{:?}", CNAM);
                        },
                        b"XXXX" => {
                            let XXXX_size: u32 = parser.read()?;
                            println!();
                            let next: FieldHeader = parser.read()?;
                            indentln!(parser, "{:?}", next);
                            parser.skip(XXXX_size as u64)?;
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"TACT" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"FULL" => {
                            let FULL = parser.read_lstring(header.size)?;
                            println!("{:?}", FULL);
                        },
                        b"OBND" => {
                            let OBND: OBND = parser.read()?;
                            println!("{:?}", OBND);
                        },
                        b"MODL" => {
                            let MODL = parser.read_zstring(header.size)?;
                            println!("{:?}", MODL);
                        },
                        b"VNAM" => {
                            let VNAM: formid_t = parser.read()?;
                            println!("{:?}", VNAM);
                        },
                        b"SCRI" => {
                            let SCRI: formid_t = parser.read()?;
                            println!("{:?}", SCRI);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"ARMO" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"FULL" => {
                            let FULL = parser.read_lstring(header.size)?;
                            println!("{:?}", FULL);
                        },
                        b"OBND" => {
                            let OBND: OBND = parser.read()?;
                            println!("{:?}", OBND);
                        },
                        b"EITM" => {
                            let EITM: formid_t = parser.read()?;
                            println!("{:?}", EITM);
                        },
                        b"ICON" => {
                            let ICON = parser.read_zstring(header.size)?;
                            println!("{:?}", ICON);
                        },
                        b"MODL" => {
                            let MODL = parser.read_zstring(header.size)?;
                            println!("{:?}", MODL);
                        },
                        b"MODS" => {
                            let MODS = parser.read_zstring(header.size)?;
                            println!("{:?}", MODS);
                        },
                        b"MOD2" => {
                            let MOD2 = parser.read_zstring(header.size)?;
                            println!("{:?}", MOD2);
                        },
                        b"MOD3" => {
                            let MOD3 = parser.read_zstring(header.size)?;
                            println!("{:?}", MOD3);
                        },
                        b"MO2S" => {
                            let MO2S = parser.read_zstring(header.size)?;
                            println!("{:?}", MO2S);
                        },
                        b"MO3S" => {
                            let MO3S = parser.read_zstring(header.size)?;
                            println!("{:?}", MO3S);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"DOOR" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"FULL" => {
                            let FULL = parser.read_lstring(header.size)?;
                            println!("{:?}", FULL);
                        },
                        b"OBND" => {
                            let OBND: OBND = parser.read()?;
                            println!("{:?}", OBND);
                        },
                        b"SCRI" => {
                            let SCRI: formid_t = parser.read()?;
                            println!("{:?}", SCRI);
                        },
                        b"MODL" => {
                            let MODL = parser.read_zstring(header.size)?;
                            println!("{:?}", MODL);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"SCOL" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"OBND" => {
                            let OBND: OBND = parser.read()?;
                            println!("{:?}", OBND);
                        },
                        b"MODL" => {
                            let MODL = parser.read_zstring(header.size)?;
                            println!("{:?}", MODL);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"IDLM" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"OBND" => {
                            let OBND: OBND = parser.read()?;
                            println!("{:?}", OBND);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"IMAD" => { self.skip(size as u64)?; },
            b"GRUP" => {
                self.parse_records(ESMParser::GRUP, header.size as u64)?;
                return Ok(())
            },
            b"CELL" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        b"FULL" => {
                            let FULL = parser.read_lstring(header.size)?;
                            println!("{:?}", FULL);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"ACRE" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"EDID" => {
                            let EDID = parser.read_zstring(header.size)?;
                            println!("{:?}", EDID);
                        },
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"NAVM" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"ACHR" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"INFO" => {
                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        _ => {
                            parser.skip(header.size as u64)?;
                            println!("Unknown field '{}'", header.type_id);
                        }
                    }
                    Ok(())
                }, size)?;
            },
            b"REFR" => { self.skip(size as u64)?; },
            _ => {
                self.skip(size as u64)?;
                println!("Unknown record '{}'", type_id);
            },
        }
        Ok(())
    }

    pub fn TES4(&mut self, header: &RecordHeader) -> Result<()> {
        let RecordHeader { type_id, size, .. } = *header;
        match &type_id.0 {
            b"TES4" => {
                println!("{:?} ", header);

                let flags = header.flags;
                if (flags & 0x00000001) != 0 {} // Master (ESM) file
                self.localised = (flags & 0x00000080) != 0;
                if (flags & 0x00000200) != 0 {} // Light Master (ESL) File

                self.parse_fields(|parser, header| {
                    indent!(parser, "{:?} ", header);
                    match &header.type_id.0 {
                        b"HEDR" => {
                            let HEDR: HEDR = parser.read()?;
                            println!("{:?}", HEDR);
                        },
                        b"CNAM" => {
                            let CNAM = parser.read_zstring(header.size)?;
                            println!("{:?}", CNAM);
                        },
                        b"MAST" => {
                            let MAST = parser.read_zstring(header.size)?;
                            println!("{:?}", MAST);
                        },
                        b"DATA" => {
                            let DATA: u64 = parser.read()?;
                            println!("{:?}", DATA);
                        },
                        b"ONAM" => {
                            parser.skip(header.size as u64)?;
                            println!("Unimplemented");
                        },
                        _ => { println!("Unknown typeid '{}'", header.type_id) }
                    }
                    Ok(())
                }, size)?;
            },
            b"GRUP" => {
                let GRUP: GroupHeader = unsafe { std::mem::transmute(*header) };
                println!("{:?}", GRUP);
                self.parse_records(ESMParser::GRUP, size as u64)?;
                return Ok(())
            },
            _ => { println!("Unknown record '{}'", type_id)}
        }
        Ok(())
    }

    pub fn parse_fields(&mut self, f: FieldParser<Self>, total_size: u32) -> Result<()> {
        if total_size == 0 { return Ok(()) }
        let loop_end = self.reader().stream_position()? + total_size as u64;
        self.push();
        match loop {
            let header: FieldHeader = self.read()?;
            let start = self.reader().stream_position()?;
            let size = header.size as u64;
            f(self, &header)?; // parse the contents
            let end = start + size;
            let pos = self.reader().stream_position()?;
            if pos == loop_end { break Ok(()) } // function consumed chunk
            else if pos != end { return Err(chunk_parser::Error::ParseError) } // function made a mistake
        } {
            res => { self.pop(); res }
        }
    }

    fn parse_records(&mut self, f: RecordParser<Self>, total_size: u64) -> Result<()> {
        if total_size == 0 { return Ok(()) }
        let loop_end = self.reader().stream_position()? + total_size as u64;
        self.push();
        match loop {
            let start = self.reader().stream_position()?;
            let mut header: RecordHeader = self.read()?;
            let mut size = header.size as u64;
            if header.type_id != b"GRUP" { size += 24; }
            else { header.size -= 24; }
            f(self, &header)?; // parse the contents
            let end = start + size;
            let pos = self.reader().stream_position()?;
            if pos == loop_end { break Ok(()) } // function consumed chunk
            else if pos != end { return Err(chunk_parser::Error::ParseError) } // function made a mistake
        } {
            res => { self.pop(); res }
        }
    }

    pub fn parse_top_level(&mut self, f: RecordParser<Self>) -> Result<()> {
        let total_size = self.reader().seek(std::io::SeekFrom::End(0))?;
        self.reader().seek(std::io::SeekFrom::Start(0))?;
//        self.pop();
        self.parse_records(f, total_size)?;
//        self.push();
        Ok(())
    }
}

//------------------------------------------------------------------------------

pub mod prelude {
    pub use chunk_parser::prelude::*;
    pub use super::ESMParser;
}

//==============================================================================

#[cfg(test)]
mod tests {
    use super::prelude::*;

    #[test]
    fn zeta() -> chunk_parser::Result<()> {
        const DATA: &[u8] = include_bytes!("../data/Zeta.esm");
        let mut esm = ESMParser::cursor(DATA);
        esm.parse_top_level(ESMParser::TES4)
    }
}
