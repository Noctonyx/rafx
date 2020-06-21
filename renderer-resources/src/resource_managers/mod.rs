// use crate::upload::{
//     UploadQueue, ImageUploadOpResult, BufferUploadOpResult, PendingImageUpload, UploadOp,
// };
use crossbeam_channel::{Sender, Receiver};
use renderer_shell_vulkan::{VkDeviceContext, VkImage, VkImageRaw, VkBuffer, VkBufferRaw};
use ash::prelude::*;
use ash::vk;
use renderer_assets::assets::image::ImageAsset;
use renderer_assets::assets::shader::ShaderAsset;
use renderer_assets::assets::pipeline::{PipelineAsset, MaterialAsset, MaterialInstanceAsset, MaterialPass, MaterialInstanceSlotAssignment, RenderpassAsset};
use renderer_assets::vk_description::SwapchainSurfaceInfo;
use atelier_assets::loader::handle::Handle;
use std::mem::ManuallyDrop;
use fnv::FnvHashMap;
use renderer_assets::vk_description as dsc;
use atelier_assets::loader::AssetLoadOp;
use atelier_assets::loader::LoadHandle;
use atelier_assets::loader::handle::AssetHandle;
use std::sync::Arc;

mod resource_arc;
use resource_arc::ResourceId;
pub use resource_arc::ResourceArc;
use resource_arc::WeakResourceArc;

mod resource_lookup;
use resource_lookup::ResourceHash;
use resource_lookup::ResourceLookupSet;
use resource_lookup::DescriptorSetLayoutResource;

mod dyn_resource_allocator;
pub use dyn_resource_allocator::DynResourceAllocatorSet;

mod load_queue;
use load_queue::LoadQueues;
use load_queue::GenericLoadHandler;
use load_queue::LoadRequest;
use load_queue::LoadQueueSet;

mod swapchain_management;
use swapchain_management::ActiveSwapchainSurfaceInfoSet;

mod asset_lookup;
use asset_lookup::LoadedImage;
use asset_lookup::LoadedShaderModule;
use asset_lookup::LoadedMaterialInstance;
use asset_lookup::LoadedMaterial;
use asset_lookup::LoadedMaterialPass;
use asset_lookup::LoadedGraphicsPipeline;
use asset_lookup::LoadedAssetLookupSet;
use asset_lookup::AssetLookup;
use asset_lookup::SlotLocation;
use asset_lookup::LoadedAssetMetrics;
use asset_lookup::SlotNameLookup;

mod descriptor_sets;
use descriptor_sets::DescriptorSetAllocator;
pub use descriptor_sets::DescriptorSetAllocatorRef;
pub use descriptor_sets::DescriptorSetAllocatorProvider;
pub use descriptor_sets::DescriptorSetArc;
use descriptor_sets::DescriptorSetPoolMetrics;
use descriptor_sets::DescriptorSetAllocatorMetrics;
pub use descriptor_sets::DynDescriptorSet;
pub use descriptor_sets::DynPassMaterialInstance;
pub use descriptor_sets::DynMaterialInstance;
use descriptor_sets::DescriptorSetWriteSet;

mod upload;
use upload::UploadQueue;
use upload::ImageUploadOpResult;
use upload::BufferUploadOpResult;
use upload::PendingImageUpload;
use upload::PendingBufferUpload;
use upload::UploadManager;
use crate::resource_managers::resource_lookup::{
    PipelineLayoutResource, PipelineResource,
};

pub use resource_lookup::ImageViewResource;
use renderer_assets::assets::gltf::MeshAsset;
use renderer_assets::assets::buffer::BufferAsset;
use crate::resource_managers::asset_lookup::{LoadedBuffer, LoadedMesh, LoadedRenderpass, LoadedMeshPart};
use crate::resource_managers::dyn_resource_allocator::DynResourceAllocatorManagerSet;
use crate::resource_managers::descriptor_sets::{DescriptorSetAllocatorManager};

//TODO: Support descriptors that can be different per-view
//TODO: Support dynamic descriptors tied to command buffers?
//TODO: Support data inheritance for descriptors

// Information about a pipeline for a particular swapchain, resources may or may not be shared
// across swapchains depending on if they are the same size/format
pub struct PipelineSwapchainInfo {
    pub descriptor_set_layouts: Vec<ResourceArc<DescriptorSetLayoutResource>>,
    pub pipeline_layout: ResourceArc<PipelineLayoutResource>,
    pub renderpass: ResourceArc<vk::RenderPass>,
    pub pipeline: ResourceArc<PipelineResource>,
}

// Information about a single loaded image
pub struct ImageInfo {
    pub image: ResourceArc<VkImageRaw>,
    pub image_view: ResourceArc<ImageViewResource>,
}

// Information about a single descriptor set
pub struct DescriptorSetInfo {
    pub descriptor_set_layout_def: dsc::DescriptorSetLayout,
    pub descriptor_set_layout: ResourceArc<DescriptorSetLayoutResource>,
}

