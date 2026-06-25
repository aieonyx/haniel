// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// axon_gpu stub — CI build only, no Vulkan/shader compilation

#[derive(Debug, Clone)]
pub struct GpuBuffer { pub data: Vec<f32>, pub len: usize }

#[derive(Debug, Clone)]
pub enum BufferKind { Compute, Staging }

#[derive(Debug)]
pub enum GpuError { NotAvailable, OutOfMemory(String) }

pub type GpuResult<T> = Result<T, GpuError>;

#[derive(Debug)]
pub struct GpuDevice;

#[derive(Debug, Clone)]
pub enum GpuBackend { Cpu, Vulkan }

#[derive(Debug)]
pub struct GpuCapabilities { pub backend: GpuBackend }

#[derive(Debug)]
pub struct GpuKernel;

#[derive(Debug, Clone)]
pub enum KernelOp { Add, Mul, Scale, Relu, Matmul }

impl GpuBuffer {
    pub fn zeros(kind: BufferKind, len: usize) -> GpuResult<Self> {
        Ok(Self { data: vec![0.0; len], len })
    }
    pub fn from_slice(kind: BufferKind, data: &[f32]) -> GpuResult<Self> {
        Ok(Self { data: data.to_vec(), len: data.len() })
    }
    pub fn to_vec(&self) -> Vec<f32> { self.data.clone() }
    pub fn get(&self, idx: usize) -> GpuResult<f32> {
        self.data.get(idx).copied().ok_or(GpuError::NotAvailable)
    }
    pub fn set(&mut self, idx: usize, val: f32) -> GpuResult<()> {
        if idx < self.len { self.data[idx] = val; Ok(()) }
        else { Err(GpuError::NotAvailable) }
    }
    pub fn fill(&mut self, val: f32) { self.data.fill(val); }
    pub fn as_slice(&self) -> &[f32] { &self.data }
    pub fn as_mut_slice(&mut self) -> &mut [f32] { &mut self.data }
    pub fn size_bytes(&self) -> usize { self.len * 4 }
}
