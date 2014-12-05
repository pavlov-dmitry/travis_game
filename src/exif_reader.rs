extern crate libc;
extern crate num;
use self::libc::{ size_t, c_int };
use self::num::rational::Ratio;
use std::collections::HashMap;
use std::c_str::CString;

pub type ExifEntries = HashMap<String, ExifValue>;

/// вычитывает exif в память в виде таблицы
pub fn from_memory( data: &[u8] ) -> Option<ExifEntries> {
    let exif_data = unsafe {
        exif_data_new_from_data( data.as_ptr(), data.len() as size_t )
    };
    if exif_data != 0 { 
        let byte_order = unsafe{ exif_data_get_byte_order( exif_data ) };
        let mut read_body = ReadBody{ 
            byte_order: byte_order, 
            ifd: 0, 
            entries: HashMap::new() 
        };
        unsafe{ 
            exif_data_foreach_content( exif_data, read_exif_content, &mut read_body );
            exif_data_free( exif_data ); 
        }
        Some( read_body.entries )
    }
    else {
        None
    }
}

///упрощенный доступ к опередленным параметрам exif-a
pub trait ExifValues {
    fn iso(&self) -> Option<u32>;
    fn focal_length(&self) -> Option<u16>;
    fn focal_length_35mm(&self) -> Option<u16>;
    fn aperture(&self) -> Option<f32>;
    fn shutter_speed(&self) -> Option<i32>;
    fn camera_model<'a>(&'a self)->Option<&'a str>;
}

impl ExifValues for ExifEntries {
    fn iso(&self) -> Option<u32> {
        self.get( &"ISOSpeedRatings".to_string() )
            .and_then( |v| v.as_short().map( |v| v.to_u32().unwrap() ) )
    }
    fn focal_length(&self) -> Option<u16> {
        self.get( &"FocalLength".to_string() )
            .and_then( |v| v.as_ratio() )
            .map( |r| r.to_integer() as u16 )
    }
    fn focal_length_35mm(&self) -> Option<u16> {
        self.get( &"FocalLengthIn35mmFilm".to_string() )
            .and_then( |v| v.as_short() )
    }
    fn aperture(&self) -> Option<f32> {
        self.get( &"FNumber".to_string() )
            .and_then( |v| v.as_ratio() )
            .and_then( |r| 
                if *r.denom() != 0 { 
                    Some( r.numer().to_f32().unwrap() / r.denom().to_f32().unwrap() )
                }  
                else { 
                    None 
                } 
            )
    }
    fn shutter_speed(&self) -> Option<i32> {
        self.get( &"ExposureTime".to_string() )
            .and_then( |v| v.as_ratio() )
            .map( |r| 
                if r.is_integer() {
                    r.to_integer() as i32
                }
                else {
                    r.denom().to_i32().unwrap().neg()
                }
            )
    }
    fn camera_model<'a>(&'a self)->Option<&'a str> {
        self.get( &"Model".to_string() )
            .and_then( |v| v.as_text() )
    }
}

#[allow(dead_code)]
#[deriving(Show)]
enum ExifValue {
    Byte( u8 ),
    Text( String ),
    Short( u16 ),
    Long( u32 ),
    Ratio( Ratio<u32> ),
    SByte( i8 ),
    Undefined,
    SShort( i16 ),
    SLong( i32 ),
    SRatio( Ratio<i32> ),
    Float( f32 ),
    Double( f64 ),
    Error
}

impl ExifValue {
	#[allow(dead_code)]
    pub fn as_u8(&self) -> Option<u8>{  
        match self {
            &ExifValue::Byte( v ) => Some( v ),
            _ => None
        }
    }

	#[allow(dead_code)]
    pub fn as_text<'a>(&'a self) -> Option<&'a str> {
        match self {
            &ExifValue::Text( ref v ) => Some( v.as_slice() ),
            _ => None
        }
    }

    #[allow(dead_code)]
    pub fn as_short(&self) -> Option<u16> {
        match self {
            &ExifValue::Short( v ) => Some( v ),
            _ => None
        }
    }

    #[allow(dead_code)]
    pub fn as_long(&self) -> Option<u32> {
        match self {
            &ExifValue::Long( v ) => Some( v ),
            _ => None
        }
    }

    #[allow(dead_code)]
    pub fn as_ratio(&self) -> Option<Ratio<u32>> {
        match self {
            &ExifValue::Ratio( v ) => Some( v ),
            _ => None
        }
    }

    #[allow(dead_code)]
    pub fn is_undefined(&self) -> bool {
        match self {
            &ExifValue::Undefined => true,
            _ => false
        }
    }

    #[allow(dead_code)]
    pub fn as_sshort(&self) -> Option<i16> {
        match self {
            &ExifValue::SShort( v ) => Some( v ),
            _ => None
        }
    }

    #[allow(dead_code)]
    pub fn as_slong(&self) -> Option<i32> {
        match self {
            &ExifValue::SLong( v ) => Some( v ),
            _ => None
        }
    }

    #[allow(dead_code)]
    pub fn as_sratio(&self) -> Option<Ratio<i32>> {
        match self {
            &ExifValue::SRatio( v ) => Some( v ),
            _ => None
        }
    }

	#[allow(dead_code)]
    pub fn as_float(&self) -> Option<f32> {
        match self {
            &ExifValue::Float( v ) => Some( v ),
            _ => None
        }
    }

    #[allow(dead_code)]
    pub fn as_double(&self) -> Option<f64> {
        match self {
            &ExifValue::Double( v ) => Some( v ),
            _ => None
        }
    }
}