pub struct MeshPartInfo {
    //pub draw_info: LoadedMeshPart,
    pub material_instance: Vec<Vec<DescriptorSetArc>>
}

pub struct MeshInfo {
    pub vertex_buffer: ResourceArc<VkBufferRaw>,
    pub index_buffer: ResourceArc<VkBufferRaw>,
    pub mesh_asset: MeshAsset,
    pub mesh_parts: Vec<MeshPartInfo>
}

// Information about a descriptor set for a particular frame. Descriptor sets may be updated
// every frame and we rotate through them, so this information must not be persisted across frames
pub struct MaterialInstanceInfo {
    pub descriptor_sets: Vec<Vec<DescriptorSetArc>>,
}

#[derive(Debug)]
pub struct ResourceManagerMetrics {
    pub dyn_resource_metrics: dyn_resource_allocator::ResourceMetrics,
    pub resource_metrics: resource_lookup::ResourceMetrics,
    pub loaded_asset_metrics: LoadedAssetMetrics,
    pub resource_descriptor_sets_metrics: DescriptorSetAllocatorMetrics,
}

pub struct ResourceManager {
    device_context: VkDeviceContext,

    dyn_resources: DynResourceAllocatorManagerSet,
    resources: ResourceLookupSet,
    loaded_assets: LoadedAssetLookupSet,
    load_queues: LoadQueueSet,
    swapchain_surfaces: ActiveSwapchainSurfaceInfoSet,
    resource_descriptor_sets: DescriptorSetAllocator,
    descriptor_set_allocator: DescriptorSetAllocatorManager,
    upload_manager: UploadManager,
}

impl ResourceManager {
    pub fn new(device_context: &VkDeviceContext) -> Self {
        ResourceManager {
            device_context: device_context.clone(),
            dyn_resources: DynResourceAllocatorManagerSet::new(
                device_context,
                renderer_shell_vulkan::MAX_FRAMES_IN_FLIGHT as u32,
            ),
            resources: ResourceLookupSet::new(
                device_context,
                renderer_shell_vulkan::MAX_FRAMES_IN_FLIGHT as u32,
            ),
            loaded_assets: Default::default(),
            load_queues: Default::default(),
            swapchain_surfaces: Default::default(),
            resource_descriptor_sets: DescriptorSetAllocator::new(device_context),
            descriptor_set_allocator: DescriptorSetAllocatorManager::new(device_context),
            upload_manager: UploadManager::new(device_context),
        }
    }

    pub fn create_shader_load_handler(&self) -> GenericLoadHandler<ShaderAsset> {
        self.load_queues.shader_modules.create_load_handler()
    }

    pub fn create_pipeline_load_handler(&self) -> GenericLoadHandler<PipelineAsset> {
        self.load_queues.graphics_pipelines.create_load_handler()
    }

    pub fn create_renderpass_load_handler(&self) -> GenericLoadHandler<RenderpassAsset> {
        self.load_queues.renderpasses.create_load_handler()
    }

    pub fn create_material_load_handler(&self) -> GenericLoadHandler<MaterialAsset> {
        self.load_queues.materials.create_load_handler()
    }

    pub fn create_material_instance_load_handler(
        &self
    ) -> GenericLoadHandler<MaterialInstanceAsset> {
        self.load_queues.material_instances.create_load_handler()
    }

    pub fn create_image_load_handler(&self) -> GenericLoadHandler<ImageAsset> {
        self.load_queues.images.create_load_handler()
    }

    pub fn create_buffer_load_handler(&self) -> GenericLoadHandler<BufferAsset> {
        self.load_queues.buffers.create_load_handler()
    }

    pub fn create_mesh_load_handler(&self) -> GenericLoadHandler<MeshAsset> {
        self.load_queues.meshes.create_load_handler()
    }

    pub fn create_dyn_resource_allocator_set(&self) -> DynResourceAllocatorSet {
        self.dyn_resources.create_allocator_set()
    }

    pub fn create_descriptor_set_allocator(&self) -> DescriptorSetAllocatorRef {
        self.descriptor_set_allocator.get_allocator()
    }

    pub fn create_descriptor_set_allocator_provider(&self) -> DescriptorSetAllocatorProvider {
        self.descriptor_set_allocator.create_allocator_provider()
    }

    pub fn get_image_info(
        &self,
        handle: &Handle<ImageAsset>,
    ) -> Option<ImageInfo> {
        self
            .loaded_assets
            .images
            .get_committed(handle.load_handle())
            .map(|loaded_image| {
                ImageInfo {
                    image: loaded_image.image.clone(),
                    image_view: loaded_image.image_view.clone(),
                }
            })
    }

