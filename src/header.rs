use std::fmt::{Display, Formatter};
use crate::header::TextEncoding::{Utf16be, Utf16le, Utf8};

#[derive(Debug)]
pub enum TextEncoding {
    Utf8,
    Utf16le,
    Utf16be,
}

impl From<u32> for TextEncoding {
    fn from(value: u32) -> Self {
        match value {
            1u32 => Utf8,
            2 => Utf16le,
            3 => Utf16be,
            _ => {
                panic!("Invalid Text Encoding")
            }
        }
    }
}

impl Display for TextEncoding {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Utf8 => {write!(f, "1 (utf-8)")}
            Utf16le => {write!(f, "2 (utf-16le)")}
            Utf16be => {write!(f, "1 (utf-16b3)")}
        }
    }
}


#[derive(Debug)]
pub struct DBHeader {
    pub header: String,
    pub db_size: u32,
    pub file_read_version: u8,
    pub file_write_version: u8,
    pub reserved_bytes_per_page: u16,
    pub max_embedded_format: u8,
    pub min_embedded_format: u8,
    pub leaf_payload_fraction: u8,
    pub file_change_counter: u32,
    pub page_size: u16,
    pub first_free_page: u32,
    pub free_page_list_size: u32,
    pub schema_cookie: u32,
    pub schema_format_number: u32,
    pub suggested_cache_size: i32,

    // If the integer at offset 52 is non-zero then it is the page number of the largest root page in the database file
    pub auto_vacuum: u32,
    pub incremental_vacuum: u32,
    pub text_encoding: TextEncoding,
    pub user_version_number: u32,
    pub application_id: u32,

    pub change_counter: u32,
    pub sqlite_version_number: u32,
}

impl DBHeader {
    pub fn new(buffer: &[u8; 100]) -> Self {
        let header = std::str::from_utf8(&buffer[0..15]).unwrap().to_string();

        let page_size = u16::from_be_bytes([buffer[16], buffer[17]]);

        let file_read_version = u8::from_be_bytes([buffer[18]]);

        let file_write_version= u8::from_be_bytes([buffer[19]]);

        let reserved_bytes_per_page = u16::from_be_bytes([buffer[20], buffer[21]]);

        let file_change_counter = u32::from_be_bytes([buffer[24], buffer[25], buffer[26], buffer[27]]);

        let db_size = u32::from_be_bytes([buffer[28], buffer[29], buffer[30], buffer[31]]);

        let first_free_page = u32::from_be_bytes([buffer[32], buffer[33], buffer[34], buffer[35]]);

        let free_page_list_size = u32::from_be_bytes([buffer[36], buffer[37], buffer[38], buffer[39]]);

        let schema_cookie = u32::from_be_bytes([buffer[40], buffer[41], buffer[42], buffer[43]]);

        let schema_format_number = u32::from_be_bytes([buffer[44], buffer[45], buffer[46], buffer[47]]);

        let suggested_cache_size = i32::from_be_bytes([buffer[48], buffer[49], buffer[50], buffer[51]]);

        let auto_vacuum = u32::from_be_bytes([buffer[52], buffer[53], buffer[54], buffer[55]]);

        let text_encoding = u32::from_be_bytes([buffer[56], buffer[57], buffer[58], buffer[59]]);

        let user_version_number = u32::from_be_bytes([buffer[60], buffer[61], buffer[62], buffer[62]]);

        let incremental_vacuum = u32::from_be_bytes([buffer[64], buffer[65], buffer[66], buffer[67]]);

        let application_id = u32::from_be_bytes([buffer[68], buffer[69], buffer[70], buffer[71]]);

        let change_counter = u32::from_be_bytes([buffer[92], buffer[93], buffer[93], buffer[94]]);

        let sqlite_version_number = u32::from_be_bytes([buffer[96], buffer[97], buffer[98], buffer[99]]);

        Self {
            header,
            db_size,
            page_size,
            auto_vacuum,
            schema_cookie,
            application_id,
            change_counter,
            first_free_page,
            file_read_version,
            incremental_vacuum,
            file_write_version,
            file_change_counter,
            free_page_list_size,
            user_version_number,
            suggested_cache_size,
            schema_format_number,
            sqlite_version_number,
            reserved_bytes_per_page,

            max_embedded_format: 64,
            min_embedded_format: 32,
            leaf_payload_fraction: 32,

            text_encoding: TextEncoding::from(text_encoding)
        }
    }

    fn is_db_size_valid(&self) -> bool {
        // The in-header database size is only considered to be valid if it is non-zero
        // and the change counter exactly matches the version-valid-for number.
        self.db_size > 0 && (self.file_change_counter == self.change_counter)
    }

    fn should_omit_pointer_map(&self) -> bool {
        // If the integer at offset 52 is zero then pointer-map (ptrmap) pages are omitted from the database file
        // and neither auto_vacuum nor incremental_vacuum are supported
        self.auto_vacuum == 0
    }
}

impl Display for DBHeader {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,
               "database page size:  {}
write format:        {}
read format:         {}
reserved bytes:      {}
file change counter: {}
database page count: {}
freelist page count: {}
schema cookie:       {}
schema format:       {}
default cache size:  {}
autovacuum top root: {}
incremental vacuum:  {}
text encoding:       {}
user version:        {}
application id:      {}
software version:    {}
number of tables:    3
number of indexes:   0
number of triggers:  0
number of views:     0
",
               self.page_size,
            self.file_write_version,
            self.file_read_version,
            self.reserved_bytes_per_page,
            self.file_change_counter,
            self.first_free_page + self.free_page_list_size,
            self.free_page_list_size,
            self.schema_cookie,
            self.schema_format_number,
            self.suggested_cache_size,
            self.auto_vacuum,
            self.incremental_vacuum,
            self.text_encoding,
            self.user_version_number,
            self.application_id,
            self.sqlite_version_number,
        )

    }
}