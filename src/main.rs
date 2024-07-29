use bevy::{prelude::*, window::PrimaryWindow};
use bevy_kira_audio::prelude::*;
use rand::Rng;

const DHEIGHT: f32 = 500.;
const DWIDTH: f32 = 700.;
const IHEIGHT: f32 = 50.;
const DPADDING: f32 = 50.;

fn main() {
    let mut app = App::new();
    app.add_plugins((DefaultPlugins, AudioPlugin));
    app.insert_resource(Desktop {
        icons_amount: rand::thread_rng().gen_range(1..10),
        // icons_amount: 1,
        trash_pos: Vec2::new(
            -DWIDTH / 2. + DPADDING + IHEIGHT / 2.,
            -DHEIGHT / 2. + DPADDING + IHEIGHT / 2.,
        ),
    });
    app.add_systems(Startup, (spawn_camera, spawn_desktop));
    app.add_systems(Update, move_icon);

    app.run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

#[derive(Resource, Default)]
struct Desktop {
    icons_amount: usize,
    trash_pos: Vec2,
}

#[derive(Component)]
struct Pressed {
    offset: Vec2,
}

#[derive(Component)]
struct TrashBin;

#[derive(Component)]
struct Icon {
    id: usize,
}

fn spawn_desktop(mut commands: Commands, desktop: Res<Desktop>, asset_server: Res<AssetServer>) {
    commands.spawn(SpriteBundle {
        transform: Transform::from_xyz(0., 0., 0.),
        sprite: Sprite {
            color: Color::WHITE,
            custom_size: Some(Vec2::new(DWIDTH, DHEIGHT)),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut icons: Vec<Handle<Image>> = vec![];
    icons.push(asset_server.load("telegram.png"));
    icons.push(asset_server.load("youtube.png"));
    icons.push(asset_server.load("snapchat.png"));

    // 0 id is for trash bin reserved
    (0..desktop.icons_amount).for_each(|id| {
        let icon_id: usize = rand::thread_rng().gen_range(0..icons.len());
        commands.spawn((
            SpriteBundle {
                texture: icons[icon_id].clone(),
                transform: Transform::from_translation(Vec3::new(
                    rand::thread_rng().gen_range(-DWIDTH / 2.0 + DPADDING..DWIDTH / 2.0 - DPADDING),
                    rand::thread_rng()
                        .gen_range(-DHEIGHT / 2.0 + DPADDING..DHEIGHT / 2.0 - DPADDING),
                    1.,
                )),
                sprite: Sprite {
                    color: Color::WHITE,
                    custom_size: Some(Vec2::new(IHEIGHT, IHEIGHT)),
                    ..Default::default()
                },
                ..Default::default()
            },
            Icon { id },
        ));
    });

    // todo custom icon bundle?
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("trash-empty.png"),
            transform: Transform::from_translation(desktop.trash_pos.extend(1.)),
            sprite: Sprite {
                color: Color::srgb(0., 0., 0.4),
                custom_size: Some(Vec2::new(IHEIGHT, IHEIGHT)),
                ..Default::default()
            },
            ..Default::default()
        },
        TrashBin,
    ));
}

fn move_icon(
    icons_q: Query<(Entity, &Transform, &Icon), Without<Pressed>>,
    mut pressed_icon_q: Query<(Entity, &mut Transform, &Pressed), With<Icon>>,
    trash_q: Query<&Transform, (With<TrashBin>, Without<Icon>)>,
    window_q: Query<&Window, With<PrimaryWindow>>,
    audio: Res<Audio>,
    asset_server: Res<AssetServer>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut commands: Commands,
) {
    let window = window_q.single();
    if let Some(cursor_pos) = window.cursor_position() {
        // cursor position gives us the coordinates from the left up window, but the translation of
        // the icon gives us the distance from the center of the window
        let cursor_pos = cursor_pos - Vec2::new(window.width() / 2., window.height() / 2.);
        let cursor_pos = Vec2::new(cursor_pos.x, cursor_pos.y * -1.);

        if mouse.just_pressed(MouseButton::Left) {
            println!("Button just pressed");
            // the bool to stop the closure
            let mut pressed = false;

            icons_q
                .iter()
                .for_each(|(icon_entity, icon_transform, icon)| {
                    let collision_info =
                        check_cursor_in_icon(cursor_pos, icon_transform.translation.xy(), icon.id);
                    if collision_info.0 == true && !pressed {
                        commands.entity(icon_entity).insert(Pressed {
                            offset: collision_info.1,
                        });
                        audio.play(asset_server.load("click.ogg"));
                        pressed = true;
                    }
                })
        }

        if mouse.pressed(MouseButton::Left) {
            if let Ok((_, mut i_transform, pressed)) = pressed_icon_q.get_single_mut() {
                i_transform.translation = (cursor_pos - pressed.offset).extend(1.);
            }
        }

        if mouse.just_released(MouseButton::Left) {
            if let Ok((icon_entity, _, _)) = pressed_icon_q.get_single() {
                // check if cursor was on the trash, when the icon was dropped
                if check_cursor_in_icon(cursor_pos, trash_q.single().translation.xy(), 0).0 {
                    commands.entity(icon_entity).despawn();
                } else {
                    // just remove the pressed component
                    commands.entity(icon_entity).remove::<Pressed>();
                }
            }
        }
    }
}

// checks if the cursor position on the click was in the icon
// returns offset to center
fn check_cursor_in_icon(cursor_pos: Vec2, icon_center: Vec2, id: usize) -> (bool, Vec2) {
    // for now this is the collision with the circle with radius IHEIGHT/2 with center in icon_center
    let offset = cursor_pos - icon_center;
    if offset.length() <= IHEIGHT / 2. {
        println!("Icon {} was pressed.", id);
        return (true, offset);
    }

    (false, Vec2::new(0., 0.))
}
