use std::{collections::HashMap, marker::PhantomData};

use crate::{
    app::{
        EntityInput, EntitySpawnContext, IntCellInput, LdtkEntityMap, LdtkIntCellMap,
        SpawnableEntity, SpawnableIntCell,
    },
    level::spawn_level,
    prelude::{Spawn, TilesetDefinition},
    utils::{create_entity_definition_map, create_layer_definition_map},
    LdtkAsset, LdtkLevel, LdtkSettings, LevelEvent, Respawn, Worldly,
};
use bevy::{
    ecs::system::{EntityCommands, SystemParam, SystemParamFetch, SystemParamItem},
    prelude::*,
};

pub struct Spawner<E = DefaultSpawner, I = DefaultSpawner> {
    pub(crate) entity_dispatcher: E,
    pub(crate) int_cell_dispatcher: I,
    pub(crate) entity_map: LdtkEntityMap,
    pub(crate) int_cell_map: LdtkIntCellMap,
}

pub struct FunctionSpawner<F, Out, Param, Marker>(pub F, pub PhantomData<fn(Out, Param, Marker)>);

impl<F, Out, Param, Marker> FunctionSpawner<F, Out, Param, Marker> {}

pub trait EntitySpawner {
    type Param<'w, 's>: SystemParam;

    const SIZE: usize;

    fn spawn(
        &mut self,
        index: usize,
        commands: &mut EntityCommands,
        input: EntityInput,
        param: <<Self::Param<'_, '_> as SystemParam>::Fetch as SystemParamFetch<'_, '_>>::Item,
    );
}

pub trait IntCellSpawner {
    type Param<'w, 's>: SystemParam;

    const SIZE: usize;

    fn spawn(
        &mut self,
        index: usize,
        commands: &mut EntityCommands,
        input: IntCellInput,
        param: <<Self::Param<'_, '_> as SystemParam>::Fetch as SystemParamFetch<'_, '_>>::Item,
    );
}

pub struct DefaultSpawner;

pub struct GenericSpawner<T>(PhantomData<T>);

pub struct EntitySystemSpawner<Out, Param, Marker, F>(F, PhantomData<fn(Out, Param, Marker)>);

impl<T: SpawnableEntity> EntitySpawner for GenericSpawner<T> {
    type Param<'w, 's> = <T as SpawnableEntity>::Param<'w, 's>;

    const SIZE: usize = 1;

    fn spawn(
        &mut self,
        _: usize,
        commands: &mut EntityCommands,
        input: EntityInput,
        param: <<Self::Param<'_, '_> as SystemParam>::Fetch as SystemParamFetch<'_, '_>>::Item,
    ) {
        commands.insert(<T as SpawnableEntity>::spawn(input, param));
    }
}

impl<T: SpawnableIntCell> IntCellSpawner for GenericSpawner<T> {
    type Param<'w, 's> = <T as SpawnableIntCell>::Param<'w, 's>;

    const SIZE: usize = 1;

    fn spawn(
        &mut self,
        _: usize,
        commands: &mut EntityCommands,
        input: IntCellInput,
        param: <<Self::Param<'_, '_> as SystemParam>::Fetch as SystemParamFetch<'_, '_>>::Item,
    ) {
        commands.insert(<T as SpawnableIntCell>::spawn(input, param));
    }
}

impl EntitySpawner for DefaultSpawner {
    type Param<'w, 's> = ();

    const SIZE: usize = 1;

    fn spawn(
        &mut self,
        _: usize,
        commands: &mut EntityCommands,
        input: EntityInput,
        _: <<Self::Param<'_, '_> as SystemParam>::Fetch as SystemParamFetch<'_, '_>>::Item,
    ) {
        commands.insert(input.entity_instance.clone());
    }
}

pub trait IntoEntitySpawner<Params> {
    type Spawner: EntitySpawner;

    fn into_spawner(self) -> Self::Spawner;
}

impl<Out: Bundle, Param: SystemParam, Marker, F> IntoEntitySpawner<(Out, Param, Marker)> for F
where
    F: for<'a> SystemParamFunction<EntityInput<'a>, Out, Param, Marker>,
{
    type Spawner = EntitySystemSpawner<Out, Param, Marker, F>;

    fn into_spawner(self) -> Self::Spawner {
        EntitySystemSpawner(self, PhantomData)
    }
}

