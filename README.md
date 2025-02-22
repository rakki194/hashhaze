# hashhaze

A high-performance Rust implementation of the [BlurHash algorithm](https://github.com/woltapp/blurhash). HashHaze is a CLI tool that generates compact, visually pleasing image placeholders using the BlurHash encoding scheme.

## Features

- Fast, parallel processing of multiple images using Tokio
- Support for various image formats
- Configurable X and Y components for hash generation
- Efficient CPU utilization with automatic thread management
- Simple CLI interface

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/yourusername/hashhaze.git
cd hashhaze

# Build the project
cargo build --release

# The binary will be available at target/release/hashhaze
```

### Usage

```bash
# Basic usage with a single image
hashhaze input.jpg

# Process multiple images
hashhaze image1.jpg image2.png directory/

# Customize components (default: -x 4 -y 3)
hashhaze -x 5 -y 4 input.jpg

# Process an entire directory of images
hashhaze path/to/images/
```

### Options

- `-x, --components-x <NUMBER>`: Number of X components for BlurHash (default: 4)
- `-y, --components-y <NUMBER>`: Number of Y components for BlurHash (default: 3)

## How It Works

HashHaze implements the BlurHash algorithm, which creates a compact string representation of an image placeholder. The algorithm works by:

1. Breaking down the image into a grid (specified by X and Y components)
2. Calculating DCT (Discrete Cosine Transform) coefficients
3. Encoding the results into a base83 string

The resulting hash can be used to generate a blurred placeholder while the original image loads.

## Performance

The tool is optimized for performance:

- Parallel processing using Tokio async runtime
- Automatic CPU core detection for optimal thread usage
- Memory-efficient image processing

## Dependencies

- clap: Command-line argument parsing
- tokio: Async runtime for parallel processing
- imx: Image processing
- xio: File I/O operations
- futures: Async utilities
- num_cpus: CPU core detection
- anyhow: Error handling
- thiserror: Custom error types

## License

This project is licensed under the MIT License - see the [LICENSE.md](LICENSE.md) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Acknowledgments

- Original [BlurHash algorithm](https://github.com/woltapp/blurhash) by Wolt
- All contributors to the project
