use crate::{
    components::{IntGridCell, IntGridCellBundle},
    ldtk::LayerInstance,
    prelude::TilesetDefinition,
    EntityInstance, TilesetMap,
};
use bevy::{ecs::system::EntityCommands, prelude::*};
use std::{collections::HashMap, marker::PhantomData};

/// [Bundle]: bevy::prelude::Bundle
/// [App]: bevy::prelude::App
/// [Component]: bevy::prelude::Component
///
/// Provides a constructor which can be used for spawning additional components on IntGrid tiles.
///
/// After implementing this trait on a [Bundle], you can register it to spawn automatically for a
/// given int grid value via [RegisterLdtkObjects] on your [App].
///
/// For common use cases, you'll want to use derive-macro `#[derive(LdtkIntCell)]`, but you can
/// also provide a custom implementation.
///
/// You can also implement this trait on non-[Bundle] types, but only [Bundle]s can be registered.
///
/// If there is an IntGrid tile in the LDtk file whose value is NOT registered, an entity will be
/// spawned with an [IntGridCell] component, allowing you to flesh it out in your own system.
///
/// *Derive macro requires the "derive" feature, which is enabled by default*
///
/// ## Derive macro usage
/// Using `#[derive(LdtkIntCell)]` on a [Bundle] struct will allow the type to be registered to the
/// [App] via [RegisterLdtkObjects] functions:
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_ecs_ldtk::prelude::*;
///
/// fn main() {
///     App::empty()
///         .add_plugin(LdtkPlugin)
///         .register_ldtk_int_cell::<MyBundle>(1)
///         // add other systems, plugins, resources...
///         .run();
/// }
///
/// # #[derive(Component, Default)]
/// # struct ComponentA;
/// # #[derive(Component, Default)]
/// # struct ComponentB;
/// # #[derive(Component, Default)]
/// # struct ComponentC;
/// #[derive(Bundle, LdtkIntCell)]
/// pub struct MyBundle {
///     a: ComponentA,
///     b: ComponentB,
///     c: ComponentC,
/// }
/// ```
/// Now, when loading your ldtk file, any IntGrid tiles with the value `1` will be spawned with as
/// tiles with `MyBundle` inserted.
///
/// By default, each component or nested bundle in the bundle will be created using their [Default]
/// implementations.
/// However, this behavior can be overriden with some field attribute macros...
///
/// ### `#[ldtk_int_cell]`
/// Indicates that a component or bundle that implements [LdtkIntCell] should be created with
/// [LdtkIntCell::bundle_int_cell], allowing for nested [LdtkIntCell]s.
///
/// Note: the [LdtkIntCell] field decorated with this attribute doesn't have to be a [Bundle].
/// This can be useful if a [Component]'s construction requires the additional access to the world
/// provided by [LdtkIntCell::bundle_int_cell].
/// ```
/// # use bevy::prelude::*;
/// # use bevy_ecs_ldtk::prelude::*;
/// # #[derive(Component, Default)]
/// # struct RigidBody;
/// # #[derive(Component, Default)]
/// # struct Damage;
/// #[derive(Bundle, LdtkIntCell)]
/// pub struct Wall {
///     rigid_body: RigidBody,
/// }
///
/// #[derive(Bundle, LdtkIntCell)]
/// pub struct DestructibleWall {
///     #[ldtk_int_cell]
///     #[bundle]
///     wall: Wall,
///     damage: Damage,
/// }
/// ```
///
/// ### `#[from_int_grid_cell]`
/// Indicates that a component or bundle that implements [From<IntGridCell>] should be created
/// using that conversion.
/// This allows for more modular and custom component construction, and for different structs that
/// contain the same component to have different constructions of that component, without having to
/// `impl LdtkIntCell` for both of them.
/// It also allows you to have an [IntGridCell] field, since all types `T` implement `From<T>`.
/// ```
/// # use bevy::prelude::*;
/// # use bevy_ecs_ldtk::prelude::*;
/// # #[derive(Component, Default)]
/// # struct Fluid { viscosity: i32 }
/// # #[derive(Component, Default)]
/// # struct Damage;
/// impl From<IntGridCell> for Fluid {
///     fn from(int_grid_cell: IntGridCell) -> Fluid {
///         let viscosity = match int_grid_cell.value {
///             1 => 5,
///             2 => 20,
///             _ => 0,
///         };
///
///         Fluid {
///             viscosity,
///         }
///     }
/// }
///
/// #[derive(Bundle, LdtkIntCell)]
/// pub struct Lava {
///     #[from_int_grid_cell]
///     fluid: Fluid,
///     #[from_int_grid_cell]
///     int_grid_cell: IntGridCell,
///     damage: Damage,
/// }
/// ```
pub trait LdtkIntCell {
    /// The constructor used by the plugin when spawning additional components on IntGrid tiles.
    /// If you need access to more of the [World](bevy::prelude::World), you can create a system that queries for
    /// `Added<IntGridCell>`, and flesh out the entity from there, instead of implementing this
    /// trait.
    /// This is because the plugin spawns a tile with an [IntGridCell] component if the tile's
    /// value is not registered to the app.
    ///
    /// Note: whether or not the entity is registered to the app, the plugin will insert a
    /// [SpatialBundle](bevy::prelude::SpatialBundle) to the entity **after** this bundle is
    /// inserted.
    /// So, any custom implementations of these components within this trait will be overwritten.
    /// Furthermore, a [bevy_ecs_tilemap::tiles::TileBundle] will be inserted **before** this bundle, so
    /// be careful not to overwrite the components provided by that bundle.
    fn bundle_int_cell(int_grid_cell: IntGridCell, layer_instance: &LayerInstance) -> Self;
}

