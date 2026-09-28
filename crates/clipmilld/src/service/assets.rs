//! The asset folder's requests: bring a picture or a sound in, list what is
//! there, and say whether a hash is one of them for the media door.

use std::path::Path;

use clipmill_contracts::proto::ipc::v1::{
    AssetV1, ErrorCode, ImportAssetRequest, ImportAssetResponse, ListAssetsRequest,
    ListAssetsResponse, ResolveAssetRequest, ResolveAssetResponse, response,
};

use super::{Reply, Service, error_reply, response_reply};
use crate::assets::{AssetError, AssetRecord, AssetStore};

impl Service {
    /// Give the service its asset folder.
    #[must_use]
    pub(crate) fn with_assets(mut self, assets: AssetStore) -> Self {
        self.assets = Some(assets);
        self
    }

    pub(super) async fn import_asset(
        &self,
        request_id: String,
        import: &ImportAssetRequest,
    ) -> Reply {
        let Some(store) = &self.assets else {
            return unavailable(request_id);
        };
        let path = Path::new(&import.path);
        if !path.is_absolute() {
            return error_reply(
                request_id,
                ErrorCode::InvalidArgument,
                "choose the file to bring in",
            );
        }
        match store.import(path, &import.license).await {
            Ok(record) => response_reply(
                request_id,
                response::Body::ImportAsset(ImportAssetResponse {
                    asset: Some(asset_view(record)),
                }),
            ),
            Err(AssetError::Refused(reason) | AssetError::Unreadable(reason)) => {
                error_reply(request_id, ErrorCode::InvalidArgument, reason)
            }
            Err(error @ AssetError::Store(_)) => {
                error_reply(request_id, ErrorCode::Internal, error.to_string())
            }
        }
    }

    pub(super) fn list_assets(&self, request_id: String, list: &ListAssetsRequest) -> Reply {
        let Some(store) = &self.assets else {
            return unavailable(request_id);
        };
        let assets = store
            .list()
            .into_iter()
            .filter(|record| list.kind.is_empty() || record.kind.as_str() == list.kind)
            .map(asset_view)
            .collect();
        response_reply(
            request_id,
            response::Body::ListAssets(ListAssetsResponse { assets }),
        )
    }

    pub(super) fn resolve_asset(&self, request_id: String, resolve: &ResolveAssetRequest) -> Reply {
        let Some(store) = &self.assets else {
            return unavailable(request_id);
        };
        match (store.get(&resolve.hash), store.file(&resolve.hash)) {
            (Some(record), Some(_)) => response_reply(
                request_id,
                response::Body::ResolveAsset(ResolveAssetResponse {
                    hash: record.hash,
                    media_type: record.media_type,
                    bytes: record.bytes,
                }),
            ),
            _ => error_reply(request_id, ErrorCode::NotFound, "no asset has that hash"),
        }
    }
}

fn unavailable(request_id: String) -> Reply {
    error_reply(
        request_id,
        ErrorCode::Unavailable,
        "this daemon keeps no asset folder",
    )
}

pub(super) fn asset_view(record: AssetRecord) -> AssetV1 {
    AssetV1 {
        hash: record.hash,
        kind: record.kind.as_str().to_owned(),
        name: record.name,
        media_type: record.media_type,
        bytes: record.bytes,
        width: record.width,
        height: record.height,
        duration_ticks: record.duration_ticks,
        license: record.license,
        added_unix_millis: record.added_unix_millis,
    }
}
