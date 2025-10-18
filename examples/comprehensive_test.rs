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

    // Try to load a firmware (this may fail if firmware doesn't exist)
    println!("\n=== Testing Firmware Operations ===");
    match client.load("kv260_blinking_led_ps").await {
        Ok((result, slot)) => {
            println!("✓ Load firmware: result={}, slot={}", result, slot);
            if result {
                // Unload the firmware
                match client.unload(slot).await {
                    Ok(unload_result) => println!("✓ Unload firmware: {}", unload_result),
                    Err(e) => println!("✗ Unload failed: {}", e),
                }
            }
        }
        Err(e) => println!("✗ Load firmware failed: {}", e),
    }

    // Test device operations
    println!("\n=== Testing Device Operations ===");
    
    // Try to open a UIO device
    match client.open_uio("uio0", 4).await {
        Ok((result, id)) => {
            println!("✓ Open UIO: result={}, id={}", result, id);
            if result {
                // Test register operations
                println!("\n=== Testing Register Operations ===");
                
                // Write to register 0
                match client.write_reg_u(id, 0x00, 0x12345678, 4).await {
                    Ok(write_result) => {
                        println!("✓ Write register: {}", write_result);
                        
                        // Read back from register 0
                        match client.read_reg_u(id, 0x00, 4).await {
                            Ok((read_result, data)) => {
                                println!("✓ Read register: result={}, data=0x{:08x}", read_result, data);
                            }
                            Err(e) => println!("✗ Read register failed: {}", e),
                        }
                    }
                    Err(e) => println!("✗ Write register failed: {}", e),
                }

                // Test floating point operations
                println!("\n=== Testing Float Operations ===");
                match client.write_reg_f32(id, 0x04, 3.14159).await {
                    Ok(write_result) => {
                        println!("✓ Write float register: {}", write_result);
                        
                        match client.read_reg_f32(id, 0x04).await {
                            Ok((read_result, data)) => {
                                println!("✓ Read float register: result={}, data={}", read_result, data);
                            }
                            Err(e) => println!("✗ Read float register failed: {}", e),
                        }
                    }
                    Err(e) => println!("✗ Write float register failed: {}", e),
                }

                // Get device information
                println!("\n=== Testing Device Info ===");
                match client.get_addr(id).await {
                    Ok((result, addr)) => println!("✓ Device address: result={}, addr=0x{:x}", result, addr),
                    Err(e) => println!("✗ Get address failed: {}", e),
                }

                match client.get_size(id).await {
                    Ok((result, size)) => println!("✓ Device size: result={}, size={}", result, size),
                    Err(e) => println!("✗ Get size failed: {}", e),
                }

                // Close the device
                match client.close(id).await {
                    Ok(close_result) => println!("✓ Close device: {}", close_result),
                    Err(e) => println!("✗ Close device failed: {}", e),
                }
            }
        }
        Err(e) => println!("✗ Open UIO failed: {}", e),
    }

    // Test UDMABUF operations
    println!("\n=== Testing UDMABUF Operations ===");
    match client.open_udmabuf("udmabuf0", true, 1).await {
        Ok((result, id)) => {
            println!("✓ Open UDMABUF: result={}, id={}", result, id);
            if result {
                // Test memory copy operations
                let test_data = vec![0xde, 0xad, 0xbe, 0xef, 0x01, 0x02, 0x03, 0x04];
                
                match client.mem_copy_to(id, 0, test_data.clone()).await {
                    Ok(copy_result) => {
                        println!("✓ Memory copy to: {}", copy_result);
                        
                        match client.mem_copy_from(id, 0, test_data.len() as u64).await {
                            Ok((read_result, data)) => {
                                println!("✓ Memory copy from: result={}, data={:?}", read_result, data);
                                if data == test_data {
                                    println!("✓ Data verification passed!");
                                } else {
                                    println!("✗ Data verification failed!");
                                }
                            }
                            Err(e) => println!("✗ Memory copy from failed: {}", e),
                        }
                    }
                    Err(e) => println!("✗ Memory copy to failed: {}", e),
                }

                // Close the UDMABUF
                match client.close(id).await {
                    Ok(close_result) => println!("✓ Close UDMABUF: {}", close_result),
                    Err(e) => println!("✗ Close UDMABUF failed: {}", e),
                }
            }
        }
        Err(e) => println!("✗ Open UDMABUF failed: {}", e),
    }

    println!("\n=== Test completed ===");
    Ok(())
}