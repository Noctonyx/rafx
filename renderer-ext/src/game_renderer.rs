use crate::imgui_support::{VkImGuiRenderPassFontAtlas, VkImGuiRenderPass, ImguiRenderEventListener};
use renderer_shell_vulkan::{VkDevice, VkSwapchain, VkSurface, Window, VkTransferUpload, VkTransferUploadState, VkImage, VkDeviceContext, VkContextBuilder, VkCreateContextError, VkContext, VkSurfaceSwapchainLifetimeListener};
use ash::prelude::VkResult;
use crate::renderpass::{VkSpriteRenderPass, VkMeshRenderPass};
use std::mem::{ManuallyDrop, swap};
use crate::image_utils::{decode_texture, enqueue_load_images};
use ash::vk;
use crate::time::{ScopeTimer, TimeState};
use crossbeam_channel::Sender;
use std::ops::Deref;
// use crate::resource_managers::{
//     SpriteResourceManager, VkMeshResourceManager, ImageResourceManager,
//     MaterialResourceManager,
// };
//use crate::renderpass::VkMeshRenderPass;
use crate::pipeline_description::SwapchainSurfaceInfo;
use crate::pipeline::pipeline::{MaterialAsset, PipelineAsset, MaterialInstanceAsset};
use atelier_assets::loader::handle::Handle;
use crate::asset_resource::AssetResource;
//use crate::upload::UploadQueue;
//use crate::load_handlers::{ImageLoadHandler, MeshLoadHandler, SpriteLoadHandler, MaterialLoadHandler};
use crate::pipeline::shader::ShaderAsset;
use crate::pipeline::image::ImageAsset;
//use crate::pipeline::gltf::{GltfMaterialAsset, MeshAsset};
//use crate::pipeline::sprite::SpriteAsset;
use atelier_assets::core::asset_uuid;
use atelier_assets::loader::LoadStatus;
use atelier_assets::loader::handle::AssetHandle;
use atelier_assets::core as atelier_core;
use atelier_assets::core::AssetUuid;
use crate::resource_managers::{ResourceManager, DynDescriptorSet, DynMaterialInstance};
use crate::pipeline::gltf::{MeshAsset, GltfMaterialAsset, GltfMaterialData, GltfMaterialDataShaderParam};
use crate::pipeline::buffer::BufferAsset;

#[derive(Default, Copy, Clone)]
#[repr(C)]
struct PointLight {
    position_world: glam::Vec3, // +0
    position_view: glam::Vec3, // +16
    color: glam::Vec4, // +32
    range: f32, // +48
    intensity: f32, // +52
    //enabled: bool, //+56
} // 4*16 = 64 bytes

#[derive(Default, Copy, Clone)]
#[repr(C)]
struct DirectionalLight {
    direction_world: glam::Vec3, // +0
    direction_view: glam::Vec3, // +16
    color: glam::Vec4, // +32
    spotlight_half_angle: f32, // +48
    intensity: f32, // +52
    //enabled: bool, // +56
} // 4*16 = 64 bytes

#[derive(Default, Copy, Clone)]
#[repr(C)]
struct SpotLight {
    position_world: glam::Vec3, // +0
    direction_world: glam::Vec3, // +16
    position_view: glam::Vec3, // +32
    direction_view: glam::Vec3, // +48
    color: glam::Vec4, // +64
    spotlight_half_angle: f32, //+80
    range: f32, // +84
    intensity: f32, // +88
    //enabled: bool, // +92
    //pad: [bool; 3] //
} // 6*16 = 96 bytes

#[derive(Default, Copy, Clone)]
#[repr(C)]
struct PerFrameDataShaderParam {
    view: glam::Mat4, // +0
    proj: glam::Mat4, // +64
    point_light_count: u32, // +128
    directional_light_count: u32, // +132
    spot_light_count: u32, // +136
    point_lights: [PointLight; 16], // +144 (64*16 = 1024),
    directional_lights: [DirectionalLight; 16], // +1168 (64*16 = 1024),
    spot_lights: [SpotLight; 16], // +2192 (96*16 = 1536)
} // 3728 bytes

