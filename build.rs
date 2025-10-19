fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_prost_build::compile_protos("jelly-fpga-server/protos/jelly_fpga_control.proto")?;
    Ok(())
}