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

    println!("\n=== Testing Accelerator Management ===");

    // Test register_accel (this example assumes you have the files)
    let accel_name = "test_accel";
    let bin_file = "/lib/firmware/test.bin"; // example path
    let dtbo_file = "/lib/firmware/test.dtbo"; // example path

    match client.register_accel(accel_name, bin_file, dtbo_file, None, true).await {
        Ok(result) => println!("✓ Register accelerator: {}", result),
        Err(e) => println!("✗ Register accelerator failed: {}", e),
    }

    // Test load accelerator
    match client.load(accel_name).await {
        Ok((result, slot)) => {
            println!("✓ Load accelerator: result={}, slot={}", result, slot);
            
            if result {
                // Test unload
                match client.unload(slot).await {
                    Ok(unload_result) => println!("✓ Unload accelerator: {}", unload_result),
                    Err(e) => println!("✗ Unload accelerator failed: {}", e),
                }
            }
        }
        Err(e) => println!("✗ Load accelerator failed: {}", e),
    }

    // Test unregister_accel
    match client.unregister_accel(accel_name).await {
        Ok(result) => println!("✓ Unregister accelerator: {}", result),
        Err(e) => println!("✗ Unregister accelerator failed: {}", e),
    }

    println!("\n=== Accelerator Management Test Complete ===");
    Ok(())
}