use jelly_fpga_client::JellyFpgaClient;
use std::env;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // コマンドライン引数でIPアドレスを取得
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <target_address>", args[0]);
        eprintln!("Example: {} 127.0.0.1:8051", args[0]);
        std::process::exit(1);
    }
    
    let target = &args[1];
    println!("target: {}", target);
    
    // サーバーに接続
    let target_url = if target.starts_with("http://") {
        target.to_string()
    } else {
        format!("http://{}", target)
    };
    
    let mut fpga_ctl = JellyFpgaClient::connect(target_url).await?;
    
    // FPGAをリセット
    let reset_result = fpga_ctl.reset().await?;
    println!("Reset result: {}", reset_result);

    // Device Tree Source を定義
    let dts = r#"/dts-v1/; /plugin/;

/ {
    fragment@0 {
        target = <&fpga_full>;
        overlay0: __overlay__ {
            #address-cells = <2>;
            #size-cells = <2>;
            firmware-name = "kv260_blinking_led_ps.bit.bin";
        };
    };

    fragment@1 {
        target = <&amba>;
        overlay1: __overlay__ {
            clocking0: clocking0 {
                #clock-cells = <0>;
                assigned-clock-rates = <100000000>;
                assigned-clocks = <&zynqmp_clk 71>;
                clock-output-names = "fabric_clk";
                clocks = <&zynqmp_clk 71>;
                compatible = "xlnx,fclk";
            };
        };
    };
};
"#;

    // DTSをDTBに変換してfirmwareとしてアップロード
    let (dtb_result, dtb_data) = fpga_ctl.dts_to_dtb(dts).await?;
    if !dtb_result {
        eprintln!("Failed to convert DTS to DTB");
        return Err("DTS to DTB conversion failed".into());
    }
    
    let dtb_upload_result = fpga_ctl.upload_firmware("kv260_blinking_led_ps.dtbo", dtb_data).await?;
    println!("DTB upload result: {}", dtb_upload_result);

    // bitstreamファイルをアップロード
    println!("Bitstream upload start");
    let bit_upload_result = fpga_ctl.upload_firmware_file(
        "kv260_blinking_led_ps.bit",
        "../bitstream/kv260_blinking_led_ps.bit"
    ).await?;
    println!("Bitstream upload result: {}", bit_upload_result);
    
    // アップロードしたbitstreamファイルをbinファイルに変換
    let bin_convert_result = fpga_ctl.bitstream_to_bin(
        "kv260_blinking_led_ps.bit",
        "kv260_blinking_led_ps.bit.bin",
        "zynqmp",
    ).await?;
    println!("Bitstream to bin conversion result: {}", bin_convert_result);

    // 現在のfirmwareをアンロード
    let unload_result = fpga_ctl.unload_all().await?;
    println!("Unload result: {}", unload_result);
    
    // DTBOをロード
    let load_dtbo_result = fpga_ctl.load_dtbo("kv260_blinking_led_ps.dtbo").await?;
    println!("Load DTBO result: {}", load_dtbo_result);

    // /dev/memをmmapしてLED0を点滅させる
    let (mmap_result, accessor_id) = fpga_ctl.open_mmap("/dev/mem", 0xa0000000, 0x1000, 8).await?;
    if !mmap_result {
        eprintln!("Failed to open memory map");
        return Err("Memory map failed".into());
    }
    println!("Memory map opened with ID: {}", accessor_id);

    // LED0を3回点滅させる
    for i in 1..=3 {
        println!("Blink {}/3", i);
        
        // LED0 ON
        let led_on_result = fpga_ctl.write_mem_u64(accessor_id, 0, 1).await?;
        println!("LED ON result: {}", led_on_result);
        tokio::time::sleep(Duration::from_millis(500)).await;
        
        // LED0 OFF
        let led_off_result = fpga_ctl.write_mem_u64(accessor_id, 0, 0).await?;
        println!("LED OFF result: {}", led_off_result);
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    // メモリマップを閉じる
    let close_result = fpga_ctl.close(accessor_id).await?;
    println!("Close result: {}", close_result);

    // 後始末：アップロードしたファイルを削除
    let remove_dtbo_result = fpga_ctl.remove_firmware("kv260_blinking_led_ps.dtbo").await?;
    println!("Remove DTBO result: {}", remove_dtbo_result);
    
    let remove_bit_result = fpga_ctl.remove_firmware("kv260_blinking_led_ps.bit").await?;
    println!("Remove bitstream result: {}", remove_bit_result);
    
    let remove_bin_result = fpga_ctl.remove_firmware("kv260_blinking_led_ps.bit.bin").await?;
    println!("Remove bin result: {}", remove_bin_result);

    // 元の設定に戻す
    let final_unload_result = fpga_ctl.unload_all().await?;
    println!("Final unload result: {}", final_unload_result);
    
    let (load_starter_result, _slot) = fpga_ctl.load("k26-starter-kits").await?;
    println!("Load k26-starter-kits result: {}", load_starter_result);

    println!("Blinking LED test completed successfully!");
    Ok(())
}