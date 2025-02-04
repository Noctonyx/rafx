use super::RenderObjectInstance;
use crate::render_features::render_features_prelude::*;
use std::marker::PhantomData;
use std::ops::Range;

/// An `ExtractJob` implements the `RenderFeatureExtractJob` trait by wrapping an instance
/// of an `ExtractJobEntryPoints` type defined at compile-time. The `ExtractJob` contains
/// the frame packet and presents the correct context (like `ExtractPerFrameContext`) to each
/// entry point defined in `ExtractJobEntryPoints`.
pub struct ExtractJob<'extract, ExtractJobEntryPointsT: ExtractJobEntryPoints<'extract>> {
    inner: ExtractJobEntryPointsT,
    frame_packet: Option<Box<FramePacket<ExtractJobEntryPointsT::FramePacketDataT>>>,
    #[allow(dead_code)]
    debug_constants: &'static RenderFeatureDebugConstants,
    _phantom: (PhantomData<&'extract ()>,),
}

impl<'extract, ExtractJobEntryPointsT: 'extract + ExtractJobEntryPoints<'extract>>
    ExtractJob<'extract, ExtractJobEntryPointsT>
{
    pub fn new(
        inner: ExtractJobEntryPointsT,
        frame_packet: Box<FramePacket<ExtractJobEntryPointsT::FramePacketDataT>>,
    ) -> Self {
        let debug_constants = inner.feature_debug_constants();
        Self {
            inner,
            frame_packet: Some(frame_packet),
            debug_constants,
            _phantom: Default::default(),
        }
    }

    fn frame_packet(&self) -> &Option<Box<FramePacket<ExtractJobEntryPointsT::FramePacketDataT>>> {
        &self.frame_packet
    }

    fn view_packets(&self) -> &Vec<ViewPacket<ExtractJobEntryPointsT::FramePacketDataT>> {
        &self.frame_packet.as_ref().unwrap().view_packets()
    }

    fn render_object_instances(&self) -> &Vec<RenderObjectInstance> {
        &self
            .frame_packet
            .as_ref()
            .unwrap()
            .render_object_instances()
    }

    fn force_to_extract_lifetime(
        &self,
        inner: &ExtractJobEntryPointsT,
    ) -> &'extract ExtractJobEntryPointsT {
        unsafe {
            // SAFETY: The 'extract lifetime added here is already required by the ExtractJobEntryPointsT.
            // This transmute is just avoiding the need to proliferate even _more_ 'extract lifetimes through
            // _every single function_.
            std::mem::transmute::<_, &'extract ExtractJobEntryPointsT>(inner)
        }
    }
}

impl<'extract, ExtractJobEntryPointsT: 'extract + ExtractJobEntryPoints<'extract>>
    RenderFeatureExtractJob<'extract> for ExtractJob<'extract, ExtractJobEntryPointsT>
{
    fn begin_per_frame_extract(&self) {
        profiling::scope!(self.debug_constants.begin_per_frame_extract);

        let context = ExtractPerFrameContext::new(self.frame_packet.as_ref().unwrap());
        self.inner.begin_per_frame_extract(&context);
    }

    fn extract_render_object_instance(
        &self,
        range: Range<usize>,
    ) {
        if range.is_empty() {
            return;
        }

        let mut job_context = {
            let inner = self.force_to_extract_lifetime(&self.inner);
            inner.new_render_object_instance_job_context()
        };

        if job_context.is_none() {
            return;
        }

        profiling::scope!(self.debug_constants.extract_render_object_instance);

        let job_context = job_context.as_mut().unwrap();
        let frame_packet = self.frame_packet.as_ref().unwrap();
        for id in range {
            let context = ExtractRenderObjectInstanceContext::new(frame_packet, id);
            self.inner
                .extract_render_object_instance(job_context, &context);
        }
    }

    fn view_packet(
        &self,
        view_index: ViewFrameIndex,
    ) -> &dyn RenderFeatureViewPacket {
        self.frame_packet()
            .as_ref()
            .unwrap()
            .render_feature_view_packet(view_index)
    }

    fn extract_render_object_instance_per_view(
        &self,
        view_packet: &dyn RenderFeatureViewPacket,
        range: Range<usize>,
    ) {
        if range.is_empty() {
            return;
        }

        let mut job_context = {
            let inner = self.force_to_extract_lifetime(&self.inner);
            inner.new_render_object_instance_per_view_job_context()
        };

        if job_context.is_none() {
            return;
        }

        profiling::scope!(self.debug_constants.extract_render_object_instance_per_view);

        let job_context = job_context.as_mut().unwrap();
        let frame_packet = self.frame_packet.as_ref().unwrap();
        let view_packet: &ViewPacket<ExtractJobEntryPointsT::FramePacketDataT> =
            view_packet.as_concrete();

        for id in range {
            let context =
                ExtractRenderObjectInstancePerViewContext::new(frame_packet, view_packet, id);
            self.inner
                .extract_render_object_instance_per_view(job_context, &context);
        }
    }

    fn end_per_view_extract(
        &self,
        view_packet: &dyn RenderFeatureViewPacket,
    ) {
        profiling::scope!(self.debug_constants.end_per_view_extract);

        let view_packet: &ViewPacket<ExtractJobEntryPointsT::FramePacketDataT> =
            view_packet.as_concrete();

        let context = ExtractPerViewContext::new(self.frame_packet.as_ref().unwrap(), view_packet);
        self.inner.end_per_view_extract(&context);
    }

    fn end_per_frame_extract(&self) {
        profiling::scope!(self.debug_constants.end_per_frame_extract);

        let context = ExtractPerFrameContext::new(self.frame_packet.as_ref().unwrap());
        self.inner.end_per_frame_extract(&context);
    }

    fn num_views(&self) -> usize {
        self.view_packets().len()
    }

    fn num_render_object_instances(&self) -> usize {
        self.render_object_instances().len()
    }

    fn take_frame_packet(&mut self) -> Box<dyn RenderFeatureFramePacket> {
        std::mem::take(&mut self.frame_packet).unwrap()
    }

    fn feature_debug_constants(&self) -> &'static RenderFeatureDebugConstants {
        self.inner.feature_debug_constants()
    }

    fn feature_index(&self) -> RenderFeatureIndex {
        self.inner.feature_index()
    }
}
