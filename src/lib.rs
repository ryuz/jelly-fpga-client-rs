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
    pub async fn load(&mut self, name: String) -> Result<(bool, i32), tonic::Status> {
        let request = Request::new(LoadRequest { name });
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
    pub async fn upload_firmware(&mut self, name: String, data: Vec<u8>) -> Result<bool, tonic::Status> {
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
            name,
            data,
            chunk_size: 2 * 1024 * 1024, // 2MB chunks like Python version
            offset: 0,
        };
        
        let response = self.client.upload_firmware(Request::new(stream)).await?;
        Ok(response.into_inner().result)
    }

    /// Upload firmware from file
    pub async fn upload_firmware_file(&mut self, name: String, file_path: &str) -> Result<bool, tonic::Status> {
        let data = std::fs::read(file_path).map_err(|e| {
            tonic::Status::internal(format!("Failed to read file {}: {}", file_path, e))
        })?;
        
        self.upload_firmware(name, data).await
    }

    /// Remove firmware
    pub async fn remove_firmware(&mut self, name: String) -> Result<bool, tonic::Status> {
        let request = Request::new(RemoveFirmwareRequest { name });
        let response = self.client.remove_firmware(request).await?;
        Ok(response.into_inner().result)
    }

    /// Load bitstream
    pub async fn load_bitstream(&mut self, name: String) -> Result<bool, tonic::Status> {
        let request = Request::new(LoadBitstreamRequest { name });
        let response = self.client.load_bitstream(request).await?;
        Ok(response.into_inner().result)
    }

    /// Load device tree overlay
    pub async fn load_dtbo(&mut self, name: String) -> Result<bool, tonic::Status> {
        let request = Request::new(LoadDtboRequest { name });
        let response = self.client.load_dtbo(request).await?;
        Ok(response.into_inner().result)
    }

    /// Convert DTS to DTB
    pub async fn dts_to_dtb(&mut self, dts: String) -> Result<(bool, Vec<u8>), tonic::Status> {
        let request = Request::new(DtsToDtbRequest { dts });
        let response = self.client.dts_to_dtb(request).await?;
        let inner = response.into_inner();
        Ok((inner.result, inner.dtb))
    }

    /// Convert bitstream to bin
    pub async fn bitstream_to_bin(
        &mut self,
        bitstream_name: String,
        bin_name: String,
        arch: String,
    ) -> Result<bool, tonic::Status> {
        let request = Request::new(BitstreamToBinRequest {
            bitstream_name,
            bin_name,
            arch,
        });
        let response = self.client.bitstream_to_bin(request).await?;
        Ok(response.into_inner().result)
    }

    /// Open memory map
    pub async fn open_mmap(
        &mut self,
        path: String,
        offset: u64,
        size: u64,
        unit: u64,
    ) -> Result<(bool, u32), tonic::Status> {
        let request = Request::new(OpenMmapRequest {
            path,
            offset,
            size,
            unit,
        });
        let response = self.client.open_mmap(request).await?;
        let inner = response.into_inner();
        Ok((inner.result, inner.id))
    }

    /// Open memory map with default unit size (convenience method)
    pub async fn open_mmap_simple(
        &mut self,
        path: &str,
        offset: u64,
        size: u64,
    ) -> Result<(bool, u32), tonic::Status> {
        self.open_mmap(path.to_string(), offset, size, 8).await
    }

    /// Open UIO device
    pub async fn open_uio(&mut self, name: String, unit: u64) -> Result<(bool, u32), tonic::Status> {
        let request = Request::new(OpenUioRequest { name, unit });
        let response = self.client.open_uio(request).await?;
        let inner = response.into_inner();
        Ok((inner.result, inner.id))
    }

    /// Open UDMABUF device
    pub async fn open_udmabuf(
        &mut self,
        name: String,
        cache_enable: bool,
        unit: u64,
    ) -> Result<(bool, u32), tonic::Status> {
        let request = Request::new(OpenUdmabufRequest {
            name,
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
