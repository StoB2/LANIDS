pub mod prefilter;
mod closing;

use std::sync::mpsc;
use super::Packet;

const CPU_PARALLEL: usize = 4;
const GPU_PARALLEL: usize = 256;

pub struct GPUComputeSet{
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,

    pub bind_group_layout: wgpu::BindGroupLayout,
    pub pipeline_layout: wgpu::PipelineLayout,
}

impl GPUComputeSet {
    pub async fn new_pair() -> (Self, Self) {
        let instance = wgpu::Instance::new(wgpu::BackendBit::all());

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .unwrap();

        (Self::new(&adapter).await, Self::new(&adapter).await)
    }

    async fn new(adapter: &wgpu::Adapter) -> Self {
        let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
                shader_validation: true,
            },
            None,
        )
        .await
        .unwrap();

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::COMPUTE,
                    ty: wgpu::BindingType::StorageBuffer {
                        dynamic: false,
                        readonly: true,
                        min_binding_size: wgpu::BufferSize::new(
                            (std::mem::size_of::<crate::Packet>() * GPU_PARALLEL) as wgpu::BufferAddress
                        ),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStage::COMPUTE,
                    ty: wgpu::BindingType::StorageBuffer {
                        dynamic: false,
                        readonly: true,
                        min_binding_size: wgpu::BufferSize::new(
                            (std::mem::size_of::<u32>() * GPU_PARALLEL) as wgpu::BufferAddress
                        ),
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        Self{
            device,
            queue,
            bind_group_layout,
            pipeline_layout,
        }
    }
}

pub fn start_analysers(
    producer: mpsc::Receiver<Packet>,
    responder: mpsc::Sender<(Packet, std::time::SystemTime, u8)>,
) -> Vec<std::thread::JoinHandle<()>> {
    let (closing_tx, closing_rx) = mpsc::channel();

    let mut threads = Vec::new();

    let (prefilter_gpu, closing_gpu) = futures::executor::block_on(GPUComputeSet::new_pair());
    let prefilter_set = prefilter::Prefilter::new(prefilter_gpu, producer, closing_tx);
    let closing_set = closing::Closing::new(closing_gpu, closing_rx, responder);

    threads.push(std::thread::spawn(move || {
        prefilter_set.run();
    }));

    threads.push(std::thread::spawn(move || {
        closing_set.run();
    }));

    threads
}