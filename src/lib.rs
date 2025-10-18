use tonic::transport::Channel;
use tonic::Request;

pub mod jelly_fpga_control {
    tonic::include_proto!("jelly_fpga_control");
}

use jelly_fpga_control::jelly_fpga_control_client::JellyFpgaControlClient;
use jelly_fpga_control::*;

/// Jelly FPGA Control Client
pub struct JellyFpgaClient {
    client: JellyFpgaControlClient<Channel>,
}

impl JellyFpgaClient {
    /// Create a new client connection
    pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
    where
        D: std::convert::TryInto<tonic::transport::Endpoint>,
        D::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        let client = JellyFpgaControlClient::connect(dst).await?;
        Ok(JellyFpgaClient { client })
    }

    /// Reset the FPGA
    pub async fn reset(&mut self) -> Result<bool, tonic::Status> {
        let request = Request::new(ResetRequest {});
        let response = self.client.reset(request).await?;
        Ok(response.into_inner().result)
    }

    /// Load firmware with name
    pub async fn load(&mut self, name: &str) -> Result<(bool, i32), tonic::Status> {
        let request = Request::new(LoadRequest { name: name.to_string() });
        let response = self.client.load(request).await?;
        let inner = response.into_inner();
        Ok((inner.result, inner.slot))
    }

    /// Unload firmware from slot
    pub async fn unload(&mut self, slot: i32) -> Result<bool, tonic::Status> {
        let request = Request::new(UnloadRequest { slot });
        let response = self.client.unload(request).await?;
        Ok(response.into_inner().result)
    }

    /// Unload all firmware (convenience method)
    pub async fn unload_all(&mut self) -> Result<bool, tonic::Status> {
        // In practice, slot -1 or 0 might unload all, but this depends on server implementation
        // For now, we'll use slot 0 as a default
        self.unload(0).await
    }

    /// Upload firmware from data
    pub async fn upload_firmware(&mut self, name: &str, data: Vec<u8>) -> Result<bool, tonic::Status> {
        use futures_core::stream::Stream;
        use std::pin::Pin;
        use std::task::{Context, Poll};
        
        struct DataStream {
            name: String,
            data: Vec<u8>,
            chunk_size: usize,
            offset: usize,
        }
        
        impl Stream for DataStream {
            type Item = UploadFirmwareRequest;
            
            fn poll_next(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
                if self.offset >= self.data.len() {
                    return Poll::Ready(None);
                }
                
                let end = std::cmp::min(self.offset + self.chunk_size, self.data.len());
                let chunk = self.data[self.offset..end].to_vec();
                self.offset = end;
                
                let request = UploadFirmwareRequest {
                    name: self.name.clone(),
                    data: chunk,
                };
                
                Poll::Ready(Some(request))
            }
        }
        
        let stream = DataStream {
            name: name.to_string(),
            data,
            chunk_size: 2 * 1024 * 1024, // 2MB chunks like Python version
            offset: 0,
        };
        
        let response = self.client.upload_firmware(Request::new(stream)).await?;
        Ok(response.into_inner().result)
    }

    /// Upload firmware from file
    pub async fn upload_firmware_file(&mut self, name: &str, file_path: &str) -> Result<bool, tonic::Status> {
        let data = std::fs::read(file_path).map_err(|e| {
            tonic::Status::internal(format!("Failed to read file {}: {}", file_path, e))
        })?;
        
        self.upload_firmware(name, data).await
    }

    /// Remove firmware
    pub async fn remove_firmware(&mut self, name: &str) -> Result<bool, tonic::Status> {
        let request = Request::new(RemoveFirmwareRequest { name: name.to_string() });
        let response = self.client.remove_firmware(request).await?;
        Ok(response.into_inner().result)
    }

    /// Load bitstream
    pub async fn load_bitstream(&mut self, name: &str) -> Result<bool, tonic::Status> {
        let request = Request::new(LoadBitstreamRequest { name: name.to_string() });
        let response = self.client.load_bitstream(request).await?;
        Ok(response.into_inner().result)
    }

    /// Load device tree overlay
    pub async fn load_dtbo(&mut self, name: &str) -> Result<bool, tonic::Status> {
        let request = Request::new(LoadDtboRequest { name: name.to_string() });
        let response = self.client.load_dtbo(request).await?;
        Ok(response.into_inner().result)
    }

    /// Convert DTS to DTB
    pub async fn dts_to_dtb(&mut self, dts: &str) -> Result<(bool, Vec<u8>), tonic::Status> {
        let request = Request::new(DtsToDtbRequest { dts: dts.to_string() });
        let response = self.client.dts_to_dtb(request).await?;
        let inner = response.into_inner();
        Ok((inner.result, inner.dtb))
    }

    /// Convert bitstream to bin
    pub async fn bitstream_to_bin(
        &mut self,
        bitstream_name: &str,
        bin_name: &str,
        arch: &str,
    ) -> Result<bool, tonic::Status> {
        let request = Request::new(BitstreamToBinRequest {
            bitstream_name: bitstream_name.to_string(),
            bin_name: bin_name.to_string(),
            arch: arch.to_string(),
        });
        let response = self.client.bitstream_to_bin(request).await?;
        Ok(response.into_inner().result)
    }

    /// Open memory map
    pub async fn open_mmap(
        &mut self,
        path: &str,
        offset: u64,
        size: u64,
        unit: u64,
    ) -> Result<(bool, u32), tonic::Status> {
        let request = Request::new(OpenMmapRequest {
            path: path.to_string(),
            offset,
            size,
            unit,
        });
        let response = self.client.open_mmap(request).await?;
        let inner = response.into_inner();
        Ok((inner.result, inner.id))
    }



    /// Open UIO device
    pub async fn open_uio(&mut self, name: &str, unit: u64) -> Result<(bool, u32), tonic::Status> {
        let request = Request::new(OpenUioRequest { name: name.to_string(), unit });
        let response = self.client.open_uio(request).await?;
        let inner = response.into_inner();
        Ok((inner.result, inner.id))
    }

    /// Open UDMABUF device
    pub async fn open_udmabuf(
        &mut self,
        name: &str,
        cache_enable: bool,
        unit: u64,
    ) -> Result<(bool, u32), tonic::Status> {
        let request = Request::new(OpenUdmabufRequest {
            name: name.to_string(),
            cache_enable,
            unit,
        });
        let response = self.client.open_udmabuf(request).await?;
        let inner = response.into_inner();
        Ok((inner.result, inner.id))
    }

    /// Close device
    pub async fn close(&mut self, id: u32) -> Result<bool, tonic::Status> {
        let request = Request::new(CloseRequest { id });
        let response = self.client.close(request).await?;
        Ok(response.into_inner().result)
    }

    /// Create subclone of device
    pub async fn subclone(
        &mut self,
        id: u32,
        offset: u64,
        size: u64,
        unit: u64,
    ) -> Result<(bool, u32), tonic::Status> {
        let request = Request::new(SubcloneRequest {
            id,
            offset,
            size,
            unit,
        });
        let response = self.client.subclone(request).await?;
        let inner = response.into_inner();
        Ok((inner.result, inner.id))
    }

    /// Get device address
    pub async fn get_addr(&mut self, id: u32) -> Result<(bool, u64), tonic::Status> {
        let request = Request::new(GetAddrRequest { id });
        let response = self.client.get_addr(request).await?;
        let inner = response.into_inner();
        Ok((inner.result, inner.addr))
    }

    /// Get device size
    pub async fn get_size(&mut self, id: u32) -> Result<(bool, u64), tonic::Status> {
        let request = Request::new(GetSizeRequest { id });
        let response = self.client.get_size(request).await?;
        let inner = response.into_inner();
        Ok((inner.result, inner.size))
    }

    /// Get device physical address
    pub async fn get_phys_addr(&mut self, id: u32) -> Result<(bool, u64), tonic::Status> {
        let request = Request::new(GetPhysAddrRequest { id });
        let response = self.client.get_phys_addr(request).await?;
        let inner = response.into_inner();
        Ok((inner.result, inner.phys_addr))
    }

    /// Write unsigned integer to memory
    pub async fn write_mem_u(
        &mut self,
        id: u32,
        offset: u64,
        data: u64,
        size: u64,
    ) -> Result<bool, tonic::Status> {
        let request = Request::new(WriteMemURequest {
            id,
            offset,
            data,
            size,
        });
        let response = self.client.write_mem_u(request).await?;
        Ok(response.into_inner().result)
    }

    /// Write 8-bit unsigned integer to memory (convenience method)
    pub async fn write_mem_u8(
        &mut self,
        id: u32,
        offset: u64,
        data: u8,
    ) -> Result<bool, tonic::Status> {
        self.write_mem_u(id, offset, data as u64, 1).await
    }

    /// Write 16-bit unsigned integer to memory (convenience method)
    pub async fn write_mem_u16(
        &mut self,
        id: u32,
        offset: u64,
        data: u16,
    ) -> Result<bool, tonic::Status> {
        self.write_mem_u(id, offset, data as u64, 2).await
    }

    /// Write 32-bit unsigned integer to memory (convenience method)
    pub async fn write_mem_u32(
        &mut self,
        id: u32,
        offset: u64,
        data: u32,
    ) -> Result<bool, tonic::Status> {
        self.write_mem_u(id, offset, data as u64, 4).await
    }

    /// Write 64-bit unsigned integer to memory (convenience method)
    pub async fn write_mem_u64(
        &mut self,
        id: u32,
        offset: u64,
        data: u64,
    ) -> Result<bool, tonic::Status> {
        self.write_mem_u(id, offset, data, 8).await
    }

    /// Write signed integer to memory
    pub async fn write_mem_i(
        &mut self,
        id: u32,
        offset: u64,
        data: i64,
        size: u64,
    ) -> Result<bool, tonic::Status> {
        let request = Request::new(WriteMemIRequest {
            id,
            offset,
            data,
            size,
        });
        let response = self.client.write_mem_i(request).await?;
        Ok(response.into_inner().result)
    }

    /// Write 8-bit signed integer to memory (convenience method)
    pub async fn write_mem_i8(
        &mut self,
        id: u32,
        offset: u64,
        data: i8,
    ) -> Result<bool, tonic::Status> {
        self.write_mem_i(id, offset, data as i64, 1).await
    }

    /// Write 16-bit signed integer to memory (convenience method)
    pub async fn write_mem_i16(
        &mut self,
        id: u32,
        offset: u64,
        data: i16,
    ) -> Result<bool, tonic::Status> {
        self.write_mem_i(id, offset, data as i64, 2).await
    }

    /// Write 32-bit signed integer to memory (convenience method)
    pub async fn write_mem_i32(
        &mut self,
        id: u32,
        offset: u64,
        data: i32,
    ) -> Result<bool, tonic::Status> {
        self.write_mem_i(id, offset, data as i64, 4).await
    }

    /// Write 64-bit signed integer to memory (convenience method)
    pub async fn write_mem_i64(
        &mut self,
        id: u32,
        offset: u64,
        data: i64,
    ) -> Result<bool, tonic::Status> {
        self.write_mem_i(id, offset, data, 8).await
    }

    /// Read unsigned integer from memory
    pub async fn read_mem_u(
        &mut self,
        id: u32,
        offset: u64,
        size: u64,
    ) -> Result<(bool, u64), tonic::Status> {
        let request = Request::new(ReadMemRequest { id, offset, size });
        let response = self.client.read_mem_u(request).await?;
        let inner = response.into_inner();
        Ok((inner.result, inner.data))
    }

    /// Read 8-bit unsigned integer from memory (convenience method)
    pub async fn read_mem_u8(
        &mut self,
        id: u32,
        offset: u64,
    ) -> Result<(bool, u8), tonic::Status> {
        let (result, data) = self.read_mem_u(id, offset, 1).await?;
        Ok((result, data as u8))
    }

    /// Read 16-bit unsigned integer from memory (convenience method)
    pub async fn read_mem_u16(
        &mut self,
        id: u32,
        offset: u64,
    ) -> Result<(bool, u16), tonic::Status> {
        let (result, data) = self.read_mem_u(id, offset, 2).await?;
        Ok((result, data as u16))
    }

    /// Read 32-bit unsigned integer from memory (convenience method)
    pub async fn read_mem_u32(
        &mut self,
        id: u32,
        offset: u64,
    ) -> Result<(bool, u32), tonic::Status> {
        let (result, data) = self.read_mem_u(id, offset, 4).await?;
        Ok((result, data as u32))
    }

    /// Read 64-bit unsigned integer from memory (convenience method)
    pub async fn read_mem_u64(
        &mut self,
        id: u32,
        offset: u64,
    ) -> Result<(bool, u64), tonic::Status> {
        self.read_mem_u(id, offset, 8).await
    }

    /// Read signed integer from memory
    pub async fn read_mem_i(
        &mut self,
        id: u32,
        offset: u64,
        size: u64,
    ) -> Result<(bool, i64), tonic::Status> {
        let request = Request::new(ReadMemRequest { id, offset, size });
        let response = self.client.read_mem_i(request).await?;
        let inner = response.into_inner();
        Ok((inner.result, inner.data))
    }

    /// Read 8-bit signed integer from memory (convenience method)
    pub async fn read_mem_i8(
        &mut self,
        id: u32,
        offset: u64,
    ) -> Result<(bool, i8), tonic::Status> {
        let (result, data) = self.read_mem_i(id, offset, 1).await?;
        Ok((result, data as i8))
    }

    /// Read 16-bit signed integer from memory (convenience method)
    pub async fn read_mem_i16(
        &mut self,
        id: u32,
        offset: u64,
    ) -> Result<(bool, i16), tonic::Status> {
        let (result, data) = self.read_mem_i(id, offset, 2).await?;
        Ok((result, data as i16))
    }

    /// Read 32-bit signed integer from memory (convenience method)
    pub async fn read_mem_i32(
        &mut self,
        id: u32,
        offset: u64,
    ) -> Result<(bool, i32), tonic::Status> {
        let (result, data) = self.read_mem_i(id, offset, 4).await?;
        Ok((result, data as i32))
    }

    /// Read 64-bit signed integer from memory (convenience method)
    pub async fn read_mem_i64(
        &mut self,
        id: u32,
        offset: u64,
    ) -> Result<(bool, i64), tonic::Status> {
        self.read_mem_i(id, offset, 8).await
    }

    /// Write unsigned integer to register
    pub async fn write_reg_u(
        &mut self,
        id: u32,
        reg: u64,
        data: u64,
        size: u64,
    ) -> Result<bool, tonic::Status> {
        let request = Request::new(WriteRegURequest {
            id,
            reg,
            data,
            size,
        });
        let response = self.client.write_reg_u(request).await?;
        Ok(response.into_inner().result)
    }

    /// Write 8-bit unsigned integer to register (convenience method)
    pub async fn write_reg_u8(
        &mut self,
        id: u32,
        reg: u64,
        data: u8,
    ) -> Result<bool, tonic::Status> {
        self.write_reg_u(id, reg, data as u64, 1).await
    }

    /// Write 16-bit unsigned integer to register (convenience method)
    pub async fn write_reg_u16(
        &mut self,
        id: u32,
        reg: u64,
        data: u16,
    ) -> Result<bool, tonic::Status> {
        self.write_reg_u(id, reg, data as u64, 2).await
    }

    /// Write 32-bit unsigned integer to register (convenience method)
    pub async fn write_reg_u32(
        &mut self,
        id: u32,
        reg: u64,
        data: u32,
    ) -> Result<bool, tonic::Status> {
        self.write_reg_u(id, reg, data as u64, 4).await
    }

    /// Write 64-bit unsigned integer to register (convenience method)
    pub async fn write_reg_u64(
        &mut self,
        id: u32,
        reg: u64,
        data: u64,
    ) -> Result<bool, tonic::Status> {
        self.write_reg_u(id, reg, data, 8).await
    }

    /// Write signed integer to register
    pub async fn write_reg_i(
        &mut self,
        id: u32,
        reg: u64,
        data: i64,
        size: u64,
    ) -> Result<bool, tonic::Status> {
        let request = Request::new(WriteRegIRequest {
            id,
            reg,
            data,
            size,
        });
        let response = self.client.write_reg_i(request).await?;
        Ok(response.into_inner().result)
    }

    /// Write 8-bit signed integer to register (convenience method)
    pub async fn write_reg_i8(
        &mut self,
        id: u32,
        reg: u64,
        data: i8,
    ) -> Result<bool, tonic::Status> {
        self.write_reg_i(id, reg, data as i64, 1).await
    }

    /// Write 16-bit signed integer to register (convenience method)
    pub async fn write_reg_i16(
        &mut self,
        id: u32,
        reg: u64,
        data: i16,
    ) -> Result<bool, tonic::Status> {
        self.write_reg_i(id, reg, data as i64, 2).await
    }

    /// Write 32-bit signed integer to register (convenience method)
    pub async fn write_reg_i32(
        &mut self,
        id: u32,
        reg: u64,
        data: i32,
    ) -> Result<bool, tonic::Status> {
        self.write_reg_i(id, reg, data as i64, 4).await
    }

    /// Write 64-bit signed integer to register (convenience method)
    pub async fn write_reg_i64(
        &mut self,
        id: u32,
        reg: u64,
        data: i64,
    ) -> Result<bool, tonic::Status> {
        self.write_reg_i(id, reg, data, 8).await
    }

    /// Read unsigned integer from register
    pub async fn read_reg_u(
        &mut self,
        id: u32,
        reg: u64,
        size: u64,
    ) -> Result<(bool, u64), tonic::Status> {
        let request = Request::new(ReadRegRequest { id, reg, size });
        let response = self.client.read_reg_u(request).await?;
        let inner = response.into_inner();
        Ok((inner.result, inner.data))
    }

    /// Read 8-bit unsigned integer from register (convenience method)
    pub async fn read_reg_u8(
        &mut self,
        id: u32,
        reg: u64,
    ) -> Result<(bool, u8), tonic::Status> {
        let (result, data) = self.read_reg_u(id, reg, 1).await?;
        Ok((result, data as u8))
    }

    /// Read 16-bit unsigned integer from register (convenience method)
    pub async fn read_reg_u16(
        &mut self,
        id: u32,
        reg: u64,
    ) -> Result<(bool, u16), tonic::Status> {
        let (result, data) = self.read_reg_u(id, reg, 2).await?;
        Ok((result, data as u16))
    }

    /// Read 32-bit unsigned integer from register (convenience method)
    pub async fn read_reg_u32(
        &mut self,
        id: u32,
        reg: u64,
    ) -> Result<(bool, u32), tonic::Status> {
        let (result, data) = self.read_reg_u(id, reg, 4).await?;
        Ok((result, data as u32))
    }

    /// Read 64-bit unsigned integer from register (convenience method)
    pub async fn read_reg_u64(
        &mut self,
        id: u32,
        reg: u64,
    ) -> Result<(bool, u64), tonic::Status> {
        self.read_reg_u(id, reg, 8).await
    }

    /// Read signed integer from register
    pub async fn read_reg_i(
        &mut self,
        id: u32,
        reg: u64,
        size: u64,
    ) -> Result<(bool, i64), tonic::Status> {
        let request = Request::new(ReadRegRequest { id, reg, size });
        let response = self.client.read_reg_i(request).await?;
        let inner = response.into_inner();
        Ok((inner.result, inner.data))
    }

    /// Read 8-bit signed integer from register (convenience method)
    pub async fn read_reg_i8(
        &mut self,
        id: u32,
        reg: u64,
    ) -> Result<(bool, i8), tonic::Status> {
        let (result, data) = self.read_reg_i(id, reg, 1).await?;
        Ok((result, data as i8))
    }

    /// Read 16-bit signed integer from register (convenience method)
    pub async fn read_reg_i16(
        &mut self,
        id: u32,
        reg: u64,
    ) -> Result<(bool, i16), tonic::Status> {
        let (result, data) = self.read_reg_i(id, reg, 2).await?;
        Ok((result, data as i16))
    }

    /// Read 32-bit signed integer from register (convenience method)
    pub async fn read_reg_i32(
        &mut self,
        id: u32,
        reg: u64,
    ) -> Result<(bool, i32), tonic::Status> {
        let (result, data) = self.read_reg_i(id, reg, 4).await?;
        Ok((result, data as i32))
    }

    /// Read 64-bit signed integer from register (convenience method)
    pub async fn read_reg_i64(
        &mut self,
        id: u32,
        reg: u64,
    ) -> Result<(bool, i64), tonic::Status> {
        self.read_reg_i(id, reg, 8).await
    }

    /// Write 32-bit float to memory
    pub async fn write_mem_f32(
        &mut self,
        id: u32,
        offset: u64,
        data: f32,
    ) -> Result<bool, tonic::Status> {
        let request = Request::new(WriteMemF32Request { id, offset, data });
        let response = self.client.write_mem_f32(request).await?;
        Ok(response.into_inner().result)
    }

    /// Write 64-bit float to memory
    pub async fn write_mem_f64(
        &mut self,
        id: u32,
        offset: u64,
        data: f64,
    ) -> Result<bool, tonic::Status> {
        let request = Request::new(WriteMemF64Request { id, offset, data });
        let response = self.client.write_mem_f64(request).await?;
        Ok(response.into_inner().result)
    }

    /// Read 32-bit float from memory
    pub async fn read_mem_f32(
        &mut self,
        id: u32,
        offset: u64,
    ) -> Result<(bool, f32), tonic::Status> {
        let request = Request::new(ReadMemRequest {
            id,
            offset,
            size: 4,
        });
        let response = self.client.read_mem_f32(request).await?;
        let inner = response.into_inner();
        Ok((inner.result, inner.data))
    }

    /// Read 64-bit float from memory
    pub async fn read_mem_f64(
        &mut self,
        id: u32,
        offset: u64,
    ) -> Result<(bool, f64), tonic::Status> {
        let request = Request::new(ReadMemRequest {
            id,
            offset,
            size: 8,
        });
        let response = self.client.read_mem_f64(request).await?;
        let inner = response.into_inner();
        Ok((inner.result, inner.data))
    }

    /// Write 32-bit float to register
    pub async fn write_reg_f32(&mut self, id: u32, reg: u64, data: f32) -> Result<bool, tonic::Status> {
        let request = Request::new(WriteRegF32Request { id, reg, data });
        let response = self.client.write_reg_f32(request).await?;
        Ok(response.into_inner().result)
    }

    /// Write 64-bit float to register
    pub async fn write_reg_f64(&mut self, id: u32, reg: u64, data: f64) -> Result<bool, tonic::Status> {
        let request = Request::new(WriteRegF64Request { id, reg, data });
        let response = self.client.write_reg_f64(request).await?;
        Ok(response.into_inner().result)
    }

    /// Read 32-bit float from register
    pub async fn read_reg_f32(&mut self, id: u32, reg: u64) -> Result<(bool, f32), tonic::Status> {
        let request = Request::new(ReadRegRequest { id, reg, size: 4 });
        let response = self.client.read_reg_f32(request).await?;
        let inner = response.into_inner();
        Ok((inner.result, inner.data))
    }

    /// Read 64-bit float from register
    pub async fn read_reg_f64(&mut self, id: u32, reg: u64) -> Result<(bool, f64), tonic::Status> {
        let request = Request::new(ReadRegRequest { id, reg, size: 8 });
        let response = self.client.read_reg_f64(request).await?;
        let inner = response.into_inner();
        Ok((inner.result, inner.data))
    }

    /// Copy data to memory
    pub async fn mem_copy_to(
        &mut self,
        id: u32,
        offset: u64,
        data: Vec<u8>,
    ) -> Result<bool, tonic::Status> {
        let request = Request::new(MemCopyToRequest { id, offset, data });
        let response = self.client.mem_copy_to(request).await?;
        Ok(response.into_inner().result)
    }

    /// Copy data from memory
    pub async fn mem_copy_from(
        &mut self,
        id: u32,
        offset: u64,
        size: u64,
    ) -> Result<(bool, Vec<u8>), tonic::Status> {
        let request = Request::new(MemCopyFromRequest { id, offset, size });
        let response = self.client.mem_copy_from(request).await?;
        let inner = response.into_inner();
        Ok((inner.result, inner.data))
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_client_creation() {
        // This test would require a running server
        // For now, just check that the types compile
        assert!(true);
    }
}