    pub fn get_descriptor_set_info(
        &self,
        handle: &Handle<MaterialAsset>,
        pass_index: usize,
        layout_index: usize,
    ) -> DescriptorSetInfo {
        let resource = self
            .loaded_assets
            .materials
            .get_committed(handle.load_handle())
            .unwrap();

        let descriptor_set_layout_def = resource.passes[pass_index]
            .pipeline_create_data
            .pipeline_layout_def
            .descriptor_set_layouts[layout_index]
            .clone();
        let descriptor_set_layout =
            resource.passes[pass_index].descriptor_set_layouts[layout_index].clone();

        DescriptorSetInfo {
            descriptor_set_layout_def,
            descriptor_set_layout,
        }
    }

    pub fn get_pipeline_info(
        &self,
        handle: &Handle<MaterialAsset>,
        swapchain: &SwapchainSurfaceInfo,
        pass_index: usize,
    ) -> PipelineSwapchainInfo {
        let resource = self
            .loaded_assets
            .materials
            .get_committed(handle.load_handle())
            .unwrap();

        let swapchain_index = self
            .swapchain_surfaces
            .ref_counts
            .get(swapchain)
            .unwrap()
            .index;

        PipelineSwapchainInfo {
            descriptor_set_layouts: resource.passes[pass_index].descriptor_set_layouts.clone(),
            pipeline_layout: resource.passes[pass_index].pipeline_layout.clone(),
            renderpass: resource.passes[pass_index].render_passes[swapchain_index].clone(),
            pipeline: resource.passes[pass_index].pipelines[swapchain_index].clone(),
        }
    }

    pub fn get_mesh_info(
        &self,
        handle: &Handle<MeshAsset>
    ) -> Option<MeshInfo> {
        self.loaded_assets.meshes.get_committed(handle.load_handle()).map(|loaded_mesh| {
            let mesh_parts : Vec<_> = loaded_mesh.mesh_parts.iter().map(|x| {
                MeshPartInfo {
                    material_instance: x.material_instance.clone()
                }
            }).collect();

            MeshInfo {
                vertex_buffer: loaded_mesh.vertex_buffer.clone(),
                index_buffer: loaded_mesh.index_buffer.clone(),
                mesh_asset: loaded_mesh.asset.clone(),
                mesh_parts
            }
        })
    }

    pub fn get_material_instance_info(
        &self,
        handle: &Handle<MaterialInstanceAsset>,
    ) -> MaterialInstanceInfo {
        // Get the material instance
        let resource = self
            .loaded_assets
            .material_instances
            .get_committed(handle.load_handle())
            .unwrap();

        MaterialInstanceInfo {
            descriptor_sets: resource.material_descriptor_sets.clone()
        }
    }

    pub fn add_swapchain(
        &mut self,
        swapchain_surface_info: &dsc::SwapchainSurfaceInfo,
    ) -> VkResult<()> {
        log::info!("add_swapchain {:?}", swapchain_surface_info);
        self.swapchain_surfaces.add(
            &swapchain_surface_info,
            &mut self.loaded_assets,
            &mut self.resources,
        )
    }

    pub fn remove_swapchain(
        &mut self,
        swapchain_surface_info: &dsc::SwapchainSurfaceInfo,
    ) {
        log::info!("remove_swapchain {:?}", swapchain_surface_info);
        self.swapchain_surfaces
            .remove(swapchain_surface_info, &mut self.loaded_assets);
    }

    // Call whenever you want to handle assets loading/unloading
    pub fn update_resources(&mut self) -> VkResult<()> {
        self.process_shader_load_requests();
        self.process_pipeline_load_requests();
        self.process_renderpass_load_requests();
        self.process_material_load_requests();
        self.process_material_instance_load_requests();
        self.process_image_load_requests();
        self.process_buffer_load_requests();
        self.process_mesh_load_requests();

        self.upload_manager.update()?;

        //self.dump_stats();

        Ok(())
    }

    // Call just before rendering
    pub fn on_begin_frame(&mut self) -> VkResult<()> {
        self.resource_descriptor_sets.flush_changes();
        Ok(())
    }

    pub fn on_frame_complete(&mut self) -> VkResult<()> {
        self.resources.on_frame_complete();
        self.dyn_resources.on_frame_complete();
        self.resource_descriptor_sets.on_frame_complete();
        self.descriptor_set_allocator.on_frame_complete();
        Ok(())
    }

    pub fn metrics(&self) -> ResourceManagerMetrics {
        let dyn_resource_metrics = self.dyn_resources.metrics();
        let resource_metrics = self.resources.metrics();
        let loaded_asset_metrics = self.loaded_assets.metrics();
        let resource_descriptor_sets_metrics = self.resource_descriptor_sets.metrics();

        ResourceManagerMetrics {
            dyn_resource_metrics,
            resource_metrics,
            loaded_asset_metrics,
            resource_descriptor_sets_metrics,
        }
    }

