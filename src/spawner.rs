use crate::prelude::*;
use bevy::utils::HashSet;

pub fn spawn_player(
    mut commands: Commands,
    atlas: Res<CharsetAsset>,
    mut mb: ResMut<MapBuilder>,
) {
    let player_start = mb.player_start;

    let entity = commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: atlas.atlas.clone(),
            sprite: TextureAtlasSprite {
                custom_size: Some(Vec2::new(1.0, 1.0)), 
                index: '@' as usize, 
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Position { x: player_start.x, y: player_start.y, z: 2 })
        .insert(TileSize::square(1.0))
        .insert(Health{current: 10, max: 20})
        .insert(Player{map_level: 0})
        .insert(Naming("Player".to_string()))
        .insert(FieldOfView::new(8))
        .id();

    mb.entity_occupy_tile(entity, player_start);
}

// max hp, name (like "Orc"), ascii code (like "o")
fn goblin() -> (i32, String, char) {
    (1, "Goblin".to_string(), 'g')
}

fn orc() -> (i32, String, char) {
    (2, "Orc".to_string(), 'o')
}

pub fn spawn_enemies(
    mut commands: Commands,
    atlas: Res<CharsetAsset>,
    mut mb: ResMut<MapBuilder>,
) {
    let mut rng = rand::thread_rng();
    let enemies_start = mb.enemies_start.clone();

    for position in enemies_start {

        let roll = rng.gen_range(1..3);
        match roll {
            1 => spawn_healing_potion(&mut commands, atlas.atlas.clone(), &position),
            2 => spawn_magic_mapper(&mut commands, atlas.atlas.clone(), &position),
            _ => {
                let (hp, name, glyph) = match rng.gen_range(0..4) {
                    0 => orc(),
                    _ => goblin(),
                };
                
                let monster_entity = spawn_enemy(
                    &mut commands, 
                    atlas.atlas.clone(), 
                    TextureAtlasSprite {
                        color: Color::rgb(0.698, 0.094, 0.168),
                        custom_size: Some(Vec2::new(1.0, 1.0)), 
                        index: glyph as usize, 
                        ..Default::default()
                    },
                    &name,
                    hp,
                    &position);
        
                mb.entity_occupy_tile(monster_entity, position);
            }
        }
    }
}


fn spawn_enemy(
    commands: &mut Commands,
    atlas: Handle<TextureAtlas>,
    sprite: TextureAtlasSprite,
    name: &String,
    hp: i32,
    position: &Position,
) -> Entity 
{
    commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: atlas,
            sprite: sprite,
            visibility: Visibility{is_visible:false},
            ..Default::default()
        })
        .insert(Naming(name.clone()))
        .insert(Health{current: hp, max: hp})
        .insert(Position { x: position.x, y: position.y, z: 2 })
        .insert(TileSize::square(1.0))
        .insert(ChasingPlayer)
        .insert(FieldOfView::new(8))
        .insert(Enemy).id()
}

fn spawn_amulet_of_yala(
    mut commands: Commands,
    atlas: Res<CharsetAsset>,
    mb: Res<MapBuilder>,
) {
    let amulet_start = mb.amulet_start;
    commands
    .spawn_bundle(SpriteSheetBundle {
        texture_atlas: atlas.atlas.clone(),
        sprite: TextureAtlasSprite {
            color: Color::GOLD,
            custom_size: Some(Vec2::new(1.0, 1.0)), 
            index: 6, 
            ..Default::default()
        },
        visibility: Visibility{is_visible:false},
        ..Default::default()
    })
    .insert(Naming("Amulet of Yala".to_string()))
    .insert(Position { x: amulet_start.x, y: amulet_start.y, z: 2 })
    .insert(TileSize::square(1.0))
    .insert(Item)
    .insert(AmuletOfYala);
}

fn spawn_healing_potion(
    commands: &mut Commands,
    atlas: Handle<TextureAtlas>,
    position: &Position,
) {
    commands
    .spawn_bundle(SpriteSheetBundle {
        texture_atlas: atlas,
        sprite: TextureAtlasSprite {
            color: Color::GREEN,
            custom_size: Some(Vec2::new(1.0, 1.0)), 
            index: 'p' as usize, 
            ..Default::default()
        },
        visibility: Visibility{is_visible:false},
        ..Default::default()
    })
    .insert(Naming("Healing Potion".to_string()))
    .insert(Description("Heals 6 Health Points.".to_string()))
    .insert(Position { x: position.x, y: position.y, z: 2 })
    .insert(TileSize::square(1.0))
    .insert(Item)
    .insert(ProvidesHealing{amount: 6});
}

fn spawn_magic_mapper(
    commands: &mut Commands,
    atlas: Handle<TextureAtlas>,
    position: &Position,
) {
    commands
    .spawn_bundle(SpriteSheetBundle {
        texture_atlas: atlas,
        sprite: TextureAtlasSprite {
            color: Color::GREEN,
            custom_size: Some(Vec2::new(1.0, 1.0)), 
            index: 'm' as usize, 
            ..Default::default()
        },
        visibility: Visibility{is_visible:false},
        ..Default::default()
    })
    .insert(Naming("Dungeon Map".to_string()))
    .insert(Description("Reveals all the map tiles.".to_string()))
    .insert(Position { x: position.x, y: position.y, z: 2 })
    .insert(TileSize::square(1.0))
    .insert(Item)
    .insert(ProvidesDungeonMap{});
}

// player, enemies and tiles have position
fn despawn_all_with_position(
    mut commands: Commands, 
    position_q: Query<Entity, With<Position>>,
) {
    for e in position_q.iter() {
        commands.entity(e).despawn_recursive();
    }
}

// advance level requires to delete all entities, except the player their items
// set the field of view to dirty so it is re-calculated
fn advance_level(
    mut commands: Commands, 
    // player_q: Query<Entity, With<Player>>,
    position_q: Query<Entity, (With<Position>, Without<Player>)>,
    // items_q: Query<(Entity, &Carried)>,
    mut fov_q: Query<&mut FieldOfView>
) {

    // // get the player
    // let player = player_q.single();
    // // create a set to store the entities to keep, add player
    // let mut entities_to_keep = HashSet::default();
    // entities_to_keep.insert(player);
    
    // remove all the entities with position component except player
    for e in position_q.iter() {
        commands.entity(e).despawn_recursive();
    }

    // // save items carried by player
    // items_q.iter()
    //     .filter(|(_, carry)| carry.0 == player)
    //     .map(|(e, _)| e)
    //     .for_each(|e| {entities_to_keep.insert(e); });

    // set all the fov is_dirty to true, so they will need to be recalculated
    fov_q.iter_mut().for_each(|mut fov| fov.is_dirty = true);
}

pub struct SpawnerPlugin;
impl Plugin for SpawnerPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_system_set(
            SystemSet::on_exit(TurnState::StartScreen)
            .label("spawn_character")
            .with_system(spawn_player)
            .with_system(spawn_enemies)
            //.with_system(spawn_amulet_of_yala)
        )
        .add_system_set(
            SystemSet::on_enter(TurnState::GameOver)
            .label("despawn_all")
            .with_system(despawn_all_with_position)
        )
        .add_system_set(
            SystemSet::on_enter(TurnState::Victory)
            .label("despawn_all")
            .with_system(despawn_all_with_position)
        );
    }
}