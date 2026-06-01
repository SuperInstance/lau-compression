# lau-compression

> A teaching-quality Rust library implementing nine classic data-compression algorithms plus an entropy analysis toolkit and a higher-level agent-data compressor — all fully round-trip tested.

**62 tests · zero unsafe · serde-serializable · `cargo add lau-compression`**

---

## What This Does

`lau-compression` is a crate from the PLATO/LAU ecosystem that provides clean, composable implementations of the fundamental algorithms taught in every information-theory and data-compression course:

| Module | Algorithm | One-line summary |
|---|---|---|
| `huffman` | Huffman coding | Variable-length prefix codes from a frequency table |
| `rle` | Run-Length Encoding (RLE) | Collapse consecutive repeated bytes into (count, value) pairs |
| `lzw` | Lempel-Ziv-Welch (LZW) | Dictionary-based compression that builds its codebook on the fly |
| `lz77` | LZ77 sliding window | Back-reference compression with a sliding window |
| `bwt` | Burrows-Wheeler Transform | Reversible permutation that clusters similar characters |
| `mtf` | Move-to-Front Transform | Converts BWT output into a stream of small integers |
| `arithmetic` | Arithmetic coding | Encodes an entire message as a single fractional number in [0, 1) |
| `delta` | Delta + ZigZag + VarInt encoding | Stores differences between consecutive values, great for time-series |
| `entropy` | Shannon entropy & KL divergence | Theoretical lower bounds on how small any lossless compressor can go |
| `agent_compress` | `AgentCompressor` | A higher-level wrapper that auto-selects the best strategy for agent telemetry |

---

## The Key Idea

Compression is really about **finding and removing redundancy**. Every algorithm in this crate attacks a different *kind* of redundancy:

* **Statistical redundancy** (some symbols appear more often) → Huffman, Arithmetic coding
* **Repetition redundancy** (the same byte repeats) → RLE
* **Dictionary redundancy** (the same substring appears later) → LZW, LZ77
* **Sequential redundancy** (nearby values are correlated) → Delta encoding
* **Permutation redundancy** (rearranging data clusters similar bytes) → BWT + MTF

The `entropy` module gives you the **information-theoretic yardstick**: Shannon entropy tells you the absolute minimum number of bits per symbol that *any* lossless compressor can achieve, no matter how clever. The `compression_efficiency()` function compares your actual result against that theoretical limit.

The `agent_compress` module ties everything together into a practical API: feed it raw bytes, and it analyses the entropy to pick between Huffman-only, LZ77-only, a full BWT→MTF→Huffman pipeline, or Delta+Huffman, then returns a serialisable `CompressedPacket` with metadata (entropy, compression ratio, theoretical maximum ratio).

---

## Install

```toml
# Cargo.toml
[dependencies]
lau-compression = "0.1"
```

Or via the CLI:

```bash
cargo add lau-compression
```

### Dependencies

| Crate | Why |
|---|---|
| `serde` (with `derive`) | All public types are serialisable — save trees, packets, and snapshots to any serde format |
| `nalgebra` | Linear-algebra types used by other LAU crates; re-exported for interop |

---

## Quick Start

```rust
use lau_compression::*;

// --- Huffman ---
let data = b"abracadabra";
let coder = HuffmanCoder::from_data(data);
let (packed, padding) = coder.encode_packed(data);
let recovered = coder.decode_packed(&packed, padding);
assert_eq!(recovered, data);

// --- Entropy analysis ---
let entropy = Entropy::shannon_entropy(data);
println!("Shannon entropy: {entropy:.3} bits/symbol");
let min_bits = Entropy::min_compressed_bits(data);
println!("Theoretical minimum: {min_bits:.1} bits total");

// --- Agent compressor (auto-selects strategy) ---
let compressor = AgentCompressor::default(); // Auto strategy
let packet = compressor.compress(data);
println!("Strategy used: {:?}", packet.strategy);
println!("Original {} bytes → {} bytes (ratio {:.2}x)",
    packet.original_size, packet.compressed.len(), packet.metadata.compression_ratio);
```

---

## API Reference

### `Entropy` — Information-theoretic analysis

