use ash::vk;
use ash::prelude::VkResult;
use std::mem::ManuallyDrop;

use ash::version::DeviceV1_0;

use renderer::vulkan::{VkDeviceContext, MsaaLevel};
use renderer::vulkan::VkSwapchain;
use renderer::vulkan::SwapchainInfo;
use renderer::vulkan::VkQueueFamilyIndices;

use renderer::vulkan::VkImage;

use atelier_assets::loader::handle::Handle;

use renderer::assets::resources::{
    PipelineSwapchainInfo, DynDescriptorSet, ResourceManager, ResourceArc, ImageViewResource,
    ResourceLookupSet, RenderPassResource, FramebufferResource,
};
use renderer::assets::MaterialAsset;
use crate::game_renderer::RenderpassAttachmentImage;
use renderer::assets::vk_description as dsc;

pub struct VkBloomRenderPassResources {
    pub device_context: VkDeviceContext,
    pub bloom_blur_material: Handle<MaterialAsset>,
    pub bloom_images: [ResourceArc<ImageViewResource>; 2],
    pub bloom_image_descriptor_sets: [DynDescriptorSet; 2],
    pub color_image: ResourceArc<ImageViewResource>,
}

impl VkBloomRenderPassResources {
    pub fn new(
        device_context: &VkDeviceContext,
        swapchain: &VkSwapchain,
        resource_manager: &mut ResourceManager,
        bloom_blur_material: Handle<MaterialAsset>,
    ) -> VkResult<Self> {
        let bloom_image0 = RenderpassAttachmentImage::create_resource(
            resource_manager.resources_mut(),
            device_context,
            &swapchain.swapchain_info,
            swapchain.color_format,
            vk::ImageAspectFlags::COLOR,
            vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::SAMPLED,
            MsaaLevel::Sample1,
        )?;

        let bloom_image1 = RenderpassAttachmentImage::create_resource(
            resource_manager.resources_mut(),
            device_context,
            &swapchain.swapchain_info,
            swapchain.color_format,
            vk::ImageAspectFlags::COLOR,
            vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::SAMPLED,
            MsaaLevel::Sample1,
        )?;

        let color_image = RenderpassAttachmentImage::create_resource(
            resource_manager.resources_mut(),
            device_context,
            &swapchain.swapchain_info,
            swapchain.color_format,
            vk::ImageAspectFlags::COLOR,
            vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::SAMPLED,
            MsaaLevel::Sample1,
        )?;

        log::trace!("bloom_image0: {:?}", bloom_image0);
        log::trace!("bloom_image1: {:?}", bloom_image1);
        log::trace!("color_image: {:?}", color_image);

        let bloom_blur_layout =
            resource_manager.get_descriptor_set_info(&bloom_blur_material, 0, 0);

        let mut descriptor_set_allocator = resource_manager.create_descriptor_set_allocator();
        let mut bloom_blur_material_dyn_set0 = descriptor_set_allocator
            .create_dyn_descriptor_set_uninitialized(&bloom_blur_layout.descriptor_set_layout)?;
        bloom_blur_material_dyn_set0.set_image_raw(0, bloom_image0.get_raw().image_view);
        bloom_blur_material_dyn_set0.set_buffer_data(2, &(0 as u32));
        bloom_blur_material_dyn_set0.flush(&mut descriptor_set_allocator)?;

        let mut bloom_blur_material_dyn_set1 = descriptor_set_allocator
            .create_dyn_descriptor_set_uninitialized(&bloom_blur_layout.descriptor_set_layout)?;
        bloom_blur_material_dyn_set1.set_image_raw(0, bloom_image1.get_raw().image_view);
        bloom_blur_material_dyn_set1.set_buffer_data(2, &(1 as u32));
        bloom_blur_material_dyn_set1.flush(&mut descriptor_set_allocator)?;

        Ok(VkBloomRenderPassResources {
            device_context: device_context.clone(),
            bloom_blur_material,
            bloom_images: [bloom_image0, bloom_image1],
            bloom_image_descriptor_sets: [
                bloom_blur_material_dyn_set0,
                bloom_blur_material_dyn_set1,
            ],
            color_image,
        })
    }
}

pub struct VkBloomExtractRenderPass {
    pub device_context: VkDeviceContext,
    pub swapchain_info: SwapchainInfo,

    pipeline_info: PipelineSwapchainInfo,

    pub frame_buffers: Vec<ResourceArc<FramebufferResource>>,

    // Command pool and list of command buffers, one per present index
    pub command_pool: vk::CommandPool,
    pub command_buffers: Vec<vk::CommandBuffer>,
}

