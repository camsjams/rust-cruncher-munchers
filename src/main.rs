use bevy::{core::FixedTimestep, prelude::*, render::camera::Camera};
use rand::seq::SliceRandom;
use rand::Rng;

struct Cell {
    term: String,
}

struct ScoreText;

struct TermText {
    row: usize,
    col: usize,
}

#[derive(Default)]
struct Cruncher {
    entity: Option<Entity>,
    row: usize,
    col: usize,
}

#[derive(Default)]
struct Person {
    entity: Option<Entity>,
    row: usize,
    col: usize,
    handle: Handle<Scene>,
}

#[derive(Default)]
struct Game {
    board: Vec<Vec<Cell>>,
    cruncher: Cruncher,
    person: Person,
    score: i32,
    score_streak: i32,
}

#[derive(Clone, Eq, PartialEq, Debug)]
enum GameState {
    Crunching,
    GameOver,
    GameWin,
}

const BOARD_ROWS: usize = 6;
const BOARD_COLS: usize = 9;
const REQUIRED_CRUNCHES: usize = 15;

const TERMS: &[&str] = &[
    "frog", "pug", "tabby", "quail", "chimp", "whale", "newt", "goat", "eagle", "ferret", "crab",
    "koala",
];
const VALID_TERMS: &[&str] = &["pug", "tabby", "chimp", "goat", "ferret", "koala"];

fn main() {
    App::build()
        .insert_resource(Msaa { samples: 4 })
        .init_resource::<Game>()
        .add_plugins(DefaultPlugins)
        .add_state(GameState::Crunching)
        .add_startup_system(setup_cameras.system())
        .add_system_set(SystemSet::on_enter(GameState::Crunching).with_system(setup.system()))
        .add_system_set(
            SystemSet::on_update(GameState::Crunching)
                .with_system(move_player.system())
                .with_system(scoreboard_system.system())
                .with_system(term_system.system())
                .with_system(crunch_meter_system.system()),
        )
        .add_system_set(SystemSet::on_exit(GameState::Crunching).with_system(teardown.system()))
        .add_system_set(
            SystemSet::on_enter(GameState::GameOver).with_system(display_final_score.system()),
        )
        .add_system_set(
            SystemSet::on_update(GameState::GameOver).with_system(game_over_keyboard.system()),
        )
        .add_system_set(SystemSet::on_exit(GameState::GameOver).with_system(teardown.system()))
        .add_system_set(
            SystemSet::on_enter(GameState::GameWin).with_system(display_winning_score.system()),
        )
        .add_system_set(
            SystemSet::on_update(GameState::GameWin).with_system(game_over_keyboard.system()),
        )
        .add_system_set(SystemSet::on_exit(GameState::GameWin).with_system(teardown.system()))
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(4.0))
                .with_system(spawn_or_move_person.system()),
        )
        .run();
}