//как убрать этот варнинг в моих условиях я не понял, потому просто отключаю его
#[allow(improper_ctypes)] 
#[link(name = "exif", kind = "static")]
extern {
    fn exif_data_new_from_data ( data: *const u8, size: size_t) -> c_int;
    fn exif_data_free( ptr: c_int );
    fn exif_data_foreach_content (exif_data_ptr: c_int, func : extern fn(c_int, *mut ReadBody), user_data: *mut ReadBody);
    fn exif_content_foreach_entry( exif_content_ptr: c_int, func: extern fn( *mut ExifEntry, *mut ReadBody ), user_data: *mut ReadBody );
    fn exif_data_get_byte_order(exif_data_ptr: c_int) -> c_int;
    fn exif_tag_get_name_in_ifd(tag: i32, ifd: c_int) -> *const i8;
    fn exif_content_get_ifd(content: c_int) -> c_int;
    fn exif_get_short (b: *const u8, order: c_int) -> u16;
    fn exif_get_sshort (b: *const u8, order: c_int) -> i16;
    fn exif_get_long (b: *const u8, order: c_int) -> u32;
    fn exif_get_slong (b: *const u8, order: c_int) -> i32;
    fn exif_get_rational (b: *const u8, order: c_int) -> ExifRational;
    fn exif_get_srational (b: *const u8, order: c_int) -> ExifSRational;
}

#[repr(C)]
struct ExifRational {
    num: u32,
    den: u32
}

#[repr(C)]
struct ExifSRational {
    num: i32,
    den: i32
}

#[repr(C)]
struct ExifEntry {
    tag: i32,
    format: ExifFormat,
    components: u32,
    data: *const u8,
    size: uint,
    parent: c_int,
    private: c_int
}

//тут этот варнинг отключать смысла нет, но сцуко видать пока глючит компилятор
#[allow(dead_code)]
#[repr(C)]
enum ExifFormat {
    BYTE = 1, 
    ASCII = 2, 
    SHORT = 3,
    LONG = 4, 
    RATIONAL = 5, 
    SBYTE = 6, 
    UNDEFINED = 7, 
    SSHORT = 8, 
    SLONG = 9, 
    SRATIONAL = 10, 
    FLOAT = 11, 
    DOUBLE = 12
}

fn to_exif_value( entry: &ExifEntry, byte_order: c_int ) -> ExifValue {
    if entry.data.is_null() {
        return ExifValue::Error;
    }
    match entry.format {
        ExifFormat::BYTE => {
            let data = unsafe{ entry.data.as_ref().unwrap() };
            ExifValue::Byte( *data )
        },
        ExifFormat::ASCII => {
            let name_cstr = unsafe{ CString::new( entry.data as *const i8, false ) };
            match name_cstr.as_str() {
                Some( s ) => ExifValue::Text( s.to_string() ),
                None => ExifValue::Text( "bad ASCII".to_string() )
            }
        },
        ExifFormat::SHORT => ExifValue::Short( unsafe{ exif_get_short( entry.data, byte_order ) } ),
        ExifFormat::LONG => ExifValue::Long( unsafe{ exif_get_long( entry.data, byte_order ) } ),
        ExifFormat::RATIONAL => {
            let rat = unsafe{ exif_get_rational( entry.data, byte_order ) };
            ExifValue::Ratio( Ratio::new_raw( rat.num, rat.den ) )    
        },
        ExifFormat::SBYTE => {
            let data = unsafe{ entry.data.as_ref().unwrap() };
            ExifValue::SByte( *data as i8 ) 
        },
        ExifFormat::SSHORT => ExifValue::SShort( unsafe{ exif_get_sshort( entry.data, byte_order ) } ),
        ExifFormat::SLONG => ExifValue::SLong( unsafe{ exif_get_slong( entry.data, byte_order ) } ),
        ExifFormat::SRATIONAL => {
            let rat = unsafe{ exif_get_srational( entry.data, byte_order ) };
            ExifValue::SRatio( Ratio::new_raw( rat.num, rat.den ) )   
        },
        _ => ExifValue::Undefined
    }
}

#[repr(C)]
struct ReadBody {
    byte_order: c_int,
    ifd: c_int,
    entries: ExifEntries 
}

extern fn read_exif_entry( e: *mut ExifEntry, b: *mut ReadBody ) {
    let entry = unsafe{ e.as_ref().unwrap() };
    let body = unsafe{ b.as_mut().unwrap() };
    let name_cstr = unsafe{ CString::new( exif_tag_get_name_in_ifd( entry.tag, body.ifd ), false ) };
    match name_cstr.as_str() {
        Some( name_utf8 ) => {
            let value = to_exif_value( entry, body.byte_order );
            body.entries.insert( name_utf8.to_string(), value );
        }
        None => {}
    }
}

extern fn read_exif_content( content: c_int, b: *mut ReadBody ) {
    let body = unsafe{ b.as_mut().unwrap() };
    body.ifd = unsafe{ exif_content_get_ifd( content ) };
    unsafe{ exif_content_foreach_entry( content, read_exif_entry, b ); }
}