    fn process_shader_load_requests(&mut self) {
        for request in self.load_queues.shader_modules.take_load_requests() {
            log::trace!("Create shader module {:?}", request.load_handle);
            let loaded_asset = self.load_shader_module(&request.asset);
            Self::handle_load_result(
                request.load_op,
                loaded_asset,
                &mut self.loaded_assets.shader_modules,
            );
        }

        Self::handle_commit_requests(
            &mut self.load_queues.shader_modules,
            &mut self.loaded_assets.shader_modules,
        );
        Self::handle_free_requests(
            &mut self.load_queues.shader_modules,
            &mut self.loaded_assets.shader_modules,
        );
    }

    fn process_pipeline_load_requests(&mut self) {
        for request in self.load_queues.graphics_pipelines.take_load_requests() {
            log::trace!("Create pipeline {:?}", request.load_handle);
            let loaded_asset = self.load_graphics_pipeline(&request.asset);
            Self::handle_load_result(
                request.load_op,
                loaded_asset,
                &mut self.loaded_assets.graphics_pipelines,
            );
        }

        Self::handle_commit_requests(
            &mut self.load_queues.graphics_pipelines,
            &mut self.loaded_assets.graphics_pipelines,
        );
        Self::handle_free_requests(
            &mut self.load_queues.graphics_pipelines,
            &mut self.loaded_assets.graphics_pipelines,
        );
    }

    fn process_renderpass_load_requests(&mut self) {
        for request in self.load_queues.renderpasses.take_load_requests() {
            log::trace!("Create renderpass {:?}", request.load_handle);
            let loaded_asset = self.load_renderpass(&request.asset);
            Self::handle_load_result(
                request.load_op,
                loaded_asset,
                &mut self.loaded_assets.renderpasses,
            );
        }

        Self::handle_commit_requests(
            &mut self.load_queues.renderpasses,
            &mut self.loaded_assets.renderpasses,
        );
        Self::handle_free_requests(
            &mut self.load_queues.renderpasses,
            &mut self.loaded_assets.renderpasses,
        );
    }

    fn process_material_load_requests(&mut self) {
        for request in self.load_queues.materials.take_load_requests() {
            log::trace!("Create material {:?}", request.load_handle);
            let loaded_asset = self.load_material(&request.asset);
            Self::handle_load_result(
                request.load_op,
                loaded_asset,
                &mut self.loaded_assets.materials,
            );
        }

        Self::handle_commit_requests(
            &mut self.load_queues.materials,
            &mut self.loaded_assets.materials,
        );
        Self::handle_free_requests(
            &mut self.load_queues.materials,
            &mut self.loaded_assets.materials,
        );
    }

    fn process_material_instance_load_requests(&mut self) {
        for request in self.load_queues.material_instances.take_load_requests() {
            log::trace!("Create material instance {:?}", request.load_handle);
            let loaded_asset = self.load_material_instance(&request.asset);
            Self::handle_load_result(
                request.load_op,
                loaded_asset,
                &mut self.loaded_assets.material_instances,
            );
        }

        Self::handle_commit_requests(
            &mut self.load_queues.material_instances,
            &mut self.loaded_assets.material_instances,
        );
        Self::handle_free_requests(
            &mut self.load_queues.material_instances,
            &mut self.loaded_assets.material_instances,
        );
    }

    fn process_mesh_load_requests(&mut self) {
        for request in self.load_queues.meshes.take_load_requests() {
            log::trace!("Create mesh {:?}", request.load_handle);
            let loaded_asset = self.load_mesh(&request.asset);
            Self::handle_load_result(
                request.load_op,
                loaded_asset,
                &mut self.loaded_assets.meshes,
            );
        }

        Self::handle_commit_requests(
            &mut self.load_queues.meshes,
            &mut self.loaded_assets.meshes,
        );
        Self::handle_free_requests(
            &mut self.load_queues.meshes,
            &mut self.loaded_assets.meshes,
        );
    }

    fn process_image_load_requests(&mut self) {
        for request in self.load_queues.images.take_load_requests() {
            //TODO: Route the request directly to the upload queue
            log::trace!("Uploading image {:?}", request.load_handle);
            self.upload_manager.upload_image(request);
        }

        let results: Vec<_> = self
            .upload_manager
            .image_upload_result_rx
            .try_iter()
            .collect();
        for result in results {
            match result {
                ImageUploadOpResult::UploadComplete(load_op, image) => {
                    let loaded_asset = self.finish_load_image(load_op.load_handle(), image);
                    Self::handle_load_result(load_op, loaded_asset, &mut self.loaded_assets.images);
                }
                ImageUploadOpResult::UploadError(load_handle) => {
                    // Don't need to do anything - the uploaded should have triggered an error on the load_op
                }
                ImageUploadOpResult::UploadDrop(load_handle) => {
                    // Don't need to do anything - the uploaded should have triggered an error on the load_op
                }
            }
        }

        Self::handle_commit_requests(&mut self.load_queues.images, &mut self.loaded_assets.images);
        Self::handle_free_requests(&mut self.load_queues.images, &mut self.loaded_assets.images);
    }

