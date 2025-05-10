use bevy::{image::ImageSamplerDescriptor, prelude::*};

#[derive(Component)]
struct Dialogue {
    name: String,
    dialogue: String,
    responses: Vec<String>,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin {
            default_sampler: ImageSamplerDescriptor::nearest(),
        }))
        .add_systems(Startup, (setup, setup_ui))
        .add_systems(Update, exit_on_esc)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/VT323-Regular.ttf");
    let image = asset_server.load("nate.png");

    let name = "Nate";
    let dialogue = "This is what Nate is saying. He's saying lots of things but not realy meaning any of it. This is all just to take up space while I work on programming the layouts.";
    let responses = vec!["yes, yes!", "a partridge in a pear tree?", "goodbye!"];

    let padding = Val::Px(10.0);

    commands
        .spawn((
            Name::new("Conversation Window"),
            Node {
                display: Display::Grid,
                width: Val::Px(800.0),
                height: Val::Px(400.0),
                grid_template_columns: vec![GridTrack::auto()],
                row_gap: padding,
                column_gap: padding,
                padding: UiRect::all(padding),
                ..default()
            },
            BackgroundColor(Color::srgb(0.0, 0.3, 0.7)),
        ))
        .with_children(|builder| {
            builder.spawn((
                Name::new("Header"),
                Text::new("Conversation"),
                TextFont {
                    font: font.clone(),
                    ..default()
                },
                TextColor(Color::BLACK),
                TextLayout {
                    justify: JustifyText::Center,
                    ..default()
                },
            ));
            builder
                .spawn((
                    Name::new("NPC Row"),
                    Node {
                        display: Display::Grid,
                        grid_template_columns: vec![
                            GridTrack::percent(50.0),
                            GridTrack::min_content(),
                        ],
                        column_gap: padding,
                        ..default()
                    },
                ))
                .with_children(|builder| {
                    builder
                        .spawn((
                            Name::new("dialogue box"),
                            Node {
                                display: Display::Grid,
                                grid_template_rows: vec![
                                    GridTrack::min_content(),
                                    GridTrack::auto(),
                                ],
                                ..default()
                            },
                            BackgroundColor(Color::hsl(0.0, 0.0, 0.1)),
                        ))
                        .with_children(|builder| {
                            builder.spawn((
                                Text::new(name),
                                TextFont {
                                    font: font.clone(),
                                    ..default()
                                },
                                TextLayout {
                                    justify: JustifyText::Center,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                            builder.spawn((
                                Text::new(dialogue),
                                TextFont {
                                    font: font.clone(),
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });
                    builder
                        .spawn((Name::new("Image container"), Node { ..default() }))
                        .with_children(|builder| {
                            builder.spawn(ImageNode { image, ..default() });
                        });
                });
            builder
                .spawn((
                    Name::new("Responses"),
                    Node {
                        display: Display::Grid,
                        width: Val::Percent(100.0),
                        height: Val::Auto,
                        grid_template_columns: vec![GridTrack::auto()],
                        ..default()
                    },
                    BackgroundColor(Color::hsl(0.0, 0.0, 0.1)),
                ))
                .with_children(|builder| {
                    responses.into_iter().for_each(|response| {
                        builder.spawn((
                            Text::new(response),
                            TextFont {
                                font: font.clone(),
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    })
                });
        });
}

fn exit_on_esc(keyboard_input: Res<ButtonInput<KeyCode>>, mut exit: EventWriter<AppExit>) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
    }
}