impl LdtkIntCell for IntGridCellBundle {
    fn bundle_int_cell(int_grid_cell: IntGridCell, _: &LayerInstance) -> Self {
        IntGridCellBundle { int_grid_cell }
    }
}

pub struct IntCellInput<'a> {
    pub int_grid_cell: IntGridCell,
    pub context: LayerSpawnContext<'a>,
}

impl<'a, 'b> From<&'a mut IntCellInput<'b>> for IntCellInput<'a> {
    fn from(value: &'a mut IntCellInput<'b>) -> Self {
        Self {
            int_grid_cell: value.int_grid_cell,
            context: (&mut value.context).into(),
        }
    }
}

impl<'a, 'b> From<&'a mut EntityInput<'b>> for EntityInput<'a> {
    fn from(value: &'a mut EntityInput<'b>) -> Self {
        Self {
            entity_instance: value.entity_instance,
            context: (&mut value.context).into(),
        }
    }
}

impl<'a, 'b> From<&'a mut EntityInput<'b>> for EntityInstance {
    fn from(value: &'a mut EntityInput<'b>) -> Self {
        value.entity_instance.clone()
    }
}

impl<'a, 'b> From<&'a mut EntityInput<'b>> for &'a EntityInstance {
    fn from(value: &'a mut EntityInput<'b>) -> Self {
        value.entity_instance
    }
}

impl<'a, 'b> From<&'a mut LayerSpawnContext<'b>> for LayerSpawnContext<'a> {
    fn from(value: &'a mut LayerSpawnContext<'b>) -> Self {
        Self {
            layer_instance: value.layer_instance,
            tileset_map: value.tileset_map,
            tileset_definition_map: value.tileset_definition_map,
            asset_server: value.asset_server,
            texture_atlases: value.texture_atlases,
        }
    }
}

pub struct EntityInput<'a> {
    pub entity_instance: &'a EntityInstance,
    pub context: LayerSpawnContext<'a>,
}

pub struct LayerSpawnContext<'a> {
    pub layer_instance: &'a LayerInstance,
    pub tileset_map: &'a TilesetMap,
    pub tileset_definition_map: &'a HashMap<i32, &'a TilesetDefinition>,
    pub asset_server: &'a AssetServer,
    pub texture_atlases: &'a mut Assets<TextureAtlas>,
}
/// Used by [RegisterLdtkObjects] to associate Ldtk IntGrid values with [LdtkIntCell]s.
pub type LdtkIntCellMap = HashMap<(Option<String>, Option<i32>), usize>;
