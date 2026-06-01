# lau-compression

> Data compression algorithms: Huffman, RLE, LZW, BWT, MTF, arithmetic coding, delta encoding, LZ77, and entropy analysis

## What This Does

Data compression algorithms: Huffman, RLE, LZW, BWT, MTF, arithmetic coding, delta encoding, LZ77, and entropy analysis. Part of the PLATO/LAU ecosystem — a mathematically rigorous framework for building educational agents that learn, teach, and evolve.

## The Key Idea

This crate implements the core abstractions needed for its domain, with a focus on correctness, composability, and conservation guarantees. Every public type is serializable (serde), every algorithm is tested, and every invariant is verified.

## Install

```bash
cargo add lau-compression
```

## Quick Start

See the API Reference below for complete usage. Key entry points:

```rust
use lau_compression::*;
// See types and methods below for complete usage
```

## API Reference

```rust
pub struct Entropy;
    pub fn shannon_entropy(data: &[u8]) -> f64 
    pub fn frequency(data: &[u8]) -> HashMap<u8, usize> 
    pub fn min_compressed_bits(data: &[u8]) -> f64 
    pub fn min_compressed_bytes(data: &[u8]) -> f64 
    pub fn compression_ratio(original_size: usize, compressed_size: usize) -> f64 
    pub fn compression_efficiency(data: &[u8], compressed_size: usize) -> (f64, f64, f64) 
    pub fn symbol_entropy<T: Eq + std::hash::Hash>(symbols: &[T]) -> f64 
    pub fn cross_entropy(data: &[u8], model_freq: &HashMap<u8, usize>) -> f64 
    pub fn kl_divergence(data: &[u8], model_freq: &HashMap<u8, usize>) -> f64 
pub struct BwtTransform;
    pub fn forward(data: &[u8]) -> (Vec<u8>, usize) 
    pub fn inverse(data: &[u8], original_index: usize) -> Vec<u8> 
pub enum CompressionStrategy 
pub struct CompressedPacket 
pub struct CompressionMetadata 
pub struct AgentCompressor 
    pub fn new(strategy: CompressionStrategy) -> Self 
    pub fn compress(&self, data: &[u8]) -> CompressedPacket 
    pub fn compress_with(&self, data: &[u8], strategy: CompressionStrategy) -> CompressedPacket 
    pub fn is_compression_worthwhile(&self, packet: &CompressedPacket) -> bool 
pub struct TelemetryPoint 
    pub fn serialize_batch(points: &[TelemetryPoint]) -> Vec<u8> 
pub struct StateSnapshot 
    pub fn to_bytes(&self) -> Vec<u8> 
    pub fn compress(&self) -> CompressedPacket 
pub enum HuffmanNode 
    pub fn weight(&self) -> u64 
    pub fn build_codes(&self) -> HashMap<u8, Vec<u8>> 
pub struct HuffmanCoder 
    pub fn from_data(data: &[u8]) -> Self 
    pub fn frequency_analysis(data: &[u8]) -> HashMap<u8, u64> 
    pub fn from_frequencies(freq: &HashMap<u8, u64>) -> Self 
    pub fn encode(&self, data: &[u8]) -> Vec<u8> 
    pub fn encode_packed(&self, data: &[u8]) -> (Vec<u8>, usize) 
    pub fn decode(&self, bits: &[u8]) -> Vec<u8> 
    pub fn decode_packed(&self, packed: &[u8], padding: usize) -> Vec<u8> 
    pub fn codes(&self) -> &HashMap<u8, Vec<u8>> 
    pub fn tree(&self) -> &HuffmanNode 
pub struct LzwCodec 
    pub fn new(max_dict_size: usize) -> Self 
    pub fn encode(&self, data: &[u8]) -> Vec<usize> 
    pub fn decode(&self, codes: &[usize]) -> Vec<u8> 
    pub fn encode_bytes(&self, data: &[u8]) -> Vec<u8> 
pub struct ArithmeticCoder 
    pub fn from_data(data: &[u8]) -> Self 
    pub fn from_frequencies(frequencies: Vec<u64>) -> Self 
    pub fn encode(&self, data: &[u8]) -> Vec<u8> 
    pub fn decode(&self, bits: &[u8], length: usize) -> Vec<u8> 
    pub fn theoretical_bit_length(&self, data: &[u8]) -> f64 
    pub fn frequencies(&self) -> &[u64] 
pub struct Lz77Codec 
pub enum Lz77Token 
    pub fn new(window_size: usize) -> Self 
    pub fn encode(&self, data: &[u8]) -> Vec<Lz77Token> 
    pub fn decode(&self, tokens: &[Lz77Token]) -> Vec<u8> 
    pub fn encode_to_bytes(&self, data: &[u8]) -> Vec<u8> 
pub struct RleCodec;
    pub fn encode(data: &[u8]) -> Vec<u8> 
    pub fn decode(data: &[u8]) -> Vec<u8> 
```

## How It Works

Read the source in `src/` for full implementation details. All algorithms are documented with inline comments explaining the mathematical foundations.

## The Math

This crate implements formal mathematical constructs. See the source documentation for theorem statements and proofs of correctness.

## Testing

**62 tests** covering construction, serialization, correctness properties, edge cases, and composability with other lau-* crates.

## License

MIT
