use std::prelude::v1::*;

//use super::ResourceId;
//use super::ResourceIdTrait;
use std::marker::PhantomData;
use crate::resource;
use resource::ResourceId;

pub trait ResourceIdTrait:
    Clone
    + std::fmt::Debug
    + Eq
    + std::hash::Hash
    + Ord
    + PartialEq
    + PartialOrd
    + Send
    + Sync
    + 'static
{
}


// This is a helper that determines the reads/writes required for a system. I would have preferred
// not to need this structure at all, but many of the shred types require lifetimes that just
// don't play nicely with tasks. This gets rid of those lifetimes.
#[derive(Debug)]
pub struct RequiredResources<T, ResourceId>
    where
        ResourceId: ResourceIdTrait,
{
    pub(super) reads: Vec<ResourceId>,
    pub(super) writes: Vec<ResourceId>,
    phantom_data: PhantomData<T>,
}

impl<T, ResourceId> RequiredResources<T, ResourceId>
    where
        T: RequiresResources<ResourceId>,
        ResourceId: ResourceIdTrait,
{
    pub fn new() -> Self {
        RequiredResources {
            reads: T::reads(),
            writes: T::writes(),
            phantom_data: PhantomData,
        }
    }
}

pub trait RequiresResources<ResourceId>: Sized
    where
        ResourceId: ResourceIdTrait,
{
    fn reads() -> Vec<ResourceId>;
    fn writes() -> Vec<ResourceId>;

    fn required_resources() -> RequiredResources<Self, ResourceId> {
        RequiredResources::<Self, ResourceId>::new()
    }
}

impl<ResourceId> RequiresResources<ResourceId> for ()
    where
        ResourceId: ResourceIdTrait,
{
    fn reads() -> Vec<ResourceId> {
        vec![]
    }
    fn writes() -> Vec<ResourceId> {
        vec![]
    }
}

macro_rules! impl_data {
    ( $($ty:ident),* ) => {
        impl<$($ty),*, ResourceId : ResourceIdTrait> RequiresResources<ResourceId> for ( $( $ty , )* )
            where $( $ty : RequiresResources<ResourceId> ),*
            {
                fn reads() -> Vec<ResourceId> {
                    #![allow(unused_mut)]

                    let mut r = Vec::new();

                    $( {
                        let mut reads = <$ty as RequiresResources<ResourceId>>::reads();
                        r.append(&mut reads);
                    } )*

                    r
                }

                fn writes() -> Vec<ResourceId> {
                    #![allow(unused_mut)]

                    let mut r = Vec::new();

                    $( {
                        let mut writes = <$ty as RequiresResources<ResourceId>>::writes();
                        r.append(&mut writes);
                    } )*

                    r
                }
            }
    };
}

mod impl_data {
    #![cfg_attr(rustfmt, rustfmt_skip)]

    use super::*;

    impl_data!(A);
    impl_data!(A, B);
    impl_data!(A, B, C);
    impl_data!(A, B, C, D);
    impl_data!(A, B, C, D, E);
    impl_data!(A, B, C, D, E, F);
    impl_data!(A, B, C, D, E, F, G);
    impl_data!(A, B, C, D, E, F, G, H);
    impl_data!(A, B, C, D, E, F, G, H, I);
    impl_data!(A, B, C, D, E, F, G, H, I, J);
    impl_data!(A, B, C, D, E, F, G, H, I, J, K);
    impl_data!(A, B, C, D, E, F, G, H, I, J, K, L);
    impl_data!(A, B, C, D, E, F, G, H, I, J, K, L, M);
    impl_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
    impl_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
    impl_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
    impl_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q);
    impl_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R);
    impl_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S);
    impl_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T);
    impl_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U);
    impl_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V);
    impl_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W);
    impl_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X);
    impl_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y);
    impl_data!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z);
}

impl ResourceIdTrait for ResourceId {}

//
// Hook up Read/Write to the resource system
//
impl<T: resource::Resource> RequiresResources<ResourceId> for resource::Read<T> {
    fn reads() -> Vec<ResourceId> {
        vec![ResourceId::new::<T>()]
    }
    fn writes() -> Vec<ResourceId> {
        vec![]
    }
}

impl<T: resource::Resource> RequiresResources<ResourceId> for resource::Write<T> {
    fn reads() -> Vec<ResourceId> {
        vec![]
    }
    fn writes() -> Vec<ResourceId> {
        vec![ResourceId::new::<T>()]
    }
}

impl<T: resource::Resource> RequiresResources<ResourceId> for Option<resource::Read<T>> {
    fn reads() -> Vec<ResourceId> {
        vec![ResourceId::new::<T>()]
    }
    fn writes() -> Vec<ResourceId> {
        vec![]
    }
}

impl<T: resource::Resource> RequiresResources<ResourceId> for Option<resource::Write<T>> {
    fn reads() -> Vec<ResourceId> {
        vec![]
    }
    fn writes() -> Vec<ResourceId> {
        vec![ResourceId::new::<T>()]
    }
}