```rust
pub struct Entropy;

// Shannon entropy in bits/symbol (0 for constant data, 8 for uniformly random bytes)
Entropy::shannon_entropy(data: &[u8]) -> f64

// Frequency distribution
Entropy::frequency(data: &[u8]) -> HashMap<u8, usize>

// Theoretical lower bounds
Entropy::min_compressed_bits(data: &[u8]) -> f64
Entropy::min_compressed_bytes(data: &[u8]) -> f64

// How good is your compressor? Returns (actual_ratio, theoretical_max_ratio, efficiency)
Entropy::compression_efficiency(data: &[u8], compressed_size: usize) -> (f64, f64, f64)

// Works on any hashable symbol, not just bytes
Entropy::symbol_entropy<T: Eq + Hash>(symbols: &[T]) -> f64

// Cross-entropy and KL divergence against a model distribution
Entropy::cross_entropy(data: &[u8], model_freq: &HashMap<u8, usize>) -> f64
Entropy::kl_divergence(data: &[u8], model_freq: &HashMap<u8, usize>) -> f64
```

### `HuffmanCoder` — Variable-length prefix codes

```rust
// Build from data or a pre-computed frequency table
HuffmanCoder::from_data(data: &[u8]) -> Self
HuffmanCoder::from_frequencies(freq: &HashMap<u8, u64>) -> Self
HuffmanCoder::frequency_analysis(data: &[u8]) -> HashMap<u8, u64>

// Encode
coder.encode(data: &[u8]) -> Vec<u8>           // bit stream (0/1 per byte)
coder.encode_packed(data: &[u8]) -> (Vec<u8>, usize)  // packed bytes + padding

// Decode
coder.decode(bits: &[u8]) -> Vec<u8>
coder.decode_packed(packed: &[u8], padding: usize) -> Vec<u8>

// Introspection
coder.codes() -> &HashMap<u8, Vec<u8>>  // code table
coder.tree() -> &HuffmanNode             // the full tree
```

### `ArithmeticCoder` — Fractional-interval coding

```rust
// Build from data or a raw frequency table
ArithmeticCoder::from_data(data: &[u8]) -> Self
ArithmeticCoder::from_frequencies(frequencies: Vec<u64>) -> Self

// Encode/decode (bits are 0/1 per byte)
coder.encode(data: &[u8]) -> Vec<u8>
coder.decode(bits: &[u8], length: usize) -> Vec<u8>

// Theoretical bit length
coder.theoretical_bit_length(data: &[u8]) -> f64
```

### `RleCodec` — Run-length encoding

```rust
// Output format: pairs of (count, value); runs > 255 are split
RleCodec::encode(data: &[u8]) -> Vec<u8>
RleCodec::decode(data: &[u8]) -> Vec<u8>
```

### `LzwCodec` — Dictionary-based compression

```rust
let codec = LzwCodec::default(); // 256-entry initial dict, 4096 max
// or
let codec = LzwCodec::new(8192);

// Encode returns dictionary indices
codec.encode(data: &[u8]) -> Vec<usize>
codec.decode(codes: &[usize]) -> Vec<u8>

// Encode to packed bytes (variable-width codes)
codec.encode_bytes(data: &[u8]) -> Vec<u8>
```

### `Lz77Codec` — Sliding-window compression

```rust
let codec = Lz77Codec::default(); // 4096 window, match length 3..258
// or
let codec = Lz77Codec::new(window_size);

// Encode returns a token stream (literals + back-references)
codec.encode(data: &[u8]) -> Vec<Lz77Token>
codec.decode(tokens: &[Lz77Token]) -> Vec<u8>

// Encode to a compact byte representation
codec.encode_to_bytes(data: &[u8]) -> Vec<u8>
```

**`Lz77Token`** is an enum:
```rust
pub enum Lz77Token {
    Literal(u8),
    Reference { distance: usize, length: usize },
}
```

### `BwtTransform` — Burrows-Wheeler Transform

```rust
// Forward: returns (transformed_data, original_index)
BwtTransform::forward(data: &[u8]) -> (Vec<u8>, usize)

// Inverse: give it the transformed data and the original_index
BwtTransform::inverse(data: &[u8], original_index: usize) -> Vec<u8>
```