impl<Out: Bundle, Param: SystemParam, Marker, F> EntitySpawner
    for EntitySystemSpawner<Out, Param, Marker, F>
where
    F: for<'a> SystemParamFunction<EntityInput<'a>, Out, Param, Marker>,
{
    type Param<'w, 's> = Param;

    const SIZE: usize = 1;

    fn spawn(
        &mut self,
        _: usize,
        commands: &mut EntityCommands,
        input: EntityInput,
        param: <<Self::Param<'_, '_> as SystemParam>::Fetch as SystemParamFetch<'_, '_>>::Item,
    ) {
        commands.insert(self.0.run(input, param));
    }
}

impl IntCellSpawner for DefaultSpawner {
    type Param<'w, 's> = ();

    const SIZE: usize = 1;

    fn spawn(
        &mut self,
        _: usize,
        commands: &mut EntityCommands,
        input: IntCellInput,
        _: <<Self::Param<'_, '_> as SystemParam>::Fetch as SystemParamFetch<'_, '_>>::Item,
    ) {
        commands.insert(input.int_grid_cell);
    }
}

impl<A: EntitySpawner, B: EntitySpawner> EntitySpawner for (A, B) {
    type Param<'w, 's> = ParamSet<
        'w,
        's,
        (
            SystemParamItem<'w, 's, A::Param<'w, 's>>,
            SystemParamItem<'w, 's, B::Param<'w, 's>>,
        ),
    >;

    const SIZE: usize = A::SIZE + B::SIZE;

    fn spawn(
        &mut self,
        index: usize,
        commands: &mut EntityCommands,
        input: EntityInput,
        mut param: <<Self::Param<'_, '_> as SystemParam>::Fetch as SystemParamFetch<'_, '_>>::Item,
    ) {
        if index < A::SIZE {
            self.0.spawn(index, commands, input, param.p0());
        } else {
            self.1.spawn(index - A::SIZE, commands, input, param.p1());
        }
    }
}

impl<A: IntCellSpawner, B: IntCellSpawner> IntCellSpawner for (A, B) {
    type Param<'w, 's> = ParamSet<
        'w,
        's,
        (
            SystemParamItem<'w, 's, A::Param<'w, 's>>,
            SystemParamItem<'w, 's, B::Param<'w, 's>>,
        ),
    >;

    const SIZE: usize = A::SIZE + B::SIZE;

    fn spawn(
        &mut self,
        index: usize,
        commands: &mut EntityCommands,
        input: IntCellInput,
        mut param: <<Self::Param<'_, '_> as SystemParam>::Fetch as SystemParamFetch<'_, '_>>::Item,
    ) {
        if index < A::SIZE {
            self.0.spawn(index, commands, input, param.p0());
        } else {
            self.1.spawn(index - A::SIZE, commands, input, param.p1());
        }
    }
}

impl<
        F: for<'a> SystemParamFunction<EntityInput<'a>, Out, Param, Marker>,
        Out: Bundle,
        Param: SystemParam,
        Marker,
    > EntitySpawner for FunctionSpawner<F, Out, Param, Marker>
{
    type Param<'w, 's> = Param;

    const SIZE: usize = 1;

    fn spawn(
        &mut self,
        _: usize,
        commands: &mut EntityCommands,
        input: EntityInput,
        param: <<Self::Param<'_, '_> as SystemParam>::Fetch as SystemParamFetch<'_, '_>>::Item,
    ) {
        commands.insert(self.0.run(input, param));
    }
}

impl<
        F: for<'a> SystemParamFunction<IntCellInput<'a>, Out, Param, Marker>,
        Out: Bundle,
        Param: SystemParam,
        Marker,
    > IntCellSpawner for FunctionSpawner<F, Out, Param, Marker>
{
    type Param<'w, 's> = Param;

    const SIZE: usize = 1;

    fn spawn(
        &mut self,
        _: usize,
        commands: &mut EntityCommands,
        input: IntCellInput,
        param: <<Self::Param<'_, '_> as SystemParam>::Fetch as SystemParamFetch<'_, '_>>::Item,
    ) {
        commands.insert(self.0.run(input, param));
    }
}

