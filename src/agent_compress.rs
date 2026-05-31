//! Agent data compression: compressing agent telemetry and state snapshots.

use serde::{Deserialize, Serialize};
use crate::huffman::HuffmanCoder;
use crate::rle::RleCodec;
use crate::delta::DeltaCodec;
use crate::entropy::Entropy;
use crate::lz77::Lz77Codec;
use crate::bwt::BwtTransform;
use crate::mtf::MtfTransform;

/// Compression strategy for agent data.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum CompressionStrategy {
    /// Huffman only.
    Huffman,
    /// LZ77 only.
    Lz77,
    /// Full pipeline: BWT + MTF + Huffman.
    FullPipeline,
    /// Delta + Huffman (for telemetry sequences).
    DeltaHuffman,
    /// Auto-select based on entropy analysis.
    Auto,
}

/// Compressed agent data packet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressedPacket {
    pub strategy: CompressionStrategy,
    pub compressed: Vec<u8>,
    pub original_size: usize,
    pub metadata: CompressionMetadata,
}

/// Metadata about the compression result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionMetadata {
    pub entropy: f64,
    pub compression_ratio: f64,
    pub theoretical_max_ratio: f64,
}

/// Agent data compressor for telemetry and state snapshots.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCompressor {
    pub default_strategy: CompressionStrategy,
}

impl Default for AgentCompressor {
    fn default() -> Self {
        AgentCompressor {
            default_strategy: CompressionStrategy::Auto,
        }
    }
}

impl AgentCompressor {
    pub fn new(strategy: CompressionStrategy) -> Self {
        AgentCompressor { default_strategy: strategy }
    }

    /// Compress data using the default strategy.
    pub fn compress(&self, data: &[u8]) -> CompressedPacket {
        let strategy = match self.default_strategy {
            CompressionStrategy::Auto => self.select_strategy(data),
            s => s,
        };
        self.compress_with(data, strategy)
    }

    /// Auto-select the best strategy based on data characteristics.
    fn select_strategy(&self, data: &[u8]) -> CompressionStrategy {
        if data.is_empty() {
            return CompressionStrategy::Huffman;
        }

        let entropy = Entropy::shannon_entropy(data);

        // Low entropy: lots of repetition, use full pipeline
        if entropy < 3.0 {
            CompressionStrategy::FullPipeline
        } else if entropy < 6.0 {
            CompressionStrategy::Lz77
        } else {
            CompressionStrategy::Huffman
        }
    }

    /// Compress with a specific strategy.
    pub fn compress_with(&self, data: &[u8], strategy: CompressionStrategy) -> CompressedPacket {
        let compressed = match strategy {
            CompressionStrategy::Huffman => {
                let coder = HuffmanCoder::from_data(data);
                let (packed, _) = coder.encode_packed(data);
                packed
            }
            CompressionStrategy::Lz77 => {
                let codec = Lz77Codec::default();
                codec.encode_to_bytes(data)
            }
            CompressionStrategy::FullPipeline => {
                // BWT -> MTF -> Huffman
                let (bwt, idx) = BwtTransform::forward(data);
                let mtf = MtfTransform::forward(&bwt);
                // Store idx and original length as prefix
                let idx_bytes = (idx as u32).to_le_bytes();
                let len_bytes = (data.len() as u32).to_le_bytes();
                let coder = HuffmanCoder::from_data(&mtf);
                let (packed, _) = coder.encode_packed(&mtf);
                let mut result = Vec::new();
                result.extend_from_slice(&len_bytes);
                result.extend_from_slice(&idx_bytes);
                result.extend_from_slice(&packed);
                result
            }
            CompressionStrategy::DeltaHuffman => {
                let deltas = DeltaCodec::encode(data);
                let delta_bytes: Vec<u8> = deltas.iter().flat_map(|&d| d.to_le_bytes()).collect();
                let coder = HuffmanCoder::from_data(&delta_bytes);
                let (packed, _) = coder.encode_packed(&delta_bytes);
                packed
            }
            CompressionStrategy::Auto => unreachable!(),
        };

        let entropy = Entropy::shannon_entropy(data);
        let compression_ratio = if !compressed.is_empty() {
            data.len() as f64 / compressed.len() as f64
        } else {
            1.0
        };
        let min_bits = Entropy::min_compressed_bits(data);
        let theoretical_max = if min_bits > 0.0 {
            (data.len() * 8) as f64 / min_bits
        } else {
            f64::INFINITY
        };

        CompressedPacket {
            strategy,
            compressed,
            original_size: data.len(),
            metadata: CompressionMetadata {
                entropy,
                compression_ratio,
                theoretical_max_ratio: theoretical_max,
            },
        }
    }

