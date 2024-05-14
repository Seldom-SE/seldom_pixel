use bevy::{
    ecs::{query::QueryFilter, system::SystemParam},
    prelude::*,
};

use crate::asset::{PxAsset, PxAssetData};

pub(crate) trait SystemGet<'a, M>: 'a + Sized {
    type Param<'w, 's>: SystemParam
    where
        'w: 'a,
        's: 'a;
    type Filter: QueryFilter;

    fn get<'w: 'a, 's: 'a>(entity: Entity, param: &'a Self::Param<'w, 's>) -> Option<Self>;
}

impl<'a, T: Component> SystemGet<'a, ()> for &'a T {
    type Param<'w, 's> = Query<'w, 's, &'static T> where 'w: 'a, 's: 'a;
    type Filter = With<T>;

    fn get<'w: 'a, 's: 'a>(entity: Entity, param: &'a Self::Param<'w, 's>) -> Option<Self> {
        Some(param.get(entity).unwrap())
    }
}

// Temporarily commented out
// impl<'a, 'w: 'a, 's: 'a, T: PxAssetData> SystemGet<'a, 'w, 's, bool> for &'a T {
//     type Param = (
//         Query<'w, 's, &'static Handle<PxAsset<T>>>,
//         Res<'w, Assets<PxAsset<T>>>,
//     );
//     type Filter = With<Handle<PxAsset<T>>>;
//
//     fn get(entity: Entity, (handles, assets): &'a Self::Param) -> Option<Self> {
//         let PxAsset::Loaded { asset } = assets.get(handles.get(entity).unwrap())? else {
//             return None;
//         };
//
//         Some(asset)
//     }
// }
//
// impl<'a, 'w: 'a, 's: 'a, M, N, T: SystemGet<'a, 'w, 's, M>, U: SystemGet<'a, 'w, 's, N>>
//     SystemGet<'a, 'w, 's, (M, N)> for (T, U)
// {
//     type Param = (T::Param, U::Param);
//     type Filter = (T::Filter, U::Filter);
//
//     fn get(entity: Entity, (t_param, u_param): &'a Self::Param) -> Option<Self> {
//         Some((T::get(entity, t_param)?, U::get(entity, u_param)?))
//     }
// }

// pub(crate) trait SystemGet<'a, M>: Sized {
//     type Param<'w, 's>: SystemParam;
//     type Filter: QueryFilter;
//
//     fn get(entity: Entity, param: &'a Self::Param<'_, '_>) -> Option<Self>;
// }
//
// impl<'a, T: Component> SystemGet<'a, ()> for &'a T {
//     type Param<'w, 's> = Query<'w, 's, &'static T>;
//     type Filter = With<T>;
//
//     fn get(entity: Entity, param: &'a Self::Param<'_, '_>) -> Option<Self> {
//         Some(param.get(entity).unwrap())
//     }
// }
//
// impl<'a, T: PxAssetData> SystemGet<'a, bool> for &'a T {
//     type Param<'w, 's> = (
//         Query<'w, 's, &'static Handle<PxAsset<T>>>,
//         Res<'w, Assets<PxAsset<T>>>,
//     );
//     type Filter = With<Handle<PxAsset<T>>>;
//
//     fn get(entity: Entity, (handles, assets): &'a Self::Param<'_, '_>) -> Option<Self> {
//         let PxAsset::Loaded { asset } = assets.get(handles.get(entity).unwrap())? else {
//             return None;
//         };
//
//         Some(asset)
//     }
// }
//
// impl<'a, M, N, T: SystemGet<'a, M>, U: SystemGet<'a, N>> SystemGet<'a, (M, N)> for (T, U) {
//     type Param<'w, 's> = (T::Param<'w, 's>, U::Param<'w, 's>);
//     type Filter = (T::Filter, U::Filter);
//
//     fn get(entity: Entity, (t_param, u_param): &'a Self::Param<'_, '_>) -> Option<Self> {
//         Some((T::get(entity, t_param)?, U::get(entity, u_param)?))
//     }
// }