fn begin_load_asset<T>(
    asset_uuid: AssetUuid,
    asset_resource: &AssetResource,
) -> atelier_assets::loader::handle::Handle<T> {
    use atelier_assets::loader::Loader;
    let load_handle = asset_resource.loader().add_ref(asset_uuid);
    atelier_assets::loader::handle::Handle::<T>::new(asset_resource.tx().clone(), load_handle)
}

fn wait_for_asset_to_load<T>(
    device_context: &VkDeviceContext,
    asset_handle: &atelier_assets::loader::handle::Handle<T>,
    asset_resource: &mut AssetResource,
    renderer: &mut GameRenderer,
    asset_name: &str
) {
    loop {
        asset_resource.update();
        renderer.update_resources(device_context);
        match asset_handle.load_status(asset_resource.loader()) {
            LoadStatus::NotRequested => {
                unreachable!();
            }
            LoadStatus::Loading => {
                //log::info!("blocked waiting for asset to load {} {:?}", asset_name, asset_handle);
                std::thread::sleep(std::time::Duration::from_millis(100));
                // keep waiting
            }
            LoadStatus::Loaded => {
                break;
            }
            LoadStatus::Unloading => unreachable!(),
            LoadStatus::DoesNotExist => {
                println!("Essential asset not found");
            }
            LoadStatus::Error(err) => {
                println!("Error loading essential asset {:?}", err);
            }
        }
    }
}

pub struct GameRenderer {
    time_state: TimeState,
    imgui_event_listener: ImguiRenderEventListener,

    resource_manager: ResourceManager,

    sprite_material: Handle<MaterialAsset>,
    sprite_material_instance: Handle<MaterialInstanceAsset>,
    sprite_custom_material: Option<DynMaterialInstance>,
    mesh_custom_material: Option<DynMaterialInstance>,
    mesh: Handle<MeshAsset>,
    mesh_material: Handle<MaterialAsset>,
    mesh_material_instance: Handle<MaterialInstanceAsset>,

    mesh_renderpass: Option<VkMeshRenderPass>,
    sprite_renderpass: Option<VkSpriteRenderPass>,
    swapchain_surface_info: Option<SwapchainSurfaceInfo>,
}

