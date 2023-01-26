use std::collections::HashMap;

use bevy::{
    prelude::{AssetServer, Assets},
    sprite::{SpriteBundle, SpriteSheetBundle, TextureAtlas},
};

use crate::{
    ldtk::FieldInstance,
    prelude::{LayerInstance, TilesetDefinition},
    utils, EntityInstance, TilesetMap,
};

pub trait LdtkField {
    type Context<'a>;

    fn bundle_field(context: Self::Context<'_>) -> Self;
}

pub struct DefaultFieldContext<'a> {
    pub field_instance: &'a FieldInstance,
    pub entity_instance: &'a EntityInstance,
    pub layer_instance: &'a LayerInstance,
    pub tileset_map: &'a TilesetMap,
    pub tileset_definition_map: &'a HashMap<i32, &'a TilesetDefinition>,
    pub asset_server: &'a AssetServer,
    pub texture_atlases: &'a mut Assets<TextureAtlas>,
}

impl<'a, 'b> From<&'a mut DefaultFieldContext<'b>> for DefaultFieldContext<'a> {
    fn from(value: &'a mut DefaultFieldContext<'b>) -> Self {
        Self {
            field_instance: value.field_instance,
            entity_instance: value.entity_instance,
            layer_instance: value.layer_instance,
            tileset_map: value.tileset_map,
            tileset_definition_map: value.tileset_definition_map,
            asset_server: value.asset_server,
            texture_atlases: value.texture_atlases,
        }
    }
}

impl<'a, 'b> From<&'a mut DefaultFieldContext<'b>> for &'a FieldInstance {
    fn from(value: &'a mut DefaultFieldContext<'b>) -> Self {
        value.field_instance
    }
}

impl<'a, 'b> From<&'a mut DefaultFieldContext<'b>> for &'a LayerInstance {
    fn from(value: &'a mut DefaultFieldContext<'b>) -> Self {
        value.layer_instance
    }
}

impl<'a, 'b> From<&'a mut DefaultFieldContext<'b>> for SpriteBundleFieldContext<'a> {
    fn from(value: &'a mut DefaultFieldContext<'b>) -> Self {
        (value.field_instance, value.tileset_map)
    }
}

impl<'a, 'b> From<&'a mut DefaultFieldContext<'b>> for SpriteSheetBundleFieldContext<'a> {
    fn from(value: &'a mut DefaultFieldContext<'b>) -> Self {
        (
            value.field_instance,
            value.tileset_map,
            value.tileset_definition_map,
            value.texture_atlases,
        )
    }
}

impl<T: LdtkField> LdtkField for Option<T>
where
    for<'a, 'b> &'a mut T::Context<'b>: Into<&'a FieldInstance>,
{
    type Context<'a> = T::Context<'a>;

    fn bundle_field(mut context: Self::Context<'_>) -> Self {
        match Into::<&FieldInstance>::into(&mut context).value {
            crate::prelude::FieldValue::Int(None)
            | crate::prelude::FieldValue::Float(None)
            | crate::prelude::FieldValue::String(None)
            | crate::prelude::FieldValue::FilePath(None)
            | crate::prelude::FieldValue::Enum(None)
            | crate::prelude::FieldValue::Tile(None)
            | crate::prelude::FieldValue::EntityRef(None)
            | crate::prelude::FieldValue::Point(None) => None,
            _ => Some(T::bundle_field(context)),
        }
    }
}

pub type SpriteBundleFieldContext<'a> = (&'a FieldInstance, &'a TilesetMap);

impl LdtkField for SpriteBundle {
    type Context<'a> = SpriteBundleFieldContext<'a>;

    fn bundle_field((field_instance, tileset_map): Self::Context<'_>) -> Self {
        utils::sprite_bundle_from_tile_info(field_instance.tile.as_ref(), tileset_map)
    }
}

pub type SpriteSheetBundleFieldContext<'a> = (
    &'a FieldInstance,
    &'a TilesetMap,
    &'a HashMap<i32, &'a TilesetDefinition>,
    &'a mut Assets<TextureAtlas>,
);

impl LdtkField for SpriteSheetBundle {
    type Context<'a> = SpriteSheetBundleFieldContext<'a>;

    fn bundle_field(
        (field_instance, tileset_map, tileset_definition_map, texture_atlases): Self::Context<'_>,
    ) -> Self {
        utils::sprite_sheet_bundle_from_tile_info(
            field_instance.tile.as_ref(),
            tileset_map,
            tileset_definition_map,
            texture_atlases,
        )
    }
}
