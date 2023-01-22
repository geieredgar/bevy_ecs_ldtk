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
    fn bundle_field(context: LdtkFieldContext) -> Self;
}

pub struct LdtkFieldContext<'a> {
    pub field_instance: &'a FieldInstance,
    pub entity_instance: &'a EntityInstance,
    pub layer_instance: &'a LayerInstance,
    pub tileset_map: &'a TilesetMap,
    pub tileset_definition_map: &'a HashMap<i32, &'a TilesetDefinition>,
    pub asset_server: &'a AssetServer,
    pub texture_atlases: &'a mut Assets<TextureAtlas>,
}

impl<T: LdtkField> LdtkField for Option<T> {
    fn bundle_field(context: LdtkFieldContext) -> Self {
        match context.field_instance.value {
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

impl LdtkField for SpriteBundle {
    fn bundle_field(context: LdtkFieldContext) -> Self {
        utils::sprite_bundle_from_tile_info(
            context.field_instance.tile.as_ref(),
            context.tileset_map,
        )
    }
}

impl LdtkField for SpriteSheetBundle {
    fn bundle_field(context: LdtkFieldContext) -> Self {
        utils::sprite_sheet_bundle_from_tile_info(
            context.field_instance.tile.as_ref(),
            context.tileset_map,
            context.tileset_definition_map,
            context.texture_atlases,
        )
    }
}