impl GameRenderer {
    pub fn new(
        window: &dyn Window,
        device_context: &VkDeviceContext,
        imgui_font_atlas: VkImGuiRenderPassFontAtlas,
        time_state: &TimeState,
        asset_resource: &mut AssetResource,
    ) -> VkResult<Self> {
        let imgui_event_listener = ImguiRenderEventListener::new(imgui_font_atlas);

        let mut resource_manager = ResourceManager::new(device_context);

        asset_resource.add_storage_with_load_handler::<ShaderAsset, _>(Box::new(
            resource_manager.create_shader_load_handler(),
        ));
        asset_resource.add_storage_with_load_handler::<PipelineAsset, _>(Box::new(
            resource_manager.create_pipeline_load_handler(),
        ));
        asset_resource.add_storage_with_load_handler::<MaterialAsset, _>(Box::new(
            resource_manager.create_material_load_handler(),
        ));
        asset_resource.add_storage_with_load_handler::<MaterialInstanceAsset, _>(Box::new(
            resource_manager.create_material_instance_load_handler(),
        ));
        asset_resource.add_storage_with_load_handler::<ImageAsset, _>(Box::new(
            resource_manager.create_image_load_handler(),
        ));
        asset_resource.add_storage_with_load_handler::<BufferAsset, _>(Box::new(
            resource_manager.create_buffer_load_handler(),
        ));
        asset_resource.add_storage_with_load_handler::<MeshAsset, _>(Box::new(
            resource_manager.create_mesh_load_handler(),
        ));
        //asset_resource.add_storage::<BufferAsset>();

        asset_resource.add_storage::<GltfMaterialAsset>();
        //asset_resource.add_storage::<MeshAsset>();
        // asset_resource.add_storage::<SpriteAsset>();

        // asset_resource.add_storage_with_load_handler::<MeshAsset, _>(Box::new(
        //     resource_manager.create_mesh_load_handler(),
        // ));

        let sprite_material = begin_load_asset::<MaterialAsset>(
            asset_uuid!("f8c4897e-7c1d-4736-93b7-f2deda158ec7"),
            &asset_resource,
        );
        let sprite_material_instance = begin_load_asset::<MaterialInstanceAsset>(
            asset_uuid!("84d66f60-24b2-4eb2-b6ff-8dbc4d69e2c5"),
            &asset_resource,
        );
        let override_image = begin_load_asset::<ImageAsset>(
            asset_uuid!("7c42f3bc-e96b-49f6-961b-5bfc799dee50"),
            &asset_resource,
        );

        let mesh_material = begin_load_asset::<MaterialAsset>(
            asset_uuid!("267e0388-2611-441c-9c78-2d39d1bd3cf1"),
            &asset_resource,
        );

        // cubic
        // let mesh_material_instance = begin_load_asset::<MaterialInstanceAsset>(
        //     asset_uuid!("e7824acf-05a5-4757-a085-8b73136940e9"),
        //     &asset_resource,
        // );
        // let mesh = begin_load_asset::<MeshAsset>(
        //     asset_uuid!("93ed839e-1ad8-44e4-927d-e82f5c986d46"),
        //     &asset_resource,
        // );

        // textured cube
        // let mesh_material_instance = begin_load_asset::<MaterialInstanceAsset>(
        //     asset_uuid!("cdd1b1a3-d609-47e3-b21d-49e511b088ec"),
        //     &asset_resource,
        // );
        // let mesh = begin_load_asset::<MeshAsset>(
        //     asset_uuid!("a0bff2a6-2ee8-44d6-b114-33d1b2bc5fff"),
        //     &asset_resource,
        // );

        // spaceship
        // let mesh_material_instance = begin_load_asset::<MaterialInstanceAsset>(
        //     asset_uuid!("b822771c-f9e8-4c2c-af4a-6d38005c59e2"),
        //     &asset_resource,
        // );
        // let mesh = begin_load_asset::<MeshAsset>(
        //     asset_uuid!("76143607-6937-433b-b2a7-d1719f5211be"),
        //     &asset_resource,
        // );

        // cobblestone gltf
        // let mesh_material_instance = begin_load_asset::<MaterialInstanceAsset>(
        //     asset_uuid!("2eb77796-a5c0-42c7-bc03-1580041e4dfe"),
        //     &asset_resource,
        // );
        // let mesh = begin_load_asset::<MeshAsset>(
        //     asset_uuid!("6a1efc0f-1e67-4a6f-b5ee-185607cb3b0f"),
        //     &asset_resource,
        // );

        // cobblestone glb
        let mesh_material_instance = begin_load_asset::<MaterialInstanceAsset>(
            asset_uuid!("fbf19e7c-d416-4290-a86c-638409d15b3c"),
            &asset_resource,
        );
        let mesh = begin_load_asset::<MeshAsset>(
            asset_uuid!("3244889f-da82-474a-a7fe-e4ec37adedde"),
            &asset_resource,
        );


        let mut renderer = GameRenderer {
            time_state: time_state.clone(),
            imgui_event_listener,
            resource_manager,

            sprite_material,
            sprite_material_instance,
            sprite_custom_material: None,
            mesh,
            mesh_material,
            mesh_material_instance,
            mesh_custom_material: None,

            swapchain_surface_info: None,
            sprite_renderpass: None,
            mesh_renderpass: None,
        };

        println!("Wait for the sprite_material");
        wait_for_asset_to_load(
            device_context,
            &renderer.sprite_material.clone(),
            asset_resource,
            &mut renderer,
            "sprite_material"
        );

        println!("Wait for the sprite_material instance");
        wait_for_asset_to_load(
            device_context,
            &renderer.sprite_material_instance.clone(),
            asset_resource,
            &mut renderer,
            "sprite_material_instance"
        );


        wait_for_asset_to_load(
            device_context,
            &renderer.mesh_material.clone(),
            asset_resource,
            &mut renderer,
            "mesh material"
        );

        wait_for_asset_to_load(
            device_context,
            &renderer.mesh_material_instance.clone(),
            asset_resource,
            &mut renderer,
            "mesh material instance"
        );

        wait_for_asset_to_load(
            device_context,
            &renderer.mesh.clone(),
            asset_resource,
            &mut renderer,
            "mesh"
        );


        println!("all waits complete");

        let image_info = renderer.resource_manager.get_image_info(&override_image);

        let extents_width = 900;
        let extents_height = 600;
        let aspect_ration = extents_width as f32 / extents_height as f32;
        let half_width = 400.0;
        let half_height = 400.0 / aspect_ration;
        let proj = crate::renderpass::sprite_renderpass::orthographic_rh_gl(
            -half_width,
            half_width,
            -half_height,
            half_height,
            -100.0,
            100.0,
        );

        let mut sprite_custom_material = renderer
            .resource_manager
            .create_dyn_material_instance_from_asset(renderer.sprite_material_instance.clone())?;
        sprite_custom_material.set_image(&"texture".to_string(), &image_info.image_view);
        sprite_custom_material.set_buffer_data(&"view_proj".to_string(), &proj);
        sprite_custom_material.flush();

        renderer.sprite_custom_material = Some(sprite_custom_material);

        let mut mesh_custom_material = renderer
            .resource_manager
            .create_dyn_material_instance_from_asset(renderer.mesh_material_instance.clone())?;
        mesh_custom_material.set_buffer_data(&"view_proj".to_string(), &proj);
        mesh_custom_material.flush();

        renderer.mesh_custom_material = Some(mesh_custom_material);

        Ok(renderer)
    }