    fn process_buffer_load_requests(&mut self) {
        for request in self.load_queues.buffers.take_load_requests() {
            //TODO: Route the request directly to the upload queue
            log::trace!("Uploading buffer {:?}", request.load_handle);
            self.upload_manager.upload_buffer(request);
        }

        let results: Vec<_> = self
            .upload_manager
            .buffer_upload_result_rx
            .try_iter()
            .collect();
        for result in results {
            match result {
                BufferUploadOpResult::UploadComplete(load_op, buffer) => {
                    let loaded_asset = self.finish_load_buffer(load_op.load_handle(), buffer);
                    Self::handle_load_result(load_op, loaded_asset, &mut self.loaded_assets.buffers);
                }
                BufferUploadOpResult::UploadError(load_handle) => {
                    // Don't need to do anything - the uploaded should have triggered an error on the load_op
                }
                BufferUploadOpResult::UploadDrop(load_handle) => {
                    // Don't need to do anything - the uploaded should have triggered an error on the load_op
                }
            }
        }

        Self::handle_commit_requests(&mut self.load_queues.buffers, &mut self.loaded_assets.buffers);
        Self::handle_free_requests(&mut self.load_queues.buffers, &mut self.loaded_assets.buffers);
    }

    fn handle_load_result<LoadedAssetT>(
        load_op: AssetLoadOp,
        loaded_asset: VkResult<LoadedAssetT>,
        asset_lookup: &mut AssetLookup<LoadedAssetT>,
    ) {
        match loaded_asset {
            Ok(loaded_asset) => {
                asset_lookup.set_uncommitted(load_op.load_handle(), loaded_asset);
                load_op.complete()
            }
            Err(err) => {
                load_op.error(err);
            }
        }
    }

    fn handle_commit_requests<AssetT, LoadedAssetT>(
        load_queues: &mut LoadQueues<AssetT>,
        asset_lookup: &mut AssetLookup<LoadedAssetT>,
    ) {
        for request in load_queues.take_commit_requests() {
            log::info!("commit asset {:?} {}", request.load_handle, core::any::type_name::<AssetT>());
            asset_lookup.commit(request.load_handle);
        }
    }

    fn handle_free_requests<AssetT, LoadedAssetT>(
        load_queues: &mut LoadQueues<AssetT>,
        asset_lookup: &mut AssetLookup<LoadedAssetT>,
    ) {
        for request in load_queues.take_commit_requests() {
            asset_lookup.commit(request.load_handle);
        }
    }

    fn finish_load_image(
        &mut self,
        image_load_handle: LoadHandle,
        image: VkImage,
    ) -> VkResult<LoadedImage> {
        let format = image.format.into();
        let mip_level_count = image.mip_level_count;

        let (image_key, image_arc) = self.resources.insert_image(ManuallyDrop::new(image));

        let image_view_meta = dsc::ImageViewMeta {
            view_type: dsc::ImageViewType::Type2D,
            format,
            subresource_range: dsc::ImageSubresourceRange {
                aspect_mask: dsc::ImageAspectFlags::Color,
                base_mip_level: 0,
                level_count: mip_level_count,
                base_array_layer: 0,
                layer_count: 1,
            },
            components: dsc::ComponentMapping {
                r: dsc::ComponentSwizzle::Identity,
                g: dsc::ComponentSwizzle::Identity,
                b: dsc::ComponentSwizzle::Identity,
                a: dsc::ComponentSwizzle::Identity,
            },
        };

        let image_view = self
            .resources
            .get_or_create_image_view(image_key, &image_view_meta)?;

        Ok(LoadedImage {
            image_key,
            image: image_arc,
            image_view,
        })
    }

    fn finish_load_buffer(
        &mut self,
        buffer_load_handle: LoadHandle,
        buffer: VkBuffer,
    ) -> VkResult<LoadedBuffer> {
        let (buffer_key, buffer) = self.resources.insert_buffer(ManuallyDrop::new(buffer));

        Ok(LoadedBuffer {
            buffer_key,
            buffer,
        })
    }

    fn load_shader_module(
        &mut self,
        shader_module: &ShaderAsset,
    ) -> VkResult<LoadedShaderModule> {
        let shader_module = self
            .resources
            .get_or_create_shader_module(&shader_module.shader)?;
        Ok(LoadedShaderModule { shader_module })
    }

    fn load_graphics_pipeline(
        &mut self,
        pipeline_asset: &PipelineAsset,
    ) -> VkResult<LoadedGraphicsPipeline> {
        Ok(LoadedGraphicsPipeline {
            pipeline_asset: pipeline_asset.clone(),
        })
    }

    fn load_renderpass(
        &mut self,
        renderpass_asset: &RenderpassAsset,
    ) -> VkResult<LoadedRenderpass> {
        Ok(LoadedRenderpass {
            renderpass_asset: renderpass_asset.clone(),
        })
    }

