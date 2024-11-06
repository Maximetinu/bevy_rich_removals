//! Demonstrates the motivational use case that led to the creation of this crate

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

#[derive(Component, Deref, DerefMut)]
struct Object(KeyCode);

#[derive(Component)]
struct Character;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0),
            RapierDebugRenderPlugin::default(),
        ))
        .add_systems(Startup, (setup_graphics, setup_character))
        .add_systems(
            Update,
            (
                spawn_object_at_cursor_on_key_pressed.run_if(no_objects_exist),
                move_object_to_cursor,
                despawn_object_on_key_released,
                display_events,
            ),
        )
        .run();
}

fn setup_graphics(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn setup_character(mut commands: Commands) {
    commands.spawn((
        Character,
        Name::new("Character"),
        TransformBundle::IDENTITY,
        Collider::cuboid(100., 100.),
        ActiveEvents::COLLISION_EVENTS,
        LockedAxes::TRANSLATION_LOCKED,
        RigidBody::Dynamic,
    ));
}

fn no_objects_exist(objects_q: Query<&Object>) -> bool {
    objects_q.is_empty()
}

fn spawn_object_at_cursor_on_key_pressed(
    mut commands: Commands,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let (camera, camera_transform) = camera_q.single();

    let Some(cursor_position) = windows.single().cursor_position() else {
        return;
    };

    let Some(point) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
        return;
    };

    for key in keys.get_just_pressed() {
        commands.spawn((
            Object(*key),
            Name::new(format!("{key:?}")),
            TransformBundle::from_transform(Transform::from_translation(point.extend(0.))),
            Collider::cuboid(100., 100.),
            Sensor,
        ));
        break;
    }
}

fn despawn_object_on_key_released(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    objects_q: Query<(Entity, &Object)>,
) {
    let Some((entity, object)) = objects_q.get_single().ok() else {
        return;
    };

    if !keys.pressed(**object) {
        commands.entity(entity).despawn();
    }
}

fn move_object_to_cursor(
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut objects_q: Query<&mut Transform, With<Object>>,
) {
    let (camera, camera_transform) = camera_q.single();

    let Some(cursor_position) = windows.single().cursor_position() else {
        return;
    };

    let Some(point) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
        return;
    };

    let Some(mut transform) = objects_q.get_single_mut().ok() else {
        return;
    };

    transform.translation = point.extend(0.);
}

pub fn display_events(
    mut collision_events: EventReader<CollisionEvent>,
    names_q: Query<(Entity, &Name)>,
) {
    for evt in collision_events.read() {
        let (entity1, entity2, started_or_stopped) = match evt {
            CollisionEvent::Started(entity1, entity2, _flags) => (entity1, entity2, "started"),
            CollisionEvent::Stopped(entity1, entity2, _flags) => (entity1, entity2, "stopped"),
        };
        if let (Ok((_, entity1_name)), Ok((_, entity2_name))) =
            (names_q.get(*entity1), names_q.get(*entity2))
        {
            println!("Collision between {entity1_name} and {entity2_name} {started_or_stopped}");
        } else {
            println!("ERROR! Some name was not found. Event discarded: {evt:?}")
        }
    }
}
