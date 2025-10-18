use jelly_fpga_client::JellyFpgaClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to the gRPC server
    let mut client = JellyFpgaClient::connect("http://[::1]:8051").await?;

    println!("Connected to Jelly FPGA Server");

    // Reset the FPGA
    let reset_result = client.reset().await?;
    println!("Reset result: {}", reset_result);

    // Load a firmware
    let (load_result, slot) = client.load("sample_firmware".to_string()).await?;
    println!("Load result: {}, slot: {}", load_result, slot);

    // Open UIO device
    let (open_result, id) = client.open_uio("sample_device".to_string(), 4).await?;
    println!("Open UIO result: {}, id: {}", open_result, id);

    if open_result {
        // Write to register
        let write_result = client.write_reg_u(id, 0x00, 0x12345678, 4).await?;
        println!("Write register result: {}", write_result);

        // Read from register
        let (read_result, data) = client.read_reg_u(id, 0x00, 4).await?;
        println!("Read register result: {}, data: 0x{:08x}", read_result, data);

        // Close device
        let close_result = client.close(id).await?;
        println!("Close result: {}", close_result);
    }

    // Unload firmware
    if load_result {
        let unload_result = client.unload(slot).await?;
        println!("Unload result: {}", unload_result);
    }

    Ok(())
}