    fn load_material(
        &mut self,
        material_asset: &MaterialAsset,
    ) -> VkResult<LoadedMaterial> {
        let mut passes = Vec::with_capacity(material_asset.passes.len());

        for pass in &material_asset.passes {
            let loaded_pipeline_asset = self
                .loaded_assets
                .graphics_pipelines
                .get_latest(pass.pipeline.load_handle())
                .unwrap();
            let pipeline_asset = loaded_pipeline_asset.pipeline_asset.clone();

            let loaded_renderpass_asset = self
                .loaded_assets
                .renderpasses
                .get_latest(pass.renderpass.load_handle())
                .unwrap();
            let renderpass_asset = loaded_renderpass_asset.renderpass_asset.clone();

            let shader_hashes : Vec<_> = pass.shaders.iter().map(|shader| {
                let shader_module = self.loaded_assets.shader_modules.get_latest(shader.shader_module.load_handle()).unwrap();
                shader_module.shader_module.get_hash().into()
            }).collect();

            let swapchain_surface_infos = self.swapchain_surfaces.unique_swapchain_infos().clone();
            let pipeline_create_data = PipelineCreateData::new(
                self,
                pipeline_asset,
                renderpass_asset,
                pass,
                shader_hashes
            )?;

            // Will contain the vulkan resources being created per swapchain
            let mut render_passes = Vec::with_capacity(swapchain_surface_infos.len());
            let mut pipelines = Vec::with_capacity(swapchain_surface_infos.len());

            // Create the pipeline objects
            for swapchain_surface_info in swapchain_surface_infos {
                let pipeline = self.resources.get_or_create_graphics_pipeline(
                    &pipeline_create_data,
                    &swapchain_surface_info,
                )?;
                render_passes.push(pipeline.get_raw().renderpass);
                pipelines.push(pipeline);
            }

            // Create a lookup of the slot names
            let mut pass_slot_name_lookup: SlotNameLookup = Default::default();
            for (layout_index, layout) in pass
                .shader_interface
                .descriptor_set_layouts
                .iter()
                .enumerate()
            {
                for (binding_index, binding) in
                    layout.descriptor_set_layout_bindings.iter().enumerate()
                {
                    pass_slot_name_lookup
                        .entry(binding.slot_name.clone())
                        .or_default()
                        .push(SlotLocation {
                            layout_index: layout_index as u32,
                            binding_index: binding_index as u32,
                            //array_index: 0
                        });
                }
            }

            passes.push(LoadedMaterialPass {
                descriptor_set_layouts: pipeline_create_data.descriptor_set_layout_arcs.clone(),
                pipeline_layout: pipeline_create_data.pipeline_layout.clone(),
                shader_modules: pipeline_create_data.shader_module_arcs.clone(),
                render_passes,
                pipelines,
                pipeline_create_data,
                shader_interface: pass.shader_interface.clone(),
                pass_slot_name_lookup: Arc::new(pass_slot_name_lookup),
            })
        }

        Ok(LoadedMaterial { passes })
    }

    fn load_material_instance(
        &mut self,
        material_instance_asset: &MaterialInstanceAsset,
    ) -> VkResult<LoadedMaterialInstance> {
        // Find the material we will bind over, we need the metadata from it
        let material_asset = self
            .loaded_assets
            .materials
            .get_latest(material_instance_asset.material.load_handle())
            .unwrap();

        let mut material_instance_descriptor_set_writes = Vec::with_capacity(material_asset.passes.len());

        log::trace!("load_material_instance slot assignments\n{:#?}", material_instance_asset.slot_assignments);

        // This will be references to descriptor sets. Indexed by pass, and then by set within the pass.
        let mut material_descriptor_sets = Vec::with_capacity(material_asset.passes.len());
        for pass in &material_asset.passes {
            let pass_descriptor_set_writes =
                descriptor_sets::create_write_sets_for_material_instance_pass(
                    pass,
                    &material_instance_asset.slot_assignments,
                    &self.loaded_assets,
                    &mut self.resources,
                )?;

            log::trace!("load_material_instance descriptor set write\n{:#?}", pass_descriptor_set_writes);

            // Save the
            material_instance_descriptor_set_writes.push(pass_descriptor_set_writes.clone());

            // This will contain the descriptor sets created for this pass, one for each set within the pass
            let mut pass_descriptor_sets = Vec::with_capacity(pass_descriptor_set_writes.len());

            //
            // Register the writes into the correct descriptor set pools
            //
            //let layouts = pass.pipeline_create_data.pipeline_layout.iter().zip(&pass.pipeline_create_data.pipeline_layout_def);
            for (layout_index, layout_writes) in pass_descriptor_set_writes.into_iter().enumerate()
            {
                let descriptor_set = self.resource_descriptor_sets.create_descriptor_set(
                    &pass.pipeline_create_data.descriptor_set_layout_arcs[layout_index],
                    layout_writes,
                )?;

                pass_descriptor_sets.push(descriptor_set);
            }

            material_descriptor_sets.push(pass_descriptor_sets);
        }

        log::trace!("Loaded material\n{:#?}", material_descriptor_sets);

        Ok(LoadedMaterialInstance {
            material: material_instance_asset.material.clone(),
            material_descriptor_sets,
            slot_assignments: material_instance_asset.slot_assignments.clone(),
            descriptor_set_writes: material_instance_descriptor_set_writes,
        })
    }