    pub fn update_resources(
        &mut self,
        device_context: &VkDeviceContext,
    ) {
        self.resource_manager.update_resources();
    }

    pub fn update_time(
        &mut self,
        time_state: &TimeState,
    ) {
        self.time_state = time_state.clone();
    }
}

impl VkSurfaceSwapchainLifetimeListener for GameRenderer {
    fn swapchain_created(
        &mut self,
        device_context: &VkDeviceContext,
        swapchain: &VkSwapchain,
    ) -> VkResult<()> {
        log::debug!("game renderer swapchain_created called");
        self.imgui_event_listener
            .swapchain_created(device_context, swapchain)?;

        let swapchain_surface_info = SwapchainSurfaceInfo {
            surface_format: swapchain.swapchain_info.surface_format,
            depth_format: swapchain.depth_format,
            extents: swapchain.swapchain_info.extents,
        };

        self.swapchain_surface_info = Some(swapchain_surface_info.clone());
        self.resource_manager.add_swapchain(&swapchain_surface_info);

        log::trace!("Create VkSpriteRenderPass");
        let sprite_pipeline_info = self.resource_manager.get_pipeline_info(
            &self.sprite_material,
            &swapchain_surface_info,
            0,
        );

        self.sprite_renderpass = Some(VkSpriteRenderPass::new(
            device_context,
            swapchain,
            sprite_pipeline_info,
        )?);


        log::trace!("Create VkMeshRenderPass");
        let mesh_pipeline_info = self.resource_manager.get_pipeline_info(
            &self.mesh_material,
            &swapchain_surface_info,
            0,
        );

        self.mesh_renderpass = Some(VkMeshRenderPass::new(
            device_context,
            swapchain,
            mesh_pipeline_info,
        )?);

        log::debug!("game renderer swapchain_created finished");

        VkResult::Ok(())
    }

    fn swapchain_destroyed(
        &mut self,
        device_context: &VkDeviceContext,
        swapchain: &VkSwapchain,
    ) {
        log::debug!("game renderer swapchain destroyed");

        let swapchain_surface_info = SwapchainSurfaceInfo {
            surface_format: swapchain.swapchain_info.surface_format,
            depth_format: swapchain.depth_format,
            extents: swapchain.swapchain_info.extents,
        };

        self.resource_manager
            .remove_swapchain(&swapchain_surface_info);
        self.imgui_event_listener
            .swapchain_destroyed(device_context, swapchain);

        self.swapchain_surface_info = None;
    }
}

