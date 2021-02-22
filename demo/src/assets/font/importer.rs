use distill::importer::{Error, ImportedAsset, Importer, ImporterValue};
use distill::{core::AssetUuid, importer::ImportOp};
use rafx::api::RafxResourceType;
use serde::{Deserialize, Serialize};
use std::io::Read;
use type_uuid::*;
use crate::assets::font::FontAssetData;

#[derive(TypeUuid, Serialize, Deserialize, Default)]
#[uuid = "c0228ccb-c3d6-40c1-aa19-458f93b5aff9"]
pub struct FontImporterState(Option<AssetUuid>);

#[derive(TypeUuid)]
#[uuid = "51631327-a334-4191-9ff2-eab7106e1cae"]
pub struct FontImporter;
impl Importer for FontImporter {
    fn version_static() -> u32
        where
            Self: Sized,
    {
        1
    }

    fn version(&self) -> u32 {
        Self::version_static()
    }

    type Options = ();

    type State = FontImporterState;

    /// Reads the given bytes and produces assets.
    #[profiling::function]
    fn import(
        &self,
        _op: &mut ImportOp,
        source: &mut dyn Read,
        _options: &Self::Options,
        state: &mut Self::State,
    ) -> distill::importer::Result<ImporterValue> {
        let id = state
            .0
            .unwrap_or_else(|| AssetUuid(*uuid::Uuid::new_v4().as_bytes()));
        *state = FontImporterState(Some(id));
        let mut bytes = Vec::new();
        source.read_to_end(&mut bytes)?;

        let asset_data = FontAssetData {
            data: bytes,
        };

        Ok(ImporterValue {
            assets: vec![ImportedAsset {
                id,
                search_tags: vec![],
                build_deps: vec![],
                load_deps: vec![],
                build_pipeline: None,
                asset_data: Box::new(asset_data),
            }],
        })
    }
}