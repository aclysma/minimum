use atelier_assets::loader::{handle::RefOp, Loader, storage::IndirectionResolver};

use crate::AssetStorageSet;
use crate::DynAssetLoader;

use type_uuid::TypeUuid;

use atelier_assets::loader as atelier_loader;
use atelier_assets::core::AssetUuid;
use atelier_assets::loader::handle::Handle;
use atelier_assets::loader::storage::IndirectIdentifier;
use atelier_assets::loader::storage::LoadStatus;
use atelier_assets::loader::storage::LoadInfo;
use legion::Resources;
use crossbeam_channel::{Receiver, Sender};
use atelier_assets::loader::handle::AssetHandle;

/// If additional processing needs to be triggered in order to update assets, implement this
/// callback and pass it into AssetResource::set_update_fn. The implementation must call do_update()
/// on the provided asset_resource
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

/// A user-friendly interface to fetching/storing/loading assets. Meant to be a resource in an ECS
/// system
pub struct AssetResource {
    loader: Loader,
    resolver: Box<dyn IndirectionResolver + Send + Sync + 'static>,
    storage: AssetStorageSet,
    tx: Sender<RefOp>,
    rx: Receiver<RefOp>,
    update_callback: Option<Box<dyn AssetResourceUpdateCallback>>,
}

impl AssetResource {
    pub fn new(
        loader: Loader,
        resolver: Box<dyn IndirectionResolver + Send + Sync + 'static>,
    ) -> Self {
        let (tx, rx) = atelier_loader::crossbeam_channel::unbounded();
        let storage = AssetStorageSet::new(tx.clone(), loader.indirection_table());

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
    /// Adds a default storage object for assets of type T
    pub fn add_storage<T: TypeUuid + for<'a> serde::Deserialize<'a> + 'static + Send>(&mut self) {
        self.storage.add_storage::<T>();
    }

    /// Adds a storage object for assets of type T that proxies loading events to the given loader.
    /// This allows an end-user to do additional processing to "prepare" the asset. For example, a
    /// texture might be uploaded to GPU memory before being considered loaded.
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

    /// Call this frequently to update the asset loading system. This will call the function
    /// provided by set_update_fn, which should then call do_update. This allows for custom code to
    /// occur before/after update calls which might be triggered at the engine level rather than
    /// from user code.
    pub fn update(
        &mut self,
        resources: &Resources,
    ) {
        // This take allows us to pass mutable self to the update callback
        let cb = self.update_callback.take().unwrap();
        cb.update(resources, self);
        self.update_callback = Some(cb);
    }

    /// Should be called from the AssetResourceUpdateCallback impl provided to set_update_fn
    pub fn do_update(&mut self) {
        atelier_loader::handle::process_ref_ops(&self.loader, &self.rx);
        self.loader
            .process(&self.storage, &*self.resolver)
            .expect("failed to process loader");
    }

    /// If needed, call this function to change the asset manager update function. The callback
    /// must call do_update on AssetResource within the function
    pub fn set_update_fn(
        &mut self,
        update_callback: Box<dyn AssetResourceUpdateCallback>,
    ) {
        self.update_callback = Some(update_callback);
    }

    //
    // These functions map to atelier-assets APIs
    //
    pub fn load_asset<T>(
        &self,
        asset_uuid: AssetUuid,
    ) -> Handle<T> {
        let load_handle = self.loader.add_ref(asset_uuid);
        Handle::<T>::new(self.tx.clone(), load_handle)
    }

    pub fn load_asset_indirect<T>(
        &self,
        id: IndirectIdentifier,
    ) -> Handle<T> {
        let load_handle = self.loader.add_ref_indirect(id);
        Handle::<T>::new(self.tx.clone(), load_handle)
    }

    pub fn load_asset_path<T, U: Into<String>>(
        &self,
        path: U,
    ) -> Handle<T> {
        let load_handle = self
            .loader
            .add_ref_indirect(IndirectIdentifier::Path(path.into()));
        Handle::<T>::new(self.tx.clone(), load_handle)
    }

    pub fn asset<T: TypeUuid + 'static + Send>(
        &self,
        handle: &Handle<T>,
    ) -> Option<&T> {
        handle.asset(&self.storage)
    }

    pub fn asset_version<T: TypeUuid + 'static + Send>(
        &self,
        handle: &Handle<T>,
    ) -> Option<u32> {
        handle.asset_version::<T, _>(&self.storage)
    }

    pub fn with_serde_context<R>(
        &self,
        f: impl FnMut() -> R,
    ) -> R {
        self.loader.with_serde_context(&self.tx, f)
    }

    pub fn load_status<T>(
        &self,
        handle: &Handle<T>,
    ) -> LoadStatus {
        handle.load_status(&self.loader)
    }

    pub fn load_info<T>(
        &self,
        handle: &Handle<T>,
    ) -> Option<LoadInfo> {
        self.loader.get_load_info(handle.load_handle())
    }
}