impl GameRenderer {
    fn render(
        &mut self,
        asset_resource: &mut AssetResource,
        window: &Window,
        device_context: &VkDeviceContext,
        present_index: usize,
    ) -> VkResult<Vec<ash::vk::CommandBuffer>> {
        log::trace!("game renderer render");
        let mut command_buffers = vec![];

        let loop_time = self.time_state.total_time().as_secs_f32();

        //
        // Camera Management
        //
        let camera_distance_multiplier = 1.0;
        let eye = glam::Vec3::new(
            camera_distance_multiplier * 10.0 * f32::cos(loop_time),
            camera_distance_multiplier * 5.0,
            camera_distance_multiplier * 10.0 * f32::sin(loop_time)
        );

        let extents_width = 900;
        let extents_height = 600;
        let aspect_ratio = extents_width as f32 / extents_height as f32;
        let half_width = 10.0;
        let half_height = 10.0 / aspect_ratio;

        let view = glam::Mat4::look_at_lh(eye, glam::Vec3::new(0.0, 0.0, 0.0), glam::Vec3::new(0.0, -1.0, 0.0));
        let proj = glam::Mat4::perspective_lh(std::f32::consts::FRAC_PI_4, aspect_ratio, 0.5, 20.0 * camera_distance_multiplier);
        let view_proj = proj * view;

        //
        // Push latest light/camera info into the mesh material
        //
        let mut per_frame_data_shader_param = PerFrameDataShaderParam::default();
        per_frame_data_shader_param.point_lights[0].position_world = [5.0, 5.0, 5.0].into();
        per_frame_data_shader_param.point_lights[0].position_view = [5.0, 5.0, 5.0].into();
        per_frame_data_shader_param.point_lights[0].range = 25.0;
        per_frame_data_shader_param.point_lights[0].color = [1.0, 1.0, 1.0, 1.0].into();
        per_frame_data_shader_param.point_lights[0].intensity = 1.0;

        let mut material_data = GltfMaterialData::default();
        material_data.base_color_factor = [1.0, 1.0, 1.0, 1.0].into();
        material_data.has_base_color_texture = true;
        let material_data : GltfMaterialDataShaderParam = material_data.into();

        let material = self.mesh_custom_material.as_mut().unwrap();
        material.set_buffer_data(&"per_frame_data".to_string(), &per_frame_data_shader_param);
        material.set_buffer_data(&"material_data".to_string(), &material_data);
        material.set_buffer_data(&"view_proj".to_string(), &view_proj);
        material.flush();

        //
        // Update Resources and flush descriptor set changes
        //
        self.resource_manager.on_begin_frame();

        //
        // Sprite renderpass
        //
        if let Some(sprite_renderpass) = &mut self.sprite_renderpass {
            log::trace!("sprite_renderpass update");
            let dyn_pass_material_instance = self.sprite_custom_material.as_ref().unwrap().pass(0);
            let static_pass_material_instance = self.resource_manager.get_material_instance_descriptor_sets_for_current_frame(&self.sprite_material_instance, 0);

            //let pass = self.sprite_material_instance.asset.as_ref().unwrap().pass(0);

            // Pass 0 is "global"
            let descriptor_set_per_pass = dyn_pass_material_instance
                .descriptor_set_layout(0)
                .descriptor_set()
                .get_raw_for_gpu_read(&self.resource_manager);

            // Pass 1 is per-object
            // let descriptor_set_per_texture = pass
            //     .descriptor_set_layout(1)
            //     .descriptor_set()
            //     .get_raw_for_gpu_read(&self.resource_manager);
            let descriptor_set_per_texture = static_pass_material_instance.descriptor_sets[1];

            sprite_renderpass.update(
                present_index,
                1.0,
                //&self.sprite_resource_manager,
                descriptor_set_per_pass,
                &[descriptor_set_per_texture],
                &self.time_state,
            )?;

            command_buffers.push(sprite_renderpass.command_buffers[present_index].clone());
        }

        //
        // Mesh renderpass
        //
        if let Some(mesh_renderpass) = &mut self.mesh_renderpass {
            log::trace!("mesh_renderpass update");
            let mesh_pipeline_info = self.resource_manager.get_pipeline_info(
                &self.mesh_material,
                self.swapchain_surface_info.as_ref().unwrap(),
                0,
            );

            let mesh_info = self.resource_manager.get_mesh_info(&self.mesh);
            let pass = self.mesh_custom_material.as_ref().unwrap().pass(0);

            // Pass 0 is "global"
            let descriptor_set_per_pass = pass
                .descriptor_set_layout(0)
                .descriptor_set()
                .get_raw_for_gpu_read(&self.resource_manager);

            let descriptor_set_per_material = pass
                .descriptor_set_layout(1)
                .descriptor_set()
                .get_raw_for_gpu_read(&self.resource_manager);

            let descriptor_set_per_texture = pass
                .descriptor_set_layout(2)
                .descriptor_set()
                .get_raw_for_gpu_read(&self.resource_manager);

            mesh_renderpass.update(
                &mesh_pipeline_info,
                present_index,
                1.0,
                //mesh_descriptors.descriptor_sets[0],
                descriptor_set_per_pass,
                &[descriptor_set_per_material],
                &[descriptor_set_per_texture],
                &[mesh_info],
                &self.time_state,
            )?;
            command_buffers.push(mesh_renderpass.command_buffers[present_index].clone());
        }

        //
        // imgui
        //
        {
            log::trace!("imgui_event_listener update");
            let mut commands =
                self.imgui_event_listener
                    .render(window, device_context, present_index)?;
            command_buffers.append(&mut commands);
        }

        VkResult::Ok(command_buffers)
    }
}