    /// Check if compression was worthwhile.
    pub fn is_compression_worthwhile(&self, packet: &CompressedPacket) -> bool {
        packet.compressed.len() < packet.original_size
    }
}

/// Agent telemetry data point.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryPoint {
    pub timestamp: u64,
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub event_type: u8,
}

impl TelemetryPoint {
    /// Serialize telemetry points to bytes for compression.
    pub fn serialize_batch(points: &[TelemetryPoint]) -> Vec<u8> {
        let mut data = Vec::with_capacity(points.len() * 17);
        for p in points {
            data.extend_from_slice(&p.timestamp.to_le_bytes());
            data.extend_from_slice(&p.cpu_usage.to_le_bytes());
            data.extend_from_slice(&p.memory_usage.to_le_bytes());
            data.push(p.event_type);
        }
        data
    }
}

/// Agent state snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSnapshot {
    pub version: u32,
    pub state_data: Vec<u8>,
}

impl StateSnapshot {
    /// Serialize to bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(&self.version.to_le_bytes());
        data.extend_from_slice(&(self.state_data.len() as u32).to_le_bytes());
        data.extend_from_slice(&self.state_data);
        data
    }

    /// Compress the state snapshot.
    pub fn compress(&self) -> CompressedPacket {
        let compressor = AgentCompressor::new(CompressionStrategy::Auto);
        compressor.compress(&self.to_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_compress_huffman() {
        let compressor = AgentCompressor::new(CompressionStrategy::Huffman);
        let data = b"agent telemetry data for testing purposes".to_vec();
        let packet = compressor.compress(&data);
        assert_eq!(packet.original_size, data.len());
        assert_eq!(packet.strategy, CompressionStrategy::Huffman);
    }

    #[test]
    fn test_agent_compress_lz77() {
        let compressor = AgentCompressor::new(CompressionStrategy::Lz77);
        let data = b"repetitive repetitive repetitive repetitive data".to_vec();
        let packet = compressor.compress(&data);
        assert_eq!(packet.original_size, data.len());
    }

    #[test]
    fn test_agent_compress_auto() {
        let compressor = AgentCompressor::default();
        let data = vec![5; 1000]; // Very low entropy
        let packet = compressor.compress(&data);
        assert!(packet.metadata.entropy < 1.0);
    }

    #[test]
    fn test_agent_compress_empty() {
        let compressor = AgentCompressor::default();
        let packet = compressor.compress(&[]);
        assert_eq!(packet.original_size, 0);
    }

    #[test]
    fn test_telemetry_serialization() {
        let points: Vec<TelemetryPoint> = (0..10).map(|i| TelemetryPoint {
            timestamp: i * 100,
            cpu_usage: i as f32 * 10.0,
            memory_usage: 50.0,
            event_type: i as u8,
        }).collect();
        let serialized = TelemetryPoint::serialize_batch(&points);
        assert_eq!(serialized.len(), 10 * 17);
    }

    #[test]
    fn test_state_snapshot_compress() {
        let snapshot = StateSnapshot {
            version: 1,
            state_data: vec![42; 500],
        };
        let packet = snapshot.compress();
        assert!(packet.compressed.len() > 0);
    }

    #[test]
    fn test_worthwhile_check() {
        let compressor = AgentCompressor::default();
        let data = vec![0; 10000];
        let packet = compressor.compress(&data);
        // Highly repetitive data should compress well
        assert!(compressor.is_compression_worthwhile(&packet));
    }
}