impl Default for Spawner<DefaultSpawner, DefaultSpawner> {
    fn default() -> Self {
        Self {
            entity_dispatcher: DefaultSpawner,
            int_cell_dispatcher: DefaultSpawner,
            entity_map: HashMap::new(),
            int_cell_map: HashMap::new(),
        }
    }
}

#[derive(SystemParam)]
pub(crate) struct SpawnerParam<'w, 's> {
    commands: Commands<'w, 's>,
    asset_server: Res<'w, AssetServer>,
    images: ResMut<'w, Assets<Image>>,
    texture_atlases: ResMut<'w, Assets<TextureAtlas>>,
    ldtk_assets: Res<'w, Assets<LdtkAsset>>,
    level_assets: Res<'w, Assets<LdtkLevel>>,
    ldtk_query: Query<'w, 's, &'static Handle<LdtkAsset>>,
    level_query: Query<
        'w,
        's,
        (
            Entity,
            &'static Handle<LdtkLevel>,
            &'static Parent,
            Option<&'static Respawn>,
            Option<&'static Children>,
        ),
        Or<(Added<Handle<LdtkLevel>>, With<Respawn>)>,
    >,
    worldly_query: Query<'w, 's, &'static Worldly>,
    level_events: EventWriter<'w, 's, LevelEvent>,
    ldtk_settings: Res<'w, LdtkSettings>,
}

impl<'w, 's, E: EntitySpawner, I: IntCellSpawner>
    SystemParamFunction<
        (),
        (),
        (
            SpawnerParam<'_, '_>,
            ParamSet<
                'w,
                's,
                (
                    SystemParamItem<'_, '_, E::Param<'_, '_>>,
                    SystemParamItem<'_, '_, I::Param<'_, '_>>,
                ),
            >,
        ),
        (),
    > for Spawner<E, I>
where
    E: Send + Sync + 'static,
    I: Send + Sync + 'static,
{
    fn run(
        &mut self,
        _: (),
        mut param: SystemParamItem<(
            SpawnerParam,
            ParamSet<(
                SystemParamItem<E::Param<'_, '_>>,
                SystemParamItem<I::Param<'_, '_>>,
            )>,
        )>,
    ) {
        let SpawnerParam {
            mut commands,
            asset_server,
            mut images,
            mut texture_atlases,
            ldtk_assets,
            level_assets,
            ldtk_query,
            level_query,
            worldly_query,
            mut level_events,
            ldtk_settings,
        } = param.0;
        for (ldtk_entity, level_handle, parent, respawn, children) in level_query.iter() {
            // Checking if the level has any children is an okay method of checking whether it has
            // already been processed.
            // Users will most likely not be adding children to the level entity betwen its creation
            // and its processing.
            //
            // Furthermore, there are no circumstances where an already-processed level entity needs to
            // be processed again.
            // In the case of respawning levels, the level entity will have its descendants *despawned*
            // first, by a separate system.
            let already_processed = matches!(children, Some(children) if !children.is_empty());

            if !already_processed {
                if let Ok(ldtk_handle) = ldtk_query.get(parent.get()) {
                    if let Some(ldtk_asset) = ldtk_assets.get(ldtk_handle) {
                        // Commence the spawning
                        let tileset_definition_map: HashMap<i32, &TilesetDefinition> = ldtk_asset
                            .project
                            .defs
                            .tilesets
                            .iter()
                            .map(|t| (t.uid, t))
                            .collect();

                        let entity_definition_map =
                            create_entity_definition_map(&ldtk_asset.project.defs.entities);

                        let layer_definition_map =
                            create_layer_definition_map(&ldtk_asset.project.defs.layers);

                        let worldly_set = worldly_query.iter().cloned().collect();

                        if let Some(level) = level_assets.get(level_handle) {
                            spawn_level(
                                self,
                                &mut param.1,
                                &asset_server,
                                level,
                                &mut commands,
                                &mut images,
                                &mut texture_atlases,
                                &entity_definition_map,
                                &layer_definition_map,
                                &ldtk_asset.tileset_map,
                                &tileset_definition_map,
                                worldly_set,
                                ldtk_entity,
                                &ldtk_settings,
                            );
                            level_events.send(LevelEvent::Spawned(level.level.iid.clone()));
                        }

                        if respawn.is_some() {
                            commands.entity(ldtk_entity).remove::<Respawn>();
                        }
                    }
                }
            }
        }
    }
}

