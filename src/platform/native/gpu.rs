//! Headless GPU compute context initialization and running utilities.
//!
//! **Taxonomy Classification**: Platform & Architecture (Deployment - Native).
//!
//! This module provides a high-level wrapper to initialize a headless GPU context
//! using `wgpu` and to execute simple compute shaders without requiring a window or surface.

use wgpu::{Device, Queue, Instance, PowerPreference, RequestAdapterOptions};
use std::sync::OnceLock;

static HEADLESS_GPU: OnceLock<Option<(Device, Queue)>> = OnceLock::new();

/// Initialize the headless GPU context (Device and Queue) synchronously.
///
/// This uses `pollster` to run the async adapter and device requests on a block_on queue,
/// caching the resulting handles globally so subsequent calls are cheap.
pub fn init_headless_gpu() -> Option<(Device, Queue)> {
    let opt = HEADLESS_GPU.get_or_init(|| {
        pollster::block_on(async {
            let instance = Instance::default();
            let adapter = instance
                .request_adapter(&RequestAdapterOptions {
                    power_preference: PowerPreference::HighPerformance,
                    force_fallback_adapter: false,
                    compatible_surface: None,
                })
                .await?;
            
            let (device, queue) = adapter
                .request_device(
                    &wgpu::DeviceDescriptor {
                        label: Some("library Headless GPU Device"),
                        required_features: wgpu::Features::empty(),
                        required_limits: wgpu::Limits::downlevel_defaults(),
                        memory_hints: wgpu::MemoryHints::default(),
                    },
                    None,
                )
                .await
                .ok()?;
            Some((device, queue))
        })
    });

    opt.clone()
}

/// Helper to execute a 1D compute shader with an input float array, returning the modified array.
///
/// Automatically handles buffer generation, bind groups, command encoding, command submission,
/// and mapping results back from the GPU to CPU memory.
///
/// # Arguments
/// * `shader_src` - WGSL shader source code containing the `@compute` entry point.
/// * `entry_point` - Name of the entry point function (usually `"main"`).
/// * `data` - Input vector of `f32` floats to process.
pub fn run_compute_shader(shader_src: &str, entry_point: &str, data: &[f32]) -> Option<Vec<f32>> {
    let (device, queue) = init_headless_gpu()?;
    
    // Create shader module
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("library Compute Shader"),
        source: wgpu::ShaderSource::Wgsl(shader_src.into()),
    });

    // Create GPU buffers
    let size = std::mem::size_of_val(data) as u64;
    
    // Buffer for input/output storage on GPU
    let storage_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Storage Buffer"),
        size,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    
    // Write initial data to buffer
    queue.write_buffer(&storage_buffer, 0, bytemuck::cast_slice(data));

    // Staging buffer to read back results to CPU
    let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Staging Buffer"),
        size,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    // Create bind group layout and bind group
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Bind Group Layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Bind Group"),
        layout: &bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: storage_buffer.as_entire_binding(),
        }],
    });

    // Create compute pipeline
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Pipeline Layout"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });

    let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("Compute Pipeline"),
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: Some(entry_point),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    });

    // Dispatch compute pass
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Command Encoder"),
    });

    {
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Compute Pass"),
            timestamp_writes: None,
        });
        compute_pass.set_pipeline(&compute_pipeline);
        compute_pass.set_bind_group(0, &bind_group, &[]);
        let workgroup_size = 64;
        let workgroups = (data.len() as u32).div_ceil(workgroup_size);
        compute_pass.dispatch_workgroups(workgroups, 1, 1);
    }

    // Copy results to staging buffer
    encoder.copy_buffer_to_buffer(&storage_buffer, 0, &staging_buffer, 0, size);

    // Submit commands to queue
    queue.submit(Some(encoder.finish()));

    // Map staging buffer to read back
    let buffer_slice = staging_buffer.slice(..);
    let (sender, receiver) = std::sync::mpsc::channel();
    buffer_slice.map_async(wgpu::MapMode::Read, move |v| {
        let _ = sender.send(v);
    });

    // Poll the GPU to trigger map completion
    device.poll(wgpu::Maintain::Wait);

    match receiver.recv() {
        Ok(Ok(())) => {
            let data_raw = buffer_slice.get_mapped_range();
            let result: Vec<f32> = bytemuck::cast_slice(&data_raw).to_vec();
            drop(data_raw);
            staging_buffer.unmap();
            Some(result)
        }
        Ok(Err(e)) => {
            eprintln!("wgpu compute shader buffer mapping failed: {:?}", e);
            None
        }
        Err(e) => {
            eprintln!("wgpu compute shader map channel communication failed: {:?}", e);
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_headless_gpu_compute() {
        // Simple compute shader that multiplies each element in a buffer by 2.0
        let shader_src = r#"
            @group(0) @binding(0)
            var<storage, read_write> data: array<f32>;

            @compute @workgroup_size(64)
            fn main(@builtin(global_invocation_id) id: vec3<u32>) {
                if (id.x < arrayLength(&data)) {
                    data[id.x] = data[id.x] * 2.0;
                }
            }
        "#;

        let input = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        if let Some(output) = run_compute_shader(shader_src, "main", &input) {
            assert_eq!(output, vec![2.0, 4.0, 6.0, 8.0, 10.0]);
        } else {
            // Note: In CI environments without GPU hardware drivers, this might fail or return None,
            // which we can handle gracefully rather than failing the test unconditionally.
            println!("Headless GPU not available on this platform/runner. Skipping assertion.");
        }
    }
}
