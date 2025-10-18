# Jelly FPGA Client (Rust)

A Rust gRPC client library for interacting with the Jelly FPGA Server.

## Features

This library provides a complete Rust interface to the Jelly FPGA Server, supporting:

### System Management
- `reset()` - Reset the FPGA
- `load(name)` - Load firmware by name
- `unload(slot)` - Unload firmware from slot
- `unload_all()` - Unload all firmware (convenience method)
- `upload_firmware(name, data)` - Upload firmware from byte data
- `upload_firmware_file(name, file_path)` - Upload firmware from file
- `remove_firmware(name)` - Remove firmware
- `load_bitstream(name)` - Load bitstream
- `load_dtbo(name)` - Load device tree overlay

### Device Management
- `open_mmap(path, offset, size, unit)` - Open memory mapped device
- `open_mmap_simple(path, offset, size)` - Open memory mapped device with default unit
- `open_uio(name, unit)` - Open UIO device
- `open_udmabuf(name, cache_enable, unit)` - Open UDMABUF device
- `close(id)` - Close device
- `subclone(id, offset, size, unit)` - Create device subclone
- `get_addr(id)` - Get device address
- `get_size(id)` - Get device size
- `get_phys_addr(id)` - Get physical address

### Memory and Register Access
- Integer operations (signed/unsigned):
  - `write_mem_u/i(id, offset, data, size)` - Write to memory
  - `write_mem_u64(id, offset, data)` - Write 64-bit unsigned to memory (convenience)
  - `read_mem_u/i(id, offset, size)` - Read from memory
  - `write_reg_u/i(id, reg, data, size)` - Write to register
  - `read_reg_u/i(id, reg, size)` - Read from register

- Floating point operations:
  - `write_mem_f32/f64(id, offset, data)` - Write float to memory
  - `read_mem_f32/f64(id, offset)` - Read float from memory
  - `write_reg_f32/f64(id, reg, data)` - Write float to register
  - `read_reg_f32/f64(id, reg)` - Read float from register

- Bulk operations:
  - `mem_copy_to(id, offset, data)` - Copy data to memory
  - `mem_copy_from(id, offset, size)` - Copy data from memory

### Utilities
- `dts_to_dtb(dts)` - Convert DTS to DTB
- `bitstream_to_bin(bitstream_name, bin_name, arch)` - Convert bitstream to binary

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
jelly-fpga-client = "0.1.0"
tokio = { version = "1.0", features = ["rt-multi-thread", "macros"] }
```

### Basic Example

```rust
use jelly_fpga_client::JellyFpgaClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to the server
    let mut client = JellyFpgaClient::connect("http://192.168.1.100:8051").await?;

    // Reset the FPGA
    let reset_result = client.reset().await?;
    println!("Reset result: {}", reset_result);

    // Load firmware
    let (load_result, slot) = client.load("my_firmware".to_string()).await?;
    if load_result {
        println!("Firmware loaded in slot: {}", slot);
        
        // Open UIO device
        let (open_result, device_id) = client.open_uio("my_device".to_string(), 4).await?;
        if open_result {
            // Write to register
            client.write_reg_u(device_id, 0x00, 0x12345678, 4).await?;
            
            // Read from register
            let (_, data) = client.read_reg_u(device_id, 0x00, 4).await?;
            println!("Register value: 0x{:08x}", data);
            
            // Close device
            client.close(device_id).await?;
        }
        
        // Unload firmware
        client.unload(slot).await?;
    }

    Ok(())
}
```

### Memory Operations Example

```rust
use jelly_fpga_client::JellyFpgaClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = JellyFpgaClient::connect("http://192.168.1.100:8051").await?;
    
    // Open UDMABUF for DMA operations
    let (open_result, buf_id) = client.open_udmabuf("udmabuf0".to_string(), true, 1).await?;
    if open_result {
        // Write data to buffer
        let data = vec![0x01, 0x02, 0x03, 0x04];
        client.mem_copy_to(buf_id, 0, data).await?;
        
        // Read data back
        let (_, read_data) = client.mem_copy_from(buf_id, 0, 4).await?;
        println!("Read data: {:?}", read_data);
        
        client.close(buf_id).await?;
    }

    Ok(())
}
```

## Requirements

- Rust 1.70.0 or later
- A running Jelly FPGA Server

## Building

```bash
cargo build
```

## Running Examples

### Basic Usage Example
```bash
cargo run --example basic_usage
```

### Comprehensive Test Example
```bash
cargo run --example comprehensive_test
```

### Blinking LED Example
This example demonstrates a complete workflow similar to the Python `test_blinking_led.py`:
- Uploads bitstream and device tree files
- Configures FPGA with LED blinking firmware
- Controls LED through memory-mapped I/O
- Cleans up resources

```bash
# Copy the required bitstream file to the examples directory first
cp /path/to/kv260_blinking_led_ps.bit examples/

# Run the example with target server address
cargo run --example test_blinking_led -- 10.72.141.82:8051
```

The blinking LED example includes:
- Firmware upload via gRPC streaming
- DTS to DTB conversion
- Bitstream to binary conversion
- Device tree overlay loading
- Memory-mapped I/O for LED control
- Proper resource cleanup

## License

This project is licensed under the same license as the Jelly FPGA Server project.