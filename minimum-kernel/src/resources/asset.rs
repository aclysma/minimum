use atelier_assets::loader::{handle::RefOp, rpc_loader::RpcLoader, Loader};

use std::sync::Arc;

use crate::GenericAssetStorage;

use type_uuid::TypeUuid;

use atelier_assets::loader as atelier_loader;

pub struct AssetResource {
    loader: RpcLoader,
    storage: GenericAssetStorage,
    tx: Arc<atelier_loader::crossbeam_channel::Sender<RefOp>>,
    rx: atelier_loader::crossbeam_channel::Receiver<RefOp>,
}

impl Default for AssetResource {
    fn default() -> Self {
        let (tx, rx) = atelier_loader::crossbeam_channel::unbounded();
        let tx = Arc::new(tx);
        let storage = GenericAssetStorage::new(tx.clone());

        let loader = RpcLoader::default();

        AssetResource {
            loader,
            storage,
            tx,
            rx,
        }
    }
}

impl AssetResource {
    pub fn add_storage<T: TypeUuid + for<'a> serde::Deserialize<'a> + 'static + Send>(&mut self) {
        self.storage.add_storage::<T>();
    }

    pub fn update(&mut self) {
        atelier_loader::handle::process_ref_ops(&self.loader, &self.rx);
        self.loader
            .process(&self.storage)
            .expect("failed to process loader");
    }

    pub fn loader(&self) -> &RpcLoader {
        &self.loader
    }

    pub fn storage(&self) -> &GenericAssetStorage {
        &self.storage
    }

    pub fn tx(&self) -> &Arc<atelier_loader::crossbeam_channel::Sender<RefOp>> {
        &self.tx
    }
}