impl<E: EntitySpawner, I: IntCellSpawner> Spawner<E, I> {
    pub fn entity_for_layer_optional<S: IntoEntitySpawner<Params>, Params>(
        self,
        layer_identifier: Option<String>,
        entity_identifier: Option<String>,
        spawner: S,
    ) -> Spawner<(E, S::Spawner), I> {
        let mut entities = self.entity_map;
        entities.insert((layer_identifier, entity_identifier), E::SIZE);
        Spawner {
            entity_dispatcher: (self.entity_dispatcher, spawner.into_spawner()),
            entity_map: entities,
            int_cell_dispatcher: self.int_cell_dispatcher,
            int_cell_map: self.int_cell_map,
        }
    }

    pub fn entity<S: IntoEntitySpawner<Params>, Params>(
        self,
        entity_identifier: impl Into<String>,
        spawner: S,
    ) -> Spawner<(E, S::Spawner), I> {
        self.entity_for_layer_optional(None, Some(entity_identifier.into()), spawner)
    }

    pub fn entity_for_layer<S: IntoEntitySpawner<Params>, Params>(
        self,
        layer_identifier: impl Into<String>,
        entity_identifier: impl Into<String>,
        spawner: S,
    ) -> Spawner<(E, S::Spawner), I> {
        self.entity_for_layer_optional(
            Some(layer_identifier.into()),
            Some(entity_identifier.into()),
            spawner,
        )
    }

    pub fn default_entity_for_layer<S: IntoEntitySpawner<Params>, Params>(
        self,
        layer_identifier: impl Into<String>,
        spawner: S,
    ) -> Spawner<(E, S::Spawner), I> {
        self.entity_for_layer_optional(Some(layer_identifier.into()), None, spawner)
    }

    pub fn default_entity<S: IntoEntitySpawner<Params>, Params>(
        self,
        spawner: S,
    ) -> Spawner<(E, S::Spawner), I> {
        self.entity_for_layer_optional(None, None, spawner)
    }

    pub fn int_cell_for_layer_optional<T: SpawnableIntCell>(
        self,
        layer_identifier: Option<String>,
        value: Option<i32>,
    ) -> Spawner<E, (I, GenericSpawner<T>)> {
        let mut int_grid_cells = self.int_cell_map;
        int_grid_cells.insert((layer_identifier, value), I::SIZE);
        Spawner {
            entity_dispatcher: self.entity_dispatcher,
            entity_map: self.entity_map,
            int_cell_dispatcher: (self.int_cell_dispatcher, GenericSpawner(PhantomData)),
            int_cell_map: int_grid_cells,
        }
    }

    pub fn int_cell_for_layer<T: SpawnableIntCell>(
        self,
        layer_identifier: impl Into<String>,
        value: i32,
    ) -> Spawner<E, (I, GenericSpawner<T>)> {
        self.int_cell_for_layer_optional::<T>(Some(layer_identifier.into()), Some(value))
    }

    pub fn int_cell<T: SpawnableIntCell>(self, value: i32) -> Spawner<E, (I, GenericSpawner<T>)> {
        self.int_cell_for_layer_optional::<T>(None, Some(value))
    }

    pub fn default_int_cell_for_layer<T: SpawnableIntCell>(
        self,
        layer_identifier: impl Into<String>,
    ) -> Spawner<E, (I, GenericSpawner<T>)> {
        self.int_cell_for_layer_optional::<T>(Some(layer_identifier.into()), None)
    }

    pub fn default_int_cell<T: SpawnableIntCell>(self) -> Spawner<E, (I, GenericSpawner<T>)> {
        self.int_cell_for_layer_optional::<T>(None, None)
    }
}
