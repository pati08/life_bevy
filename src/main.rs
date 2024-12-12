use bevy::{
    prelude::*,
    utils::hashbrown::{HashMap, HashSet},
};

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
    app.add_plugins(DefaultPlugins);
    app.add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin);
    app.add_systems(Startup, setup);
    app.add_systems(Update, q_to_quit);
    #[cfg(debug_assertions)] // debug/dev builds only
    {
        use bevy::diagnostic::LogDiagnosticsPlugin;
        app.add_plugins(LogDiagnosticsPlugin::default());
    }
    app.run();
}

fn step(
    mut commands: Commands,
    mut cells: ResMut<LivingCellLocations>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut adjacency_rec: HashMap<Cell, u32> = HashMap::new();

    for i in &cells.0 {
        for j in get_adjacent(*i) {
            *adjacency_rec.entry(j).or_default() += 1;
        }
    }
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
) {
    let shapes = cells
        .0
        .iter()
        .map(|i| (meshes.add(Rectangle::new(50., 50.)), *i));
    for (shape, pos) in shapes {
        let color = Color::hsl(119., 0.68, 0.67);

        commands.spawn((
            Mesh2d(shape),
            MeshMaterial2d(materials.add(color)),
            Transform::from_xyz(pos.x as f32 * 50., pos.y as f32 * 50., 0.0),
            pos,
        ));
    }
    commands.spawn(Camera2d);
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
