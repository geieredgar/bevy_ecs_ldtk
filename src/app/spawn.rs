use std::marker::PhantomData;

use bevy::{
    ecs::system::{InputMarker, SystemParam, SystemParamFetch},
    prelude::{Bundle, In, SystemParamFunction},
    sprite::{SpriteBundle, SpriteSheetBundle},
};

use crate::{
    ldtk::FieldInstance,
    spawner::{EntitySpawner, FunctionSpawner, IntCellSpawner},
    utils::{sprite_bundle_from_tile_info, sprite_sheet_bundle_from_tile_info},
    EntityInstance, Worldly,
};

use super::{EntityInput, IntCellInput};

pub trait Spawn {
    type Context<'a>;

    fn spawn(context: Self::Context<'_>) -> Self;
}

pub trait EntitySpawnContext {
    type Param<'w, 's>: SystemParam;

    type WithLifetime<'a>;

    fn spawn<'w, 's, T>(
        input: EntityInput,
        param: <<Self::Param<'w, 's> as SystemParam>::Fetch as SystemParamFetch<'_, '_>>::Item,
    ) -> T
    where
        T: Spawn,
        for<'a> Self::WithLifetime<'a>: Into<<T as Spawn>::Context<'a>>;
}

pub trait SpawnableEntity: Bundle {
    type Param<'w, 's>: SystemParam;

    fn spawn<'w, 's>(
        input: EntityInput,
        param: <<Self::Param<'w, 's> as SystemParam>::Fetch as SystemParamFetch<'_, '_>>::Item,
    ) -> Self;
}

pub trait SpawnableIntCell: Bundle {
    type Param<'w, 's>: SystemParam;

    fn spawn<'w, 's>(
        input: IntCellInput,
        param: <<Self::Param<'w, 's> as SystemParam>::Fetch as SystemParamFetch<'_, '_>>::Item,
    ) -> Self;
}

impl<T> SpawnableEntity for T
where
    T: Spawn + Bundle,
    <T as Spawn>::Context<'static>: EntitySpawnContext,
    for<'a> <<T as Spawn>::Context<'static> as EntitySpawnContext>::WithLifetime<'a>:
        Into<<T as Spawn>::Context<'a>>,
{
    type Param<'w, 's> = <<T as Spawn>::Context<'static> as EntitySpawnContext>::Param<'w, 's>;

    fn spawn<'w, 's>(
        input: EntityInput,
        param: <<Self::Param<'w, 's> as SystemParam>::Fetch as SystemParamFetch<'_, '_>>::Item,
    ) -> Self {
        <<T as Spawn>::Context<'static> as EntitySpawnContext>::spawn(input, param)
    }
}

impl<T> SpawnableIntCell for T
where
    T: Spawn + Bundle,
    <T as Spawn>::Context<'static>: IntCellSpawnContext,
    for<'a> <<T as Spawn>::Context<'static> as IntCellSpawnContext>::WithLifetime<'a>:
        Into<<T as Spawn>::Context<'a>>,
{
    type Param<'w, 's> = <<T as Spawn>::Context<'static> as IntCellSpawnContext>::Param<'w, 's>;

    fn spawn<'w, 's>(
        input: IntCellInput,
        param: <<Self::Param<'w, 's> as SystemParam>::Fetch as SystemParamFetch<'_, '_>>::Item,
    ) -> Self {
        <<T as Spawn>::Context<'static> as IntCellSpawnContext>::spawn(input, param)
    }
}
pub trait IntCellSpawnContext {
    type Param<'w, 's>: SystemParam;

    type WithLifetime<'a>;

    fn spawn<'w, 's, T>(
        input: IntCellInput,
        param: <<Self::Param<'w, 's> as SystemParam>::Fetch as SystemParamFetch<'_, '_>>::Item,
    ) -> T
    where
        T: Spawn,
        for<'a> Self::WithLifetime<'a>: Into<<T as Spawn>::Context<'a>>;
}

impl Spawn for Worldly {
    type Context<'a> = &'a EntityInstance;

    fn spawn(entity_instance: &EntityInstance) -> Worldly {
        Worldly::from_entity_info(entity_instance)
    }
}

impl Spawn for SpriteSheetBundle {
    type Context<'a> = EntityInput<'a>;

    fn spawn(input: Self::Context<'_>) -> Self {
        sprite_sheet_bundle_from_tile_info(
            input.entity_instance.tile.as_ref(),
            input.context.tileset_map,
            input.context.tileset_definition_map,
            input.context.texture_atlases,
        )
    }
}

impl Spawn for SpriteBundle {
    type Context<'a> = EntityInput<'a>;

    fn spawn(input: Self::Context<'_>) -> Self {
        sprite_bundle_from_tile_info(
            input.entity_instance.tile.as_ref(),
            input.context.tileset_map,
        )
    }
}

impl Spawn for EntityInstance {
    type Context<'a> = EntityInstance;

    fn spawn(context: Self::Context<'_>) -> Self {
        context
    }
}

impl<T: Spawn> Spawn for Option<T>
where
    for<'a, 'b> &'a mut T::Context<'b>: Into<&'a FieldInstance>,
{
    type Context<'a> = T::Context<'a>;

    fn spawn(mut context: Self::Context<'_>) -> Self {
        match Into::<&FieldInstance>::into(&mut context).value {
            crate::prelude::FieldValue::Int(None)
            | crate::prelude::FieldValue::Float(None)
            | crate::prelude::FieldValue::String(None)
            | crate::prelude::FieldValue::FilePath(None)
            | crate::prelude::FieldValue::Enum(None)
            | crate::prelude::FieldValue::Tile(None)
            | crate::prelude::FieldValue::EntityRef(None)
            | crate::prelude::FieldValue::Point(None) => None,
            _ => Some(T::spawn(context)),
        }
    }
}

impl EntitySpawnContext for EntityInput<'static> {
    type Param<'w, 's> = ();

    type WithLifetime<'a> = EntityInput<'a>;

    fn spawn<'w, 's, T>(
        input: EntityInput,
        _: <<Self::Param<'w, 's> as SystemParam>::Fetch as SystemParamFetch<'_, '_>>::Item,
    ) -> T
    where
        T: Spawn,
        for<'a> Self::WithLifetime<'a>: Into<<T as Spawn>::Context<'a>>,
    {
        T::spawn(input.into())
    }
}

impl IntCellSpawnContext for IntCellInput<'static> {
    type Param<'w, 's> = ();

    type WithLifetime<'a> = IntCellInput<'a>;

    fn spawn<'w, 's, T>(
        input: IntCellInput,
        _: <<Self::Param<'w, 's> as SystemParam>::Fetch as SystemParamFetch<'_, '_>>::Item,
    ) -> T
    where
        T: Spawn,
        for<'a> Self::WithLifetime<'a>: Into<<T as Spawn>::Context<'a>>,
    {
        T::spawn(input.into())
    }
}

pub fn from_entity_input<T: for<'a> Spawn<Context<'a> = EntityInput<'a>>>(
) -> fn(In<EntityInput<'_>>) -> T {
    |In(input)| T::spawn(input)
}