fn setup_cameras(mut commands: Commands) {
    let mut camera = OrthographicCameraBundle::new_3d();
    camera.orthographic_projection.scale = 4.8;
    camera.transform =
        Transform::from_xyz(2.7, 3.0, 0.0).looking_at(Vec3::new(3.0, 2.0, 0.0), Vec3::Y);
    commands.spawn(camera).spawn(UiCameraBundle::default());
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, mut game: ResMut<Game>) {
    game.score = 0;
    game.score_streak = 0;
    game.cruncher.row = BOARD_ROWS / 2;
    game.cruncher.col = BOARD_COLS / 2;

    commands.spawn(LightBundle {
        transform: Transform::from_xyz(4.0, 5.0, 4.0),
        ..Default::default()
    });

    // spawn the tiles
    let cell_scene = asset_server.load("models/platform.glb#Scene0");
    game.board = (0..BOARD_COLS)
        .map(|col| {
            (0..BOARD_ROWS)
                .map(|row| {
                    let term = TERMS
                        .choose(&mut rand::thread_rng())
                        .unwrap_or(&"frog")
                        .to_string();
                    commands
                        .spawn((
                            Transform::from_xyz(row as f32, 0.0, col as f32),
                            GlobalTransform::identity(),
                        ))
                        .with_children(|cell| {
                            cell.spawn_scene(cell_scene.clone());
                        });

                    commands
                        .spawn(TextBundle {
                            text: Text::with_section(
                                format!("{}", term),
                                TextStyle {
                                    font: asset_server.load("fonts/SourceCodePro-Regular.ttf"),
                                    font_size: 20.0,
                                    color: Color::rgb(0.9, 0.9, 0.9),
                                },
                                TextAlignment {
                                    horizontal: HorizontalAlign::Center,
                                    ..Default::default()
                                },
                            ),
                            style: Style {
                                position_type: PositionType::Absolute,
                                position: Rect {
                                    left: Val::Percent(48.0 + 6.0 * col as f32),
                                    top: Val::Percent(35.0 + 11.0 * row as f32 - 2.5),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            ..Default::default()
                        })
                        .with(TermText { row, col });

                    Cell {
                        term: term.to_string(),
                    }
                })
                .collect()
        })
        .collect();

    // spawn the cruncher character
    game.cruncher.entity = commands
        .spawn((
            Transform {
                translation: Vec3::new(game.cruncher.row as f32, 0.0, game.cruncher.col as f32),
                rotation: Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2),
                ..Default::default()
            },
            GlobalTransform::identity(),
        ))
        .with_children(|cell| {
            cell.spawn_scene(asset_server.load("models/cruncher.glb#Scene0"));
        })
        .current_entity();

    // show the score and crunch meter
    commands
        .spawn(TextBundle {
            text: Text {
                sections: vec![
                    TextSection {
                        value: "Score: ".to_string(),
                        style: TextStyle {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 40.0,
                            color: Color::rgb(0.0, 1.0, 0.0),
                        },
                    },
                    TextSection {
                        value: "".to_string(),
                        style: TextStyle {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 40.0,
                            color: Color::rgb(0.0, 1.0, 1.0),
                        },
                    },
                    TextSection {
                        value: "\nCrunch Meter: ".to_string(),
                        style: TextStyle {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 40.0,
                            color: Color::rgb(0.0, 1.0, 0.0),
                        },
                    },
                    TextSection {
                        value: "".to_string(),
                        style: TextStyle {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 40.0,
                            color: Color::rgb(0.0, 1.0, 1.0),
                        },
                    },
                ],
                ..Default::default()
            },
            style: Style {
                position_type: PositionType::Absolute,
                position: Rect {
                    top: Val::Px(5.0),
                    left: Val::Px(5.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        })
        .with(ScoreText);

    // show the prompt
    commands.spawn(TextBundle {
        text: Text {
            sections: vec![TextSection {
                value: "Animals with fur".to_string(),
                style: TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 40.0,
                    color: Color::rgb(0.0, 0.0, 0.0),
                },
            }],
            ..Default::default()
        },
        style: Style {
            position_type: PositionType::Absolute,
            position: Rect {
                top: Val::Percent(15.0),
                left: Val::Percent(50.0),
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    });

    // load the scene for the cake
    game.person.handle = asset_server.load("models/bob.glb#Scene0");
}

// control the game character
fn move_player(
    mut state: ResMut<State<GameState>>,
    mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>,
    mut game: ResMut<Game>,
    mut transforms: Query<&mut Transform>,
) {
    let mut moved = false;
    let mut rotation = 0.0;
    if keyboard_input.just_pressed(KeyCode::Up) {
        if game.cruncher.row < BOARD_ROWS - 1 {
            game.cruncher.row += 1;
        }
        rotation = -std::f32::consts::FRAC_PI_2;
        moved = true;
    }
    if keyboard_input.just_pressed(KeyCode::Down) {
        if game.cruncher.row > 0 {
            game.cruncher.row -= 1;
        }
        rotation = std::f32::consts::FRAC_PI_2;
        moved = true;
    }
    if keyboard_input.just_pressed(KeyCode::Right) {
        if game.cruncher.col < BOARD_COLS - 1 {
            game.cruncher.col += 1;
        }
        rotation = std::f32::consts::PI;
        moved = true;
    }
    if keyboard_input.just_pressed(KeyCode::Left) {
        if game.cruncher.col > 0 {
            game.cruncher.col -= 1;
        }
        rotation = 0.0;
        moved = true;
    }
    if keyboard_input.just_pressed(KeyCode::Space) {
        let col = game.cruncher.col.clone();
        let row = BOARD_ROWS - 1 - game.cruncher.row.clone();
        let term = &game.board[col][row].term;
        let is_valid = VALID_TERMS.contains(&&term[..]);
        if term == "" {
            return;
        }

        if is_valid {
            game.score += 1;
            game.score_streak += 1;

            if game.score_streak >= REQUIRED_CRUNCHES as i32 {
                game.person.entity = None;
                state.set_next(GameState::GameWin).unwrap();
            }

            game.board[col][row].term = "".to_string();
        }

        if !is_valid {
            game.score_streak = 0;
        }
    }

    // move on the board
    if moved {
        *transforms.get_mut(game.cruncher.entity.unwrap()).unwrap() = Transform {
            translation: Vec3::new(game.cruncher.row as f32, 0.0, game.cruncher.col as f32),
            rotation: Quat::from_rotation_y(rotation),
            ..Default::default()
        };
    }

    // detect capture by astronaut person - Game Over
    if let Some(entity) = game.person.entity {
        if game.cruncher.row == game.person.row && game.cruncher.col == game.person.col {
            commands.despawn_recursive(entity);
            game.person.entity = None;
            state.set_next(GameState::GameOver).unwrap();
        }
    }
}

// update the score displayed during the game
fn scoreboard_system(game: Res<Game>, mut query: Query<&mut Text, With<ScoreText>>) {
    let mut text = query.single_mut().unwrap();
    text.sections[1].value = format!("{}", game.score);
}

// update the crunch meter displayed during the game
fn crunch_meter_system(game: Res<Game>, mut query: Query<&mut Text, With<ScoreText>>) {
    let mut text = query.single_mut().unwrap();
    text.sections[3].value = format!("{}/{}", game.score_streak, REQUIRED_CRUNCHES);
}

// remove all entities that are not a camera
fn teardown(mut commands: Commands, entities: Query<Entity, Without<Camera>>) {
    for entity in entities.iter() {
        commands.despawn_recursive(entity);
    }
}

fn spawn_or_move_person(
    state: ResMut<State<GameState>>,
    mut commands: Commands,
    mut game: ResMut<Game>,
    mut transforms: Query<&mut Transform>,
) {
    if *state.current() != GameState::Crunching {
        return;
    }
    if let Some(entity) = game.person.entity {
        let mut moved = false;
        let mut rotation = 0.0;
        let move_direction = rand::thread_rng().gen_range(0..3);
        match move_direction {
            0 => {
                if game.person.row < BOARD_ROWS - 1 {
                    game.person.row += 1;
                    rotation = -std::f32::consts::FRAC_PI_2;
                    moved = true;
                }
            }
            1 => {
                if game.person.row > 0 {
                    game.person.row -= 1;
                    rotation = std::f32::consts::FRAC_PI_2;
                    moved = true;
                }
            }
            2 => {
                if game.person.col < BOARD_COLS - 1 {
                    game.person.col += 1;
                    rotation = std::f32::consts::PI;
                    moved = true;
                }
            }
            3 => {
                if game.person.col > 0 {
                    game.person.col -= 1;
                    rotation = 0.0;
                    moved = true;
                }
            }
            _ => {}
        }

        if moved {
            let col = game.person.col.clone();
            let row = BOARD_ROWS - 1 - game.person.row.clone();
            game.board[col][row].term = "".to_string();
            *transforms.get_mut(entity).unwrap() = Transform {
                translation: Vec3::new(game.person.row as f32, 0.0, game.person.col as f32),
                rotation: Quat::from_rotation_y(rotation),
                ..Default::default()
            };
        }

        return;
    }

    game.person.row = rand::thread_rng().gen_range(0..BOARD_ROWS);
    game.person.col = 0;
    game.person.entity = commands
        .spawn((
            Transform {
                translation: Vec3::new(game.person.row as f32, 0.0, game.person.col as f32),
                ..Default::default()
            },
            GlobalTransform::identity(),
        ))
        .with_children(|cell| {
            cell.spawn_scene(game.person.handle.clone());
        })
        .current_entity();
}

// display the number of crunches before losing
fn display_final_score(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    game: Res<Game>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands
        .spawn(NodeBundle {
            style: Style {
                margin: Rect::all(Val::Auto),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            material: materials.add(Color::NONE.into()),
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn(TextBundle {
                text: Text {
                    sections: vec![
                        TextSection {
                            value: format!("Final Score: {}", game.score),
                            style: TextStyle {
                                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                font_size: 40.0,
                                color: Color::rgb(0.0, 1.0, 0.0),
                            },
                        },
                        TextSection {
                            value: "\nPress Enter to Try Again".to_string(),
                            style: TextStyle {
                                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                font_size: 40.0,
                                color: Color::rgb(0.0, 1.0, 0.0),
                            },
                        },
                    ],
                    ..Default::default()
                },
                ..Default::default()
            });
        });
}

// display the number of crunches after winning
fn display_winning_score(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    game: Res<Game>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands
        .spawn(NodeBundle {
            style: Style {
                margin: Rect::all(Val::Auto),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            material: materials.add(Color::NONE.into()),
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn(TextBundle {
                text: Text {
                    sections: vec![
                        TextSection {
                            value: format!("Final Score: {}", game.score),
                            style: TextStyle {
                                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                font_size: 40.0,
                                color: Color::rgb(0.0, 1.0, 0.0),
                            },
                        },
                        TextSection {
                            value: "\nYou Won! Press Enter to Play Again".to_string(),
                            style: TextStyle {
                                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                font_size: 40.0,
                                color: Color::rgb(0.0, 1.0, 0.0),
                            },
                        },
                    ],
                    ..Default::default()
                },
                ..Default::default()
            });
        });
}

// restart the game when pressing spacebar
fn game_over_keyboard(mut state: ResMut<State<GameState>>, keyboard_input: Res<Input<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::Return) {
        state.set_next(GameState::Crunching).unwrap();
    }
}

fn term_system(game: Res<Game>, mut query: Query<(&mut Text, &TermText)>) {
    for (mut text, term) in query.iter_mut() {
        let term = &game.board[term.col][term.row].term;
        text.sections[0].value = format!("{}", term);
    }
}