### `MtfTransform` — Move-to-front transform

```rust
MtfTransform::forward(data: &[u8]) -> Vec<u8>
MtfTransform::inverse(data: &[u8]) -> Vec<u8>
```

### `DeltaCodec` — Delta + ZigZag + VarInt

```rust
// Simple delta: stores first value + consecutive differences as i16
DeltaCodec::encode(data: &[u8]) -> Vec<i16>
DeltaCodec::decode(deltas: &[i16]) -> Vec<u8>

// ZigZag + VarInt: compact byte representation of signed deltas
// ZigZag maps: 0→0, -1→1, 1→2, -2→3, ...
DeltaCodec::encode_zigzag(data: &[u8]) -> Vec<u8>
DeltaCodec::decode_zigzag(data: &[u8], original_len: usize) -> Vec<u8>
```

### `AgentCompressor` — High-level auto-compressor

```rust
let compressor = AgentCompressor::default(); // Auto strategy
// or pick one:
let compressor = AgentCompressor::new(CompressionStrategy::FullPipeline);

let packet: CompressedPacket = compressor.compress(data);

// Strategies
pub enum CompressionStrategy {
    Huffman,        // Huffman only
    Lz77,           // LZ77 only
    FullPipeline,   // BWT → MTF → Huffman
    DeltaHuffman,   // Delta encoding + Huffman
    Auto,           // Pick best based on entropy
}

// CompressedPacket fields
packet.strategy                  // which strategy was used
packet.compressed                // Vec<u8> compressed data
packet.original_size             // original byte count
packet.metadata.entropy          // Shannon entropy of input
packet.metadata.compression_ratio // original / compressed
packet.metadata.theoretical_max_ratio // best possible ratio

// Convenience
compressor.is_compression_worthwhile(&packet) -> bool
```

**Telemetry helpers:**
```rust
// Batch-serialize agent telemetry for compression
TelemetryPoint::serialize_batch(points: &[TelemetryPoint]) -> Vec<u8>

// Compress a full state snapshot
StateSnapshot { version: 1, state_data: vec![...] }.compress() -> CompressedPacket
```

---

## How It Works

### Huffman Coding

1. Count symbol frequencies in the input.
2. Build a min-heap of leaf nodes weighted by frequency.
3. Repeatedly merge the two lightest nodes into an internal node — this is the classic greedy algorithm that produces an optimal prefix-free code.
4. Walk the tree to assign `0` for left, `1` for right, giving each symbol a variable-length code.
5. Encode by replacing each symbol with its code; decode by walking the tree bit-by-bit.

**Why it's optimal:** Huffman codes satisfy the *prefix-free* property (no code is a prefix of another), so decoding is unambiguous. Among all prefix-free codes, Huffman minimises the expected code length for a given symbol distribution.

### Run-Length Encoding

Dead simple: scan the input, count consecutive identical bytes, emit `(count, value)` pairs. Runs longer than 255 are split. Expands data that has no runs — that's why it's usually combined with other techniques.

### LZW Compression

1. Start with a dictionary of all single-byte values (indices 0–255).
2. Read the longest prefix `W` already in the dictionary.
3. Emit the dictionary index for `W`; add `W + next_byte` as a new entry.
4. Repeat. When the dictionary fills up (default: 4096 entries), stop adding.

On decode, the dictionary is rebuilt identically. The special case where the encoder creates an entry that the decoder hasn't seen yet (code == next_code) is handled by concatenating the previous entry with its first character.

### LZ77

A sliding window (default 4 KB) scans backward from the current position to find the longest match. If the match is at least 3 bytes, emit a `(distance, length)` back-reference; otherwise emit a literal. Overlapping references are supported — the decoder copies byte-by-byte so that it can reference bytes it just wrote.

### Burrows-Wheeler Transform

1. Conceptually write out all cyclic rotations of the input and sort them lexicographically.
2. The BWT output is the *last column* of the sorted matrix; the *original index* (which row is the original string) is stored alongside.
3. The BWT doesn't compress anything by itself — but it clusters identical characters together, making the output extremely compressible by subsequent stages (MTF, then Huffman/RLE).