impl VkBloomExtractRenderPass {
    pub fn new(
        resources: &mut ResourceLookupSet,
        device_context: &VkDeviceContext,
        swapchain_info: &SwapchainInfo,
        swapchain_images: &[ResourceArc<ImageViewResource>],
        pipeline_info: PipelineSwapchainInfo,
        bloom_resources: &VkBloomRenderPassResources,
    ) -> VkResult<Self> {
        //
        // Command Buffers
        //
        let command_pool = Self::create_command_pool(
            &device_context.device(),
            &device_context.queue_family_indices(),
        )?;

        //
        // Renderpass Resources
        //
        let frame_buffers = Self::create_framebuffers(
            resources,
            &bloom_resources.bloom_images[0],
            &bloom_resources.color_image,
            swapchain_images,
            swapchain_info,
            &pipeline_info.pipeline.get_raw().renderpass,
        )?;

        let command_buffers =
            Self::create_command_buffers(&device_context.device(), swapchain_info, &command_pool)?;

        Ok(VkBloomExtractRenderPass {
            device_context: device_context.clone(),
            swapchain_info: swapchain_info.clone(),
            pipeline_info,
            frame_buffers,
            command_pool,
            command_buffers,
        })
    }

    fn create_command_pool(
        logical_device: &ash::Device,
        queue_family_indices: &VkQueueFamilyIndices,
    ) -> VkResult<vk::CommandPool> {
        log::trace!(
            "Creating command pool with queue family index {}",
            queue_family_indices.graphics_queue_family_index
        );
        let pool_create_info = vk::CommandPoolCreateInfo::builder()
            .flags(
                vk::CommandPoolCreateFlags::TRANSIENT
                    | vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
            )
            .queue_family_index(queue_family_indices.graphics_queue_family_index);

        unsafe { logical_device.create_command_pool(&pool_create_info, None) }
    }

    fn create_framebuffers(
        resources: &mut ResourceLookupSet,
        bloom_image_view: &ResourceArc<ImageViewResource>,
        color_image_view: &ResourceArc<ImageViewResource>,
        swapchain_image_views: &[ResourceArc<ImageViewResource>],
        swapchain_info: &SwapchainInfo,
        renderpass: &ResourceArc<RenderPassResource>,
    ) -> VkResult<Vec<ResourceArc<FramebufferResource>>> {
        swapchain_image_views
            .iter()
            .map(|_swapchain_image_view| {
                let framebuffer_meta = dsc::FramebufferMeta {
                    width: swapchain_info.extents.width,
                    height: swapchain_info.extents.height,
                    layers: 1,
                };

                let attachments = [color_image_view.clone(), bloom_image_view.clone()];
                resources.get_or_create_framebuffer(
                    renderpass.clone(),
                    &attachments,
                    &framebuffer_meta,
                )
            })
            .collect()
    }

    fn create_command_buffers(
        logical_device: &ash::Device,
        swapchain_info: &SwapchainInfo,
        command_pool: &vk::CommandPool,
    ) -> VkResult<Vec<vk::CommandBuffer>> {
        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_buffer_count(swapchain_info.image_count as u32)
            .command_pool(*command_pool)
            .level(vk::CommandBufferLevel::PRIMARY);

        unsafe { logical_device.allocate_command_buffers(&command_buffer_allocate_info) }
    }

    #[allow(clippy::too_many_arguments)]
    fn update_command_buffer(
        device_context: &VkDeviceContext,
        swapchain_info: &SwapchainInfo,
        renderpass: vk::RenderPass,
        framebuffer: vk::Framebuffer,
        command_buffer: vk::CommandBuffer,
        pipeline: vk::Pipeline,
        pipeline_layout: vk::PipelineLayout,
        descriptor_set: vk::DescriptorSet,
    ) -> VkResult<()> {
        let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder();

        let clear_values = [
            vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.0, 0.0, 0.0, 1.0],
                },
            },
            vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.0, 0.0, 0.0, 1.0],
                },
            },
        ];

        let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
            .render_pass(renderpass)
            .framebuffer(framebuffer)
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: swapchain_info.extents,
            })
            .clear_values(&clear_values);

        // Implicitly resets the command buffer
        unsafe {
            let logical_device = device_context.device();
            logical_device.begin_command_buffer(command_buffer, &command_buffer_begin_info)?;

            logical_device.cmd_begin_render_pass(
                command_buffer,
                &render_pass_begin_info,
                vk::SubpassContents::INLINE,
            );

            logical_device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline,
            );

            logical_device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline_layout,
                0,
                &[descriptor_set],
                &[],
            );

            logical_device.cmd_draw(command_buffer, 3, 1, 0, 0);

            logical_device.cmd_end_render_pass(command_buffer);
            logical_device.end_command_buffer(command_buffer)
        }
    }

    pub fn update(
        &mut self,
        present_index: usize,
        descriptor_set: vk::DescriptorSet,
    ) -> VkResult<()> {
        Self::update_command_buffer(
            &self.device_context,
            &self.swapchain_info,
            self.pipeline_info
                .pipeline
                .get_raw()
                .renderpass
                .get_raw()
                .renderpass,
            self.frame_buffers[present_index].get_raw().framebuffer,
            self.command_buffers[present_index],
            self.pipeline_info.pipeline.get_raw().pipelines[0],
            self.pipeline_info.pipeline_layout.get_raw().pipeline_layout,
            descriptor_set,
        )
    }
}

impl Drop for VkBloomExtractRenderPass {
    fn drop(&mut self) {
        log::trace!("destroying VkSpriteRenderPass");

        unsafe {
            let device = self.device_context.device();
            device.destroy_command_pool(self.command_pool, None);
        }

        log::trace!("destroyed VkSpriteRenderPass");
    }
}
