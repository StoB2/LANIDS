use std::{
    sync::mpsc,
    time,
};
use super::super::{GPUComputeSet, GPU_PARALLEL};
use crate::Packet;

pub struct ClosingPipeline {
    device: wgpu::Device,
    queue: wgpu::Queue,

    bind_group: wgpu::BindGroup,

    packet_storage_buffer: wgpu::Buffer,
    result_storage_buffer: wgpu::Buffer,

    compute_pipeline: wgpu::ComputePipeline,
}

impl ClosingPipeline {
    pub fn new(GPUComputeSet{device, queue, bind_group_layout, pipeline_layout}: GPUComputeSet) -> Self {
        use wgpu::util::DeviceExt;

        let packets = [Packet([0;Packet::BYTE_COUNT]); GPU_PARALLEL];
        let result = [0u32; GPU_PARALLEL];
        
        let packet_storage_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("closing_packet_buffer"),
            contents: bytemuck::cast_slice(&packets),
            usage: wgpu::BufferUsage::STORAGE
                | wgpu::BufferUsage::COPY_DST,
        });

        let result_storage_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("closing_result_buffer"),
            contents: bytemuck::cast_slice(&result),
            usage: wgpu::BufferUsage::STORAGE
                | wgpu::BufferUsage::COPY_SRC,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer( packet_storage_buffer.slice(..) ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer( result_storage_buffer.slice(..) ),
                },
            ],
        });

        let cs_spirv = wgpu::include_spirv!("../../../../res/shaders/closing/spir-v/closing-comp.spv");
        let cs_module = device.create_shader_module(cs_spirv);
        
        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("closing_pipeline"),
            layout: Some(&pipeline_layout),
            compute_stage: wgpu::ProgrammableStageDescriptor {
                module: &cs_module,
                entry_point: "main",
            },
        });

        Self{
            device,
            queue,
            bind_group,
            packet_storage_buffer,
            result_storage_buffer,
            compute_pipeline,
        }
    }

    pub fn scan(&self, packets: Vec<Packet>, outbox: mpsc::Sender<(Packet, time::SystemTime)>) {
        use wgpu::util::DeviceExt;
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let packet_staging_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("closing_packet_buffer"),
            contents: bytemuck::cast_slice(&packets),
            usage: wgpu::BufferUsage::STORAGE
                | wgpu::BufferUsage::COPY_SRC,
        });

        encoder.copy_buffer_to_buffer(
            &packet_staging_buffer, 0, &self.packet_storage_buffer, 0,
            (std::mem::size_of::<Packet>() * packets.len()) as u64,
        );

        {
            let mut cpass = encoder.begin_compute_pass();
            cpass.set_pipeline(&self.compute_pipeline);
            cpass.set_bind_group(0, &self.bind_group, &[]);
            cpass.dispatch(packets.len() as u32, 1, 1);
        }

        let result_read_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("closing_result_read_buffer"),
            size: (std::mem::size_of::<u32>() * GPU_PARALLEL) as u64,
            usage: wgpu::BufferUsage::COPY_DST
                | wgpu::BufferUsage::MAP_READ,
            mapped_at_creation: false,
        });

        encoder.copy_buffer_to_buffer(
            &self.result_storage_buffer, 0, &result_read_buffer, 0,
            (std::mem::size_of::<u32>() * GPU_PARALLEL) as u64,
        );

        self.queue.submit(Some(encoder.finish()));

        let buffer_slice = result_read_buffer.slice(..);
        let buffer_future = buffer_slice.map_async(wgpu::MapMode::Read);
        self.device.poll(wgpu::Maintain::Wait);

        if let Ok(()) = futures::executor::block_on(buffer_future) {
            let data = buffer_slice.get_mapped_range();

            let result = data
                .chunks_exact(4)
                .map(|b| b[0] > 0)
                .collect::<Vec<bool>>();

            drop(data);
            result_read_buffer.unmap();

            for bit in 0..packets.len() {
                if result[bit] {
                    outbox.send((packets[bit], time::SystemTime::now())).unwrap();
                }
            }
        }
    }
}