    fn load_mesh(
        &mut self,
        mesh_asset: &MeshAsset,
    ) -> VkResult<LoadedMesh> {

        let vertex_buffer = self.loaded_assets.buffers.get_latest(mesh_asset.vertex_buffer.load_handle()).unwrap().buffer.clone();
        let index_buffer = self.loaded_assets.buffers.get_latest(mesh_asset.index_buffer.load_handle()).unwrap().buffer.clone();

        let mut mesh_parts = Vec::with_capacity(mesh_asset.mesh_parts.len());

        for part in &mesh_asset.mesh_parts {
            let material_instance_info = self.get_material_instance_info(&part.material_instance);
            mesh_parts.push(LoadedMeshPart {
                material_instance: material_instance_info.descriptor_sets.clone()
            })
        }

        Ok(LoadedMesh {
            vertex_buffer,
            index_buffer,
            asset: mesh_asset.clone(),
            mesh_parts
        })
    }

    pub fn create_dyn_descriptor_set_uninitialized(
        &self,
        descriptor_set_allocator: &mut DescriptorSetAllocator,
        descriptor_set_layout: &ResourceArc<DescriptorSetLayoutResource>,
    ) -> VkResult<DynDescriptorSet> {
        descriptor_set_allocator
            .create_dyn_descriptor_set_uninitialized(descriptor_set_layout)
    }

    pub fn create_dyn_pass_material_instance_uninitialized(
        &self,
        descriptor_set_allocator: &mut DescriptorSetAllocator,
        material: Handle<MaterialAsset>,
        pass_index: u32,
    ) -> VkResult<DynPassMaterialInstance> {
        let material_asset = self
            .loaded_assets
            .materials
            .get_latest(material.load_handle())
            .unwrap();

        descriptor_set_allocator
            .create_dyn_pass_material_instance_uninitialized(
                &material_asset.passes[pass_index as usize],
                &self.loaded_assets,
            )
    }

    pub fn create_dyn_pass_material_instance_from_asset(
        &mut self,
        descriptor_set_allocator: &mut DescriptorSetAllocator,
        material_instance: Handle<MaterialInstanceAsset>,
        pass_index: u32,
    ) -> VkResult<DynPassMaterialInstance> {
        let material_instance_asset = self
            .loaded_assets
            .material_instances
            .get_committed(material_instance.load_handle())
            .unwrap();

        let material_asset = self
            .loaded_assets
            .materials
            .get_latest(material_instance_asset.material.load_handle())
            .unwrap();

        descriptor_set_allocator
            .create_dyn_pass_material_instance_from_asset(
                &material_asset.passes[pass_index as usize],
                material_instance_asset.descriptor_set_writes[pass_index as usize].clone(),
            )
    }

    pub fn create_dyn_material_instance_uninitialized(
        &self,
        descriptor_set_allocator: &mut DescriptorSetAllocator,
        material: Handle<MaterialAsset>,
    ) -> VkResult<DynMaterialInstance> {
        let material_asset = self
            .loaded_assets
            .materials
            .get_latest(material.load_handle())
            .unwrap();

        descriptor_set_allocator
            .create_dyn_material_instance_uninitialized(material_asset, &self.loaded_assets)
    }

    pub fn create_dyn_material_instance_from_asset(
        &self, // mut required because the asset may describe a sampler that needs to be created
        descriptor_set_allocator: &mut DescriptorSetAllocator,
        material_instance: Handle<MaterialInstanceAsset>,
    ) -> VkResult<DynMaterialInstance> {
        let material_instance_asset = self
            .loaded_assets
            .material_instances
            .get_committed(material_instance.load_handle())
            .unwrap();

        let material_asset = self
            .loaded_assets
            .materials
            .get_latest(material_instance_asset.material.load_handle())
            .unwrap();

        descriptor_set_allocator
            .create_dyn_material_instance_from_asset(
                material_asset,
                material_instance_asset,
            )
    }
}

