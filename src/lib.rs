//! # lau-compression
//!
//! Data compression algorithms for efficient encoding of information.
//!
//! Includes: Huffman coding, RLE, LZW, BWT, Move-to-front, Arithmetic coding,
//! Delta encoding, LZ77, and Shannon entropy bounds.

pub mod huffman;
pub mod rle;
pub mod lzw;
pub mod bwt;
pub mod mtf;
pub mod arithmetic;
pub mod delta;
pub mod lz77;
pub mod entropy;
pub mod agent_compress;

pub use huffman::{HuffmanCoder, HuffmanNode};
pub use rle::RleCodec;
pub use lzw::LzwCodec;
pub use bwt::BwtTransform;
pub use mtf::MtfTransform;
pub use arithmetic::ArithmeticCoder;
pub use delta::DeltaCodec;
pub use lz77::Lz77Codec;
pub use entropy::Entropy;
pub use agent_compress::AgentCompressor;
