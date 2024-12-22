use async_trait::async_trait;
use flate2::{Compress, Decompress};
use flate2::Compression;

#[async_trait]
pub trait MessageCompressor: Send + Sync {
    async fn compress(&self, data: &[u8]) -> Result<Vec<u8>, CompressionError>;
    async fn decompress(&self, data: &[u8]) -> Result<Vec<u8>, CompressionError>;
}

pub struct GzipCompressor {
    level: Compression,
}

impl GzipCompressor {
    pub fn new(level: u32) -> Self {
        Self {
            level: Compression::new(level),
        }
    }
}

#[async_trait]
impl MessageCompressor for GzipCompressor {
    async fn compress(&self, data: &[u8]) -> Result<Vec<u8>, CompressionError> {
        let mut compressor = Compress::new(self.level, false);
        let mut compressed = Vec::with_capacity(data.len());
        
        compressor.compress_vec(data, &mut compressed, flate2::FlushCompress::Finish)
            .map_err(|e| CompressionError::CompressionFailed(e.to_string()))?;
            
        Ok(compressed)
    }

    async fn decompress(&self, data: &[u8]) -> Result<Vec<u8>, CompressionError> {
        let mut decompressor = Decompress::new(false);
        let mut decompressed = Vec::with_capacity(data.len() * 2);
        
        decompressor.decompress_vec(data, &mut decompressed, flate2::FlushDecompress::Finish)
            .map_err(|e| CompressionError::DecompressionFailed(e.to_string()))?;
            
        Ok(decompressed)
    }
} 