impl Drop for ResourceManager {
    fn drop(&mut self) {
        unsafe {
            log::info!("Cleaning up resource manager");
            log::trace!("Resource Manager Metrics:\n{:#?}", self.metrics());

            // Wipe out any loaded assets. This will potentially drop ref counts on resources
            self.loaded_assets.destroy();

            // Drop all descriptors. These bind to raw resources, so we need to drop them before
            // dropping resources
            self.resource_descriptor_sets.destroy();
            self.descriptor_set_allocator.destroy();

            // Now drop all resources with a zero ref count and warn for any resources that remain
            self.resources.destroy();
            self.dyn_resources.destroy();

            log::info!("Dropping resource manager");
            log::trace!("Resource Manager Metrics:\n{:#?}", self.metrics());
        }
    }
}

// We have to create pipelines when pipeline assets load and when swapchains are added/removed.
// Gathering all the info to hash and create a pipeline is a bit involved so we share the code
// here
#[derive(Clone)]
pub struct PipelineCreateData {
    // We store the shader module hash rather than the shader module itself because it's a large
    // binary blob. We don't use it to create or even look up the shader, just so that the hash of
    // the pipeline we create doesn't conflict with the same pipeline using reloaded (different) shaders
    shader_module_metas: Vec<dsc::ShaderModuleMeta>,
    shader_module_hashes: Vec<ResourceHash>,
    shader_module_arcs: Vec<ResourceArc<vk::ShaderModule>>,
    shader_module_vk_objs: Vec<vk::ShaderModule>,

    descriptor_set_layout_arcs: Vec<ResourceArc<DescriptorSetLayoutResource>>,

    fixed_function_state: dsc::FixedFunctionState,

    pipeline_layout_def: dsc::PipelineLayout,
    pipeline_layout: ResourceArc<PipelineLayoutResource>,

    renderpass: dsc::RenderPass,
}

impl PipelineCreateData {
    pub fn new(
        resource_manager: &mut ResourceManager,
        pipeline_asset: PipelineAsset,
        renderpass_asset: RenderpassAsset,
        material_pass: &MaterialPass,
        shader_module_hashes: Vec<ResourceHash>,
    ) -> VkResult<Self> {
        //
        // Shader module metadata (required to create the pipeline key)
        //
        let mut shader_module_metas = Vec::with_capacity(material_pass.shaders.len());
        for stage in &material_pass.shaders {
            let shader_module_meta = dsc::ShaderModuleMeta {
                stage: stage.stage,
                entry_name: stage.entry_name.clone(),
            };
            shader_module_metas.push(shader_module_meta);
        }

        //
        // Actual shader module resources (to create the pipeline)
        //
        let mut shader_module_arcs = Vec::with_capacity(material_pass.shaders.len());
        let mut shader_module_vk_objs = Vec::with_capacity(material_pass.shaders.len());
        for stage in &material_pass.shaders {
            let shader_module = resource_manager
                .loaded_assets
                .shader_modules
                .get_latest(stage.shader_module.load_handle())
                .unwrap();
            shader_module_arcs.push(shader_module.shader_module.clone());
            shader_module_vk_objs.push(shader_module.shader_module.get_raw());
        }

        //
        // Descriptor set layout
        //
        let mut descriptor_set_layout_arcs =
            Vec::with_capacity(material_pass.shader_interface.descriptor_set_layouts.len());
        let mut descriptor_set_layout_defs =
            Vec::with_capacity(material_pass.shader_interface.descriptor_set_layouts.len());
        for descriptor_set_layout_def in &material_pass.shader_interface.descriptor_set_layouts {
            let descriptor_set_layout_def = descriptor_set_layout_def.into();
            let descriptor_set_layout = resource_manager
                .resources
                .get_or_create_descriptor_set_layout(&descriptor_set_layout_def)?;
            descriptor_set_layout_arcs.push(descriptor_set_layout);
            descriptor_set_layout_defs.push(descriptor_set_layout_def);
        }

        //
        // Pipeline layout
        //
        let pipeline_layout_def = dsc::PipelineLayout {
            descriptor_set_layouts: descriptor_set_layout_defs,
            push_constant_ranges: material_pass.shader_interface.push_constant_ranges.clone(),
        };

        let pipeline_layout = resource_manager
            .resources
            .get_or_create_pipeline_layout(&pipeline_layout_def)?;

        let fixed_function_state = dsc::FixedFunctionState {
            vertex_input_state: material_pass.shader_interface.vertex_input_state.clone(),
            input_assembly_state: pipeline_asset.input_assembly_state,
            viewport_state: pipeline_asset.viewport_state,
            rasterization_state: pipeline_asset.rasterization_state,
            multisample_state: pipeline_asset.multisample_state,
            color_blend_state: pipeline_asset.color_blend_state,
            dynamic_state: pipeline_asset.dynamic_state,
            depth_stencil_state: pipeline_asset.depth_stencil_state,
        };

        Ok(PipelineCreateData {
            shader_module_metas,
            shader_module_hashes,
            shader_module_arcs,
            shader_module_vk_objs,
            descriptor_set_layout_arcs,
            fixed_function_state,
            pipeline_layout_def,
            pipeline_layout,
            renderpass: renderpass_asset.renderpass,
        })
    }
}