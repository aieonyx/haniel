// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// haniel_onyx::gpu_bridge — axon_gpu integration for AI inference
// Bridges GpuDevice/GpuKernel to HANIEL-ONYX compute pipeline

use axon_gpu::{GpuDevice, GpuKernel, GpuBuffer, BufferKind, KernelOp, GpuBackend};
use crate::OnyxError;

/// Sovereign GPU compute bridge for HANIEL-ONYX
pub struct OnxGpuBridge {
    device: GpuDevice,
}

impl OnxGpuBridge {
    /// Initialize — discovers best available GPU (Vulkan or CPU fallback)
    pub fn new() -> Result<Self, OnyxError> {
        let device = GpuDevice::discover()
            .map_err(|e| OnyxError::InferenceFailed(format!("GPU init: {:?}", e)))?;
        Ok(Self { device })
    }

    /// Create CPU fallback bridge (always available)
    pub fn cpu() -> Self {
        Self { device: GpuDevice::cpu_fallback() }
    }

    /// Run a vector addition on GPU (test operation)
    pub fn add(
        &self,
        a: &[f32],
        b: &[f32],
    ) -> Result<Vec<f32>, OnyxError> {
        if a.len() != b.len() {
            return Err(OnyxError::InferenceFailed("shape mismatch".into()));
        }
        let buf_a = GpuBuffer::from_slice(BufferKind::Input, a)
            .map_err(|e| OnyxError::InferenceFailed(format!("{:?}", e)))?;
        let buf_b = GpuBuffer::from_slice(BufferKind::Input, b)
            .map_err(|e| OnyxError::InferenceFailed(format!("{:?}", e)))?;
        let mut out = GpuBuffer::zeros(BufferKind::Output, a.len())
            .map_err(|e| OnyxError::InferenceFailed(format!("{:?}", e)))?;
        let kernel = GpuKernel::new(KernelOp::Add);
        kernel.dispatch(&self.device, &[&buf_a, &buf_b], &mut out)
            .map_err(|e| OnyxError::InferenceFailed(format!("{:?}", e)))?;
        Ok(out.to_vec())
    }

    /// Run ReLU activation on GPU
    pub fn relu(&self, input: &[f32]) -> Result<Vec<f32>, OnyxError> {
        let buf_in = GpuBuffer::from_slice(BufferKind::Input, input)
            .map_err(|e| OnyxError::InferenceFailed(format!("{:?}", e)))?;
        let mut out = GpuBuffer::zeros(BufferKind::Output, input.len())
            .map_err(|e| OnyxError::InferenceFailed(format!("{:?}", e)))?;
        let kernel = GpuKernel::new(KernelOp::ReLU);
        kernel.dispatch(&self.device, &[&buf_in], &mut out)
            .map_err(|e| OnyxError::InferenceFailed(format!("{:?}", e)))?;
        Ok(out.to_vec())
    }

    /// Matrix multiply on GPU — core of transformer attention
    pub fn matmul(
        &self,
        a:     &[f32],
        b:     &[f32],
        rows:  usize,
        cols:  usize,
        inner: usize,
    ) -> Result<Vec<f32>, OnyxError> {
        let buf_a = GpuBuffer::from_slice(BufferKind::Input, a)
            .map_err(|e| OnyxError::InferenceFailed(format!("{:?}", e)))?;
        let buf_b = GpuBuffer::from_slice(BufferKind::Input, b)
            .map_err(|e| OnyxError::InferenceFailed(format!("{:?}", e)))?;
        let mut out = GpuBuffer::zeros(BufferKind::Output, rows * cols)
            .map_err(|e| OnyxError::InferenceFailed(format!("{:?}", e)))?;
        let kernel = GpuKernel::new(KernelOp::MatMul { rows, cols, inner });
        kernel.dispatch(&self.device, &[&buf_a, &buf_b], &mut out)
            .map_err(|e| OnyxError::InferenceFailed(format!("{:?}", e)))?;
        Ok(out.to_vec())
    }

    /// Scale a vector by a scalar
    pub fn scale(&self, input: &[f32], scalar: f32) -> Result<Vec<f32>, OnyxError> {
        let buf_in = GpuBuffer::from_slice(BufferKind::Input, input)
            .map_err(|e| OnyxError::InferenceFailed(format!("{:?}", e)))?;
        let mut out = GpuBuffer::zeros(BufferKind::Output, input.len())
            .map_err(|e| OnyxError::InferenceFailed(format!("{:?}", e)))?;
        let kernel = GpuKernel::new(KernelOp::Scale(scalar));
        kernel.dispatch(&self.device, &[&buf_in], &mut out)
            .map_err(|e| OnyxError::InferenceFailed(format!("{:?}", e)))?;
        Ok(out.to_vec())
    }

    /// Backend name
    pub fn backend_name(&self) -> String {
        self.device.device_name().to_string()
    }

    /// Whether running on real GPU
    pub fn is_gpu(&self) -> bool {
        !self.device.is_cpu_fallback()
    }

    /// GPU memory available
    pub fn vram_bytes(&self) -> usize {
        self.device.vram_bytes()
    }
}

impl Default for OnxGpuBridge {
    fn default() -> Self { Self::cpu() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bridge() -> OnxGpuBridge { OnxGpuBridge::cpu() }

    #[test]
    fn bridge_constructs_cpu() {
        let b = bridge();
        assert!(!b.is_gpu());
        assert!(!b.backend_name().is_empty());
    }

    #[test]
    fn bridge_add_vectors() {
        let b = bridge();
        let a = vec![1.0, 2.0, 3.0];
        let c = vec![4.0, 5.0, 6.0];
        let r = b.add(&a, &c).unwrap();
        assert_eq!(r, vec![5.0, 7.0, 9.0]);
    }

    #[test]
    fn bridge_relu_clamps_negatives() {
        let b    = bridge();
        let data = vec![-1.0, 0.0, 1.0, 2.0, -3.0];
        let r    = b.relu(&data).unwrap();
        assert_eq!(r, vec![0.0, 0.0, 1.0, 2.0, 0.0]);
    }

    #[test]
    fn bridge_scale() {
        let b = bridge();
        let r = b.scale(&[1.0, 2.0, 3.0], 2.0).unwrap();
        assert_eq!(r, vec![2.0, 4.0, 6.0]);
    }

    #[test]
    fn bridge_matmul_2x2() {
        let b = bridge();
        // [1,2] x [5,6] = [1*5+2*7, 1*6+2*8] = [19, 22]
        // [3,4]   [7,8]   [3*5+4*7, 3*6+4*8]   [43, 50]
        let a = vec![1.0, 2.0, 3.0, 4.0];
        let c = vec![5.0, 6.0, 7.0, 8.0];
        let r = b.matmul(&a, &c, 2, 2, 2).unwrap();
        assert!((r[0] - 19.0).abs() < 0.001);
        assert!((r[1] - 22.0).abs() < 0.001);
        assert!((r[2] - 43.0).abs() < 0.001);
        assert!((r[3] - 50.0).abs() < 0.001);
    }

    #[test]
    fn bridge_vram_zero_on_cpu() {
        assert_eq!(bridge().vram_bytes(), 0);
    }

    #[test]
    fn bridge_add_shape_mismatch_errors() {
        let b = bridge();
        let r = b.add(&[1.0, 2.0], &[1.0]);
        assert!(r.is_err());
    }
}
