use crate::graphics::allocators::Allocators;
use std::sync::Arc;
use std::time::Duration;
use vulkano::VulkanLibrary;
use vulkano::device::{Device, Queue};
use vulkano::instance::Instance;
use vulkano::swapchain::Surface;
use winit::event_loop::EventLoop;
use winit::window::{CursorGrabMode, Window};

pub struct VulkanoCore {
    pub allocators: Allocators,
    pub window: Arc<Window>,
    pub queue: Arc<Queue>,
    pub device: Arc<Device>,
    pub surface: Arc<Surface>,
}

impl VulkanoCore {
    /// initializes basic vulkano components and creates a window
    pub fn new() -> (Self, EventLoop<()>) {
        let event_loop = EventLoop::new();

        let library = VulkanLibrary::new().unwrap();

        let required_extensions = vulkano::instance::InstanceExtensions {
            ext_debug_utils: true,
            ext_validation_features: true,
            ..Surface::required_extensions(&event_loop)
        };

        let validation_features =
            vec![vulkano::instance::debug::ValidationFeatureEnable::DebugPrintf];

        let instance = Instance::new(
            library,
            vulkano::instance::InstanceCreateInfo {
                flags: vulkano::instance::InstanceCreateFlags::ENUMERATE_PORTABILITY,
                enabled_extensions: required_extensions,
                enabled_validation_features: validation_features,
                enabled_layers: vec!["VK_LAYER_KHRONOS_validation".to_owned()],
                ..Default::default()
            },
        )
        .unwrap();

        let window = Arc::new(
            winit::window::WindowBuilder::new()
                .build(&event_loop)
                .unwrap(),
        );

        std::thread::sleep(Duration::from_millis(200));

        window.set_cursor_grab(CursorGrabMode::Confined).expect("Couldn't confine Cursor!");
        window.set_cursor_visible(false);
        window.set_cursor_hittest(false);

        let surface = Surface::from_window(instance.clone(), window.clone()).unwrap();

        let device_extensions = vulkano::device::DeviceExtensions {
            khr_swapchain: true,
            khr_shader_non_semantic_info: true,
            ..Default::default()
        };

        let (physical_device, queue_family_index) =
            get_physical_device(instance, &surface, &device_extensions);

        let (device, mut queues) = Device::new(
            physical_device,
            vulkano::device::DeviceCreateInfo {
                enabled_extensions: device_extensions,
                queue_create_infos: vec![vulkano::device::QueueCreateInfo {
                    queue_family_index,
                    ..Default::default()
                }],

                ..Default::default()
            },
        )
        .unwrap();

        let queue = queues.next().unwrap();

        let allocators = Allocators::new(device.clone());
        (
            Self {
                device,
                allocators,
                window,
                queue,
                surface,
            },
            event_loop
        )
    }
}

pub fn get_physical_device(
    instance: Arc<Instance>,
    surface: &Arc<Surface>,
    device_extensions: &vulkano::device::DeviceExtensions,
) -> (Arc<vulkano::device::physical::PhysicalDevice>, u32) {
    use vulkano::device::physical::PhysicalDeviceType;

    let (physical_device, queue_family_index) = instance
        .enumerate_physical_devices()
        .unwrap()
        .filter(|p| p.supported_extensions().contains(&device_extensions))
        .filter_map(|p| {
            p.queue_family_properties()
                .iter()
                .enumerate()
                .position(|(i, q)| {
                    q.queue_flags
                        .intersects(vulkano::device::QueueFlags::GRAPHICS)
                        && p.surface_support(i as u32, &surface).unwrap_or(false)
                })
                .map(|i| (p, i as u32))
        })
        .min_by_key(|(p, _)| match p.properties().device_type {
            PhysicalDeviceType::DiscreteGpu => 0,
            PhysicalDeviceType::IntegratedGpu => 1,
            PhysicalDeviceType::VirtualGpu => 2,
            PhysicalDeviceType::Cpu => 3,
            PhysicalDeviceType::Other => 4,
            _ => 5,
        })
        .expect("no suitable physical device found");
    (physical_device, queue_family_index)
}