impl Drop for GameRenderer {
    fn drop(&mut self) {
        self.sprite_renderpass = None;
        self.mesh_renderpass = None;
        self.sprite_custom_material = None;
        self.mesh_custom_material = None;
        //self.mesh_renderpass = None;
    }
}

// The context is separate from the renderer so that we can create it before creating the swapchain.
// This is required because the API design is for VkSurface to be passed temporary borrows to the
// renderer rather than owning the renderer
pub struct GameRendererWithContext {
    // Handles setting up device/instance
    context: VkContext,
    game_renderer: ManuallyDrop<GameRenderer>,
    surface: ManuallyDrop<VkSurface>,
}

impl GameRendererWithContext {
    pub fn new(
        window: &dyn Window,
        imgui_font_atlas: VkImGuiRenderPassFontAtlas,
        time_state: &TimeState,
        asset_resource: &mut AssetResource,
    ) -> Result<GameRendererWithContext, VkCreateContextError> {
        let mut context = VkContextBuilder::new()
            .use_vulkan_debug_layer(false)
            .prefer_mailbox_present_mode();

        #[cfg(debug_assertions)]
        {
            context = context.use_vulkan_debug_layer(true);
        }

        let context = context.build(window)?;

        let mut game_renderer = GameRenderer::new(
            window,
            &context.device().device_context,
            imgui_font_atlas,
            time_state,
            asset_resource,
        )?;

        let surface = VkSurface::new(&context, window, Some(&mut game_renderer))?;

        Ok(GameRendererWithContext {
            context,
            game_renderer: ManuallyDrop::new(game_renderer),
            surface: ManuallyDrop::new(surface),
        })
    }

    pub fn update_resources(&mut self) {
        self.game_renderer
            .update_resources(self.context.device_context());
    }

    pub fn draw(
        &mut self,
        asset_resource: &mut AssetResource,
        window: &dyn Window,
        time_state: &TimeState,
    ) -> VkResult<()> {
        self.game_renderer.update_time(time_state);
        self.surface.draw_with(&mut *self.game_renderer, window, |game_renderer, device_context, present_index| {
            game_renderer.render(asset_resource, window, device_context, present_index)
        })
    }

    pub fn dump_stats(&mut self) {
        if let Ok(stats) = self.context.device().allocator().calculate_stats() {
            println!("{:#?}", stats);
        } else {
            log::error!("failed to calculate stats");
        }
    }
}

impl Drop for GameRendererWithContext {
    fn drop(&mut self) {
        self.surface.tear_down(Some(&mut *self.game_renderer));
        unsafe {
            ManuallyDrop::drop(&mut self.surface);
            ManuallyDrop::drop(&mut self.game_renderer);
        }
    }
}
