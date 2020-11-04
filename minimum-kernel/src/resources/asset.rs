use atelier_assets::loader::{handle::RefOp, Loader, storage::IndirectionResolver};

use crate::AssetStorageSet;
use crate::DynAssetLoader;

use type_uuid::TypeUuid;

use atelier_assets::loader as atelier_loader;
use legion::Resources;
use crossbeam_channel::{Receiver, Sender};

pub trait AssetResourceUpdateCallback: Send + Sync {
    fn update(
        &self,
        resources: &Resources,
        asset_resource: &mut AssetResource,
    );
}

pub struct DefaultAssetResourceUpdateCallback;

impl AssetResourceUpdateCallback for DefaultAssetResourceUpdateCallback {
    fn update(
        &self,
        _resources: &Resources,
        asset_resource: &mut AssetResource,
    ) {
        asset_resource.do_update();
    }
}

pub struct AssetResource {
    loader: Loader,
    resolver: Box<dyn IndirectionResolver>,
    storage: AssetStorageSet,
    tx: Sender<RefOp>,
    rx: Receiver<RefOp>,
    update_callback: Option<Box<dyn AssetResourceUpdateCallback>>,
}

impl AssetResource {
    pub fn new(loader: Loader, resolver: Box<dyn IndirectionResolver>) -> Self {
        let (tx, rx) = atelier_loader::crossbeam_channel::unbounded();
        let storage = AssetStorageSet::new(tx.clone());

        AssetResource {
            loader,
            resolver,
            storage,
            tx,
            rx,
            update_callback: Some(Box::new(DefaultAssetResourceUpdateCallback)),
        }
    }
}

impl AssetResource {
    pub fn add_storage<T: TypeUuid + for<'a> serde::Deserialize<'a> + 'static + Send>(&mut self) {
        self.storage.add_storage::<T>();
    }

    pub fn add_storage_with_loader<AssetDataT, AssetT, LoaderT>(
        &mut self,
        loader: Box<LoaderT>,
    ) where
        AssetDataT: TypeUuid + for<'a> serde::Deserialize<'a> + 'static,
        AssetT: TypeUuid + 'static + Send,
        LoaderT: DynAssetLoader<AssetT> + 'static,
    {
        self.storage
            .add_storage_with_loader::<AssetDataT, AssetT, LoaderT>(loader);
    }

    pub fn update(
        &mut self,
        resources: &Resources,
    ) {
        // This take allows us to pass mutable self to the update callback
        let cb = self.update_callback.take().unwrap();
        cb.update(resources, self);
        self.update_callback = Some(cb);
    }

    pub fn do_update(&mut self) {
        atelier_loader::handle::process_ref_ops(&self.loader, &self.rx);
        self.loader
            .process(&self.storage, &*self.resolver)
            .expect("failed to process loader");
    }

    pub fn set_update_fn(
        &mut self,
        update_callback: Box<dyn AssetResourceUpdateCallback>,
    ) {
        self.update_callback = Some(update_callback);
    }

    pub fn loader(&self) -> &Loader {
        &self.loader
    }

    pub fn storage(&self) -> &AssetStorageSet {
        &self.storage
    }

    pub fn tx(&self) -> &atelier_loader::crossbeam_channel::Sender<RefOp> {
        &self.tx
    }
}
