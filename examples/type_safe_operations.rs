use jelly_fpga_client::JellyFpgaClient;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get server address from command line or use default
    let server_addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "http://[::1]:8051".to_string());

    println!("Connecting to Jelly FPGA Server at: {}", server_addr);

    // Connect to the gRPC server
    let mut client = JellyFpgaClient::connect(server_addr).await?;
    println!("✓ Connected to Jelly FPGA Server");

    // Test basic operations
    println!("\n=== Testing Basic Operations ===");

    // Reset the FPGA
    match client.reset().await {
        Ok(result) => println!("✓ Reset: {}", result),
        Err(e) => println!("✗ Reset failed: {}", e),
    }

    // Test memory operations with type-safe methods
    println!("\n=== Testing Type-Safe Memory Operations ===");
    
    // Try to open a memory map (this may fail if /dev/mem is not accessible)
    match client.open_mmap("/dev/mem", 0x40000000, 0x1000, 8).await {
        Ok((result, id)) => {
            println!("✓ Open memory map: result={}, id={}", result, id);
            if result {
                // Test various sized write operations
                println!("Testing type-safe memory write operations...");
                
                let _ = client.write_mem_u8(id, 0x00, 0x12u8).await;
                println!("  write_mem_u8(0x00, 0x12): completed");
                
                let _ = client.write_mem_u16(id, 0x04, 0x1234u16).await;
                println!("  write_mem_u16(0x04, 0x1234): completed");
                
                let _ = client.write_mem_u32(id, 0x08, 0x12345678u32).await;
                println!("  write_mem_u32(0x08, 0x12345678): completed");
                
                let _ = client.write_mem_u64(id, 0x10, 0x123456789ABCDEFu64).await;
                println!("  write_mem_u64(0x10, 0x123456789ABCDEF): completed");

                // Test signed operations
                let _ = client.write_mem_i8(id, 0x18, -1i8).await;
                println!("  write_mem_i8(0x18, -1): completed");
                
                let _ = client.write_mem_i16(id, 0x1C, -1000i16).await;
                println!("  write_mem_i16(0x1C, -1000): completed");
                
                let _ = client.write_mem_i32(id, 0x20, -100000i32).await;
                println!("  write_mem_i32(0x20, -100000): completed");
                
                let _ = client.write_mem_i64(id, 0x28, -1000000000i64).await;
                println!("  write_mem_i64(0x28, -1000000000): completed");

                // Test read operations
                println!("Testing type-safe memory read operations...");
                
                match client.read_mem_u8(id, 0x00).await {
                    Ok((result, data)) => println!("  read_mem_u8(0x00): result={}, data=0x{:02x}", result, data),
                    Err(e) => println!("  read_mem_u8(0x00): error={}", e),
                }
                
                match client.read_mem_u16(id, 0x04).await {
                    Ok((result, data)) => println!("  read_mem_u16(0x04): result={}, data=0x{:04x}", result, data),
                    Err(e) => println!("  read_mem_u16(0x04): error={}", e),
                }
                
                match client.read_mem_u32(id, 0x08).await {
                    Ok((result, data)) => println!("  read_mem_u32(0x08): result={}, data=0x{:08x}", result, data),
                    Err(e) => println!("  read_mem_u32(0x08): error={}", e),
                }
                
                match client.read_mem_u64(id, 0x10).await {
                    Ok((result, data)) => println!("  read_mem_u64(0x10): result={}, data=0x{:016x}", result, data),
                    Err(e) => println!("  read_mem_u64(0x10): error={}", e),
                }

                // Close the device
                match client.close(id).await {
                    Ok(close_result) => println!("✓ Close device: {}", close_result),
                    Err(e) => println!("✗ Close failed: {}", e),
                }
            }
        }
        Err(e) => println!("✗ Open memory map failed: {}", e),
    }

    // Test register operations with UIO (if available)
    println!("\n=== Testing Type-Safe Register Operations ===");
    match client.open_uio("uio0", 4).await {
        Ok((result, id)) => {
            println!("✓ Open UIO: result={}, id={}", result, id);
            if result {
                println!("Testing type-safe register operations...");
                
                // Test write operations
                let _ = client.write_reg_u8(id, 0x00, 0xAAu8).await;
                println!("  write_reg_u8(0x00, 0xAA): completed");
                
                let _ = client.write_reg_u16(id, 0x04, 0xBEEFu16).await;
                println!("  write_reg_u16(0x04, 0xBEEF): completed");
                
                let _ = client.write_reg_u32(id, 0x08, 0xDEADBEEFu32).await;
                println!("  write_reg_u32(0x08, 0xDEADBEEF): completed");

                // Test read operations
                match client.read_reg_u8(id, 0x00).await {
                    Ok((result, data)) => println!("  read_reg_u8(0x00): result={}, data=0x{:02x}", result, data),
                    Err(e) => println!("  read_reg_u8(0x00): error={}", e),
                }
                
                match client.read_reg_u16(id, 0x04).await {
                    Ok((result, data)) => println!("  read_reg_u16(0x04): result={}, data=0x{:04x}", result, data),
                    Err(e) => println!("  read_reg_u16(0x04): error={}", e),
                }
                
                match client.read_reg_u32(id, 0x08).await {
                    Ok((result, data)) => println!("  read_reg_u32(0x08): result={}, data=0x{:08x}", result, data),
                    Err(e) => println!("  read_reg_u32(0x08): error={}", e),
                }

                // Close the device
                match client.close(id).await {
                    Ok(close_result) => println!("✓ Close UIO device: {}", close_result),
                    Err(e) => println!("✗ Close UIO failed: {}", e),
                }
            }
        }
        Err(e) => println!("✗ Open UIO failed: {}", e),
    }

    println!("\n✓ Type-safe memory/register operations test completed!");
    Ok(())
}