**Inverse BWT** uses the LF-mapping: count occurrences of each byte in the last column, compute first-occurrence positions in the (sorted) first column, then chain backwards to reconstruct the original.

### Move-to-Front Transform

Maintain a list of all 256 byte values. For each input byte, output its *current index* in the list, then move it to position 0. After BWT, many consecutive bytes are identical — so they all map to index 0, producing a stream dominated by small numbers (great for entropy coders).

### Arithmetic Coding

Instead of assigning discrete code words, arithmetic coding represents the entire message as a single number in the interval [0, 1):

1. Partition [0, 1) into sub-intervals proportional to symbol probabilities.
2. For each symbol, narrow the interval to its sub-interval.
3. Output enough bits to uniquely identify the final interval.

This crate uses 32-bit fixed-point integer arithmetic with the standard E³ (Elias) renormalisation: when `high < half` or `low ≥ half`, emit a bit; when the interval straddles the quarter point, defer with a pending count. The result approaches the theoretical minimum bit length far more closely than Huffman (which is limited to whole-bit code lengths).

### Delta Encoding with ZigZag + VarInt

1. Replace each byte with the difference from the previous byte (`i16` deltas).
2. Map signed deltas to unsigned with ZigZag encoding: `0→0, -1→1, 1→2, -2→3, ...`
3. Encode each unsigned value with variable-length byte encoding (7 bits per byte, MSB continuation bit).

This is the same technique used by Protocol Buffers and many time-series databases. For slowly-changing telemetry (CPU usage ticking up by 1%), the deltas are tiny and compress to 1 byte each.

---

## The Math

### Shannon Entropy

$$H(X) = -\sum_{x \in \mathcal{X}} p(x) \log_2 p(x)$$

Where $p(x)$ is the probability of symbol $x$. `Entropy::shannon_entropy()` estimates this from empirical frequencies. Key properties:

- **$H = 0$** when all symbols are identical (no information).
- **$H = 8$ bits** for uniformly distributed bytes (maximum information per symbol).
- The **source coding theorem** (Shannon, 1948) says no lossless compressor can use fewer than $H$ bits per symbol on average.

### Kraft Inequality

For a prefix-free code with lengths $\ell_1, \ell_2, \ldots, \ell_n$:

$$\sum_{i=1}^{n} 2^{-\ell_i} \leq 1$$

Huffman's algorithm produces code lengths that satisfy this with equality (or near-equality for dyadic distributions).

### Cross-Entropy & KL Divergence

$$H_{\text{cross}}(P \| Q) = -\sum_x p(x) \log_2 q(x)$$

$$D_{\text{KL}}(P \| Q) = H_{\text{cross}}(P \| Q) - H(P)$$

These measure how many extra bits you waste when you compress data drawn from distribution $P$ using a code optimised for distribution $Q$. The `Entropy` module provides both.

### Arithmetic Coding Optimality

Arithmetic coding's output length is at most 2 bits more than $-\log_2 P(\text{message})$, making it essentially optimal for any known symbol distribution — unlike Huffman, which can waste up to 1 bit per symbol for skewed distributions (e.g., a symbol with probability 0.99 needs only 0.0145 bits but Huffman must assign at least 1 bit).

### BWT Invertibility

The BWT is a **bijection** on strings (up to the stored original index). Inversion relies on the fact that the last column and first column of the rotation matrix are both permutations of the input, and the LF-mapping (`lf[i] = first_occurrence[L[i]] + occ_count(L[i], i)`) gives a cycle that reconstructs the original string in reverse.

---

## Testing

**62 unit tests** across all 10 modules, covering:

- **Round-trip correctness:** encode → decode → compare for every algorithm
- **Edge cases:** empty input, single byte, all-same, maximum entropy
- **Special cases:** LZW decoder code-not-yet-in-dictionary, LZ77 overlapping references, Huffman single-symbol input
- **Prefix-free property:** verifies no Huffman code is a prefix of another
- **Compression quality:** arithmetic coding output is within 64 bits of theoretical minimum
- **Agent compressor:** auto-strategy selection, telemetry serialisation, worthwhileness checks

Run everything:

```bash
cargo test
```

---

## License

MIT
