use std::time::{Duration, Instant};

use bevy::{
    diagnostic::FrameTimeDiagnosticsPlugin,
    prelude::*,
    render::render_resource::AsBindGroup,
    utils::hashbrown::{HashMap, HashSet},
};
use bevy_pancam::{PanCam, PanCamPlugin};

fn main() {
    let mut app = App::new();
    let start_cells = [
        (1, 2).into(),
        (2, 1).into(),
        (2, 0).into(),
        (1, 0).into(),
        (0, 0).into(),
    ]
    .into_iter()
    .collect();
    app.insert_resource(LivingCellLocations(start_cells));
    app.insert_resource(ClickedAt(None));
    app.insert_resource(ClearColor(Color::srgb(0.35, 0.48, 0.59)));
    app.add_plugins((DefaultPlugins, PanCamPlugin, FrameTimeDiagnosticsPlugin));
    app.add_systems(Startup, setup);
    app.add_systems(Update, q_to_quit);
    app.add_systems(Update, step);
    app.add_systems(Update, toggle);
    #[cfg(debug_assertions)] // debug/dev builds only
    {
        use bevy::diagnostic::LogDiagnosticsPlugin;
        app.add_plugins(LogDiagnosticsPlugin::default());
    }
    app.run();
}

const LIVE_SPRITE: &str = "sprites/live.png";
const BACKGROUND_SHADER: &str = "shaders/bg_shader.wgsl";

#[derive(Resource)]
struct ClickedAt(Option<Instant>);

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct BackgroundMaterial {
    #[uniform(0)]
    offset: [f32; 4],
    #[uniform(1)]
    grid_size: [f32; 4],
    #[texture(2)]
    #[sampler(3)]
    color_texture: Handle<Image>,
}

fn step(
    keycode: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut cells: ResMut<LivingCellLocations>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    query: Query<Entity, (With<Cell>, Without<Window>)>,
    asset_server: Res<AssetServer>,
) {
    if !keycode.just_pressed(KeyCode::Tab) {
        return;
    }

    let mut adjacency_rec: HashMap<Cell, u32> = HashMap::new();

    for i in &cells.0 {
        for j in get_adjacent(*i) {
            *adjacency_rec.entry(j).or_default() += 1;
        }
    }
    cells.0 = adjacency_rec
        .into_iter()
        .filter(|(coords, count)| alive_rules(*count, &cells.0, *coords))
        .map(|(coords, _)| coords)
        .collect();

    for i in &query {
        commands.entity(i).despawn();
    }
    let shapes = cells
        .0
        .iter()
        .map(|i| (meshes.add(Rectangle::new(50., 50.)), *i));
    for (shape, pos) in shapes {
        commands.spawn((
            Mesh2d(shape),
            MeshMaterial2d(materials.add(asset_server.load(LIVE_SPRITE))),
            Transform::from_xyz(pos.x as f32 * 50., pos.y as f32 * 50., 0.0),
            pos,
        ));
    }
}

#[inline]
fn alive_rules(count: u32, prev: &HashSet<Cell>, coords: Cell) -> bool {
    3 == count || (2 == count && prev.contains(&coords))
}

fn get_adjacent(coords: Cell) -> [Cell; 8] {
    [
        [coords.x - 1, coords.y - 1].into(),
        [coords.x - 1, coords.y + 1].into(),
        [coords.x - 1, coords.y].into(),
        [coords.x, coords.y - 1].into(),
        [coords.x, coords.y + 1].into(),
        [coords.x + 1, coords.y].into(),
        [coords.x + 1, coords.y - 1].into(),
        [coords.x + 1, coords.y + 1].into(),
    ]
}

fn q_to_quit(
    keycode: Res<ButtonInput<KeyCode>>,
    mut app_exit_events: ResMut<Events<bevy::app::AppExit>>,
) {
    if keycode.just_pressed(KeyCode::KeyQ) {
        app_exit_events.send(bevy::app::AppExit::Success);
    }
}

fn setup(
    mut commands: Commands,
    cells: Res<LivingCellLocations>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let shapes = cells
        .0
        .iter()
        .map(|i| (meshes.add(Rectangle::new(50., 50.)), *i));
    for (shape, pos) in shapes {
        commands.spawn((
            Mesh2d(shape),
            MeshMaterial2d(materials.add(asset_server.load(LIVE_SPRITE))),
            Transform::from_xyz(pos.x as f32 * 50., pos.y as f32 * 50., 0.0),
            pos,
        ));
    }
    commands.spawn((Camera2d, PanCam::default()));
}

fn toggle(
    mb: Res<ButtonInput<MouseButton>>,
    mut lastclicked: ResMut<ClickedAt>,
    cells_query: Query<(Entity, &Cell)>,
    mut commands: Commands,
    mut cells: ResMut<LivingCellLocations>,
    window: Single<&Window>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let (camera, camera_transform) = *camera_query;

    if mb.just_released(MouseButton::Left) {
        if lastclicked
            .0
            .is_some_and(|i| i.elapsed() < Duration::from_millis(150))
        {
            lastclicked.0 = None;
            let Some(cursor_position) = window.cursor_position() else {
                return;
            };
            let Ok(point) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
                return;
            };
            let cell: Cell = (
                (point.x / 50. + 0.5).floor() as i32,
                (point.y / 50. + 0.5).floor() as i32,
            )
                .into();
            if cells.0.contains(&cell) {
                if let Some((entity, _)) = cells_query.iter().find(|i| *i.1 == cell) {
                    commands.entity(entity).despawn();
                }
                cells.0.remove(&cell);
            } else {
                cells.0.insert(cell);
                let shape = meshes.add(Rectangle::new(50., 50.));
                commands.spawn((
                    Mesh2d(shape),
                    MeshMaterial2d(materials.add(asset_server.load(LIVE_SPRITE))),
                    Transform::from_xyz(cell.x as f32 * 50., cell.y as f32 * 50., 0.0),
                    cell,
                ));
            }
        } else {
            lastclicked.0 = None;
        }
    } else if mb.just_pressed(MouseButton::Left) {
        lastclicked.0 = Some(Instant::now());
    }
}

#[derive(Component, Clone, Copy, PartialEq, Hash, Eq)]
struct Cell {
    x: i32,
    y: i32,
}

impl From<(i32, i32)> for Cell {
    fn from(value: (i32, i32)) -> Self {
        Self {
            x: value.0,
            y: value.1,
        }
    }
}

impl From<[i32; 2]> for Cell {
    fn from(value: [i32; 2]) -> Self {
        Self {
            x: value[0],
            y: value[1],
        }
    }
}

#[derive(Resource)]
struct LivingCellLocations(HashSet<Cell>);
