extern crate crc;
extern crate png;

use std::fs;
use std::mem;
use std::process;
use std::io::prelude::*;
use crc::crc32;
use png::{StreamingDecoder, DecodingError, Decoded};

/// 储存用来校验 crc 值的数据和 crc 值本身
#[derive(Debug)]
pub struct CrcData {
    pub type_str: [u8; 4],
    pub width : u32,
    pub height: u32,
    pub bits: u8,
    pub color_type: u8,
    pub compr_method : u8,
    pub filter_method: u8,
    pub interlace_method: u8,
    pub crc_val: u32,
}


impl CrcData {
    /// 将 CrcData 转化为字节数组
    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];
        let bwidth:  [u8; 4] = unsafe { mem::transmute(self.width) };
        let bheight: [u8; 4] = unsafe { mem::transmute( self.height) };

        bytes.extend(self.type_str.iter());
        bytes.extend(bwidth.iter().rev());
        bytes.extend(bheight.iter().rev());
        bytes.extend([self.bits, self.color_type, self.compr_method, self.filter_method, self.interlace_method].iter());

        bytes
    }
}

/// 从指定位置替换数据
pub fn replace_nbytes(src: &mut Vec<u8>, offset: usize, data: &[u8]) {
    for i in 0..data.len() {
        src[offset + i] = data[i];
    }
}

/// 爆破 crc32 值
pub fn crack_crc(crcdata: &mut CrcData) -> Result<(), ()> {
    let width = crcdata.width;

    for i in 1..8192 {
        crcdata.width = i;
        if crcdata.crc_val == crc32::checksum_ieee(&crcdata.as_bytes()) {
            return Ok(());
        }
    }
    crcdata.width = width;
    for i in 1..8192 {
        crcdata.height = i;
        if crcdata.crc_val == crc32::checksum_ieee(&crcdata.as_bytes()) {
            return Ok(());
        }
    }
    Err(())
}

/// 保存文件
pub fn save_file(filename: &str, data: &[u8]) -> std::io::Result<()> {
    let mut file = fs::File::create(filename)?;
    file.write_all(data)?;
    Ok(())
}

/// 解析获取 crcdata
pub fn get_crcdata(data: &[u8], crcdata: &mut CrcData) {
    let mut decoder = StreamingDecoder::new();
    let mut idx = 0;

    for _ in 0..3 {
        let (len, decoded) = match decoder.update(&data[idx..idx+1000]) {
            Ok(t) => t,
            Err(e) => match e {
                DecodingError::CrcMismatch {crc_val, ..} => {
                    crcdata.crc_val = crc_val;
                    (1, Decoded::Nothing)
                }
                _ => {
                    eprintln!("Problem parsing file: {}", e);
                    process::exit(1);
                }
            }
        };

        match decoded {
            Decoded::ChunkBegin(_length, type_str) => {
                for i in 0..4 {
                    crcdata.type_str[i] = type_str[i];
                }
            }
            Decoded::Header(width, height, bit_depth, color_type, interlaced) => {
                crcdata.width = width;
                crcdata.height = height;
                crcdata.bits = bit_depth as u8;
                crcdata.color_type = color_type as u8;
                crcdata.interlace_method = interlaced as u8;
            }
            _ => ()
        }

        idx += len;
    }
}