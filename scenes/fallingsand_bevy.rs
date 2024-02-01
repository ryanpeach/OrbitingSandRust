use bevy::diagnostic::FrameTimeDiagnosticsPlugin;

use bevy::sprite::MaterialMesh2dBundle;
use bevy::{core_pipeline::clear_color::ClearColorConfig, log::LogPlugin, prelude::*};
use bevy_egui::EguiPlugin;
use orbiting_sand::entities::camera::{move_camera_system, zoom_camera_system};
use orbiting_sand::entities::celestials::sun::SunBuilder;
use orbiting_sand::entities::celestials::{celestial::CelestialData, earthlike::EarthLikeBuilder};
use orbiting_sand::gui::brush::BrushRadius;
use orbiting_sand::gui::camera_window::camera_window_system;
use orbiting_sand::gui::element_picker::ElementSelection;
use orbiting_sand::physics::fallingsand::util::mesh::MeshBoundingBox;
use orbiting_sand::physics::orbits::components::{Mass, Velocity};
use orbiting_sand::physics::orbits::gravity_vis::GravityFieldPlugin;
use orbiting_sand::physics::orbits::nbody::NBodyPlugin;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(LogPlugin {
                    level: bevy::log::Level::INFO,
                    ..Default::default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(EguiPlugin)
        .add_plugins(NBodyPlugin)
        .add_plugins(GravityFieldPlugin)
        .add_plugins(FrameTimeDiagnosticsPlugin)
        .insert_resource(ElementSelection::default())
        .add_systems(Startup, setup)
        .add_systems(Update, (zoom_camera_system, move_camera_system))
        .add_systems(Update, CelestialData::process_system)
        .add_systems(Update, camera_window_system)
        .add_systems(Update, ElementSelection::element_picker_system)
        .add_systems(Update, frustum_culling_2d)
        .add_systems(
            Update,
            (
                BrushRadius::move_brush_system,
                BrushRadius::draw_brush_system,
                BrushRadius::resize_brush_system,
                BrushRadius::apply_brush_system,
            ),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Create a 2D camera
    let camera = commands
        .spawn(Camera2dBundle {
            camera_2d: Camera2d {
                clear_color: ClearColorConfig::Custom(Color::rgb(0.0, 0.0, 0.0)),
            },
            transform: Transform::from_scale(Vec3::new(1.0, 1.0, 1.0) * 100.0),
            ..Default::default()
        })
        .id();

    // Create the brush
    let brush = commands
        .spawn((
            BrushRadius(0.5),
            Transform::from_translation(Vec3::new(0., 0., 0.)),
        ))
        .id();

    // Parent the brush to the camera
    commands.entity(camera).push_children(&[brush]);

    // Create earth
    let planet_data = EarthLikeBuilder::new().build();
    CelestialData::setup(
        planet_data,
        Velocity(Vec2::new(0., 1200.)),
        Vec2::new(-10000., 0.),
        &mut commands,
        &mut meshes,
        &mut materials,
        &asset_server,
        true,
    );

    // Create earth2
    let planet_data = EarthLikeBuilder::new().build();
    CelestialData::setup(
        planet_data,
        Velocity(Vec2::new(0., -1200.)),
        Vec2::new(10000., 0.),
        &mut commands,
        &mut meshes,
        &mut materials,
        &asset_server,
        true,
    );

    // Create a sun
    let sun_data = SunBuilder::new().build();
    let sun_id = CelestialData::setup(
        sun_data,
        Velocity(Vec2::new(0., 0.)),
        Vec2::new(0., 0.),
        &mut commands,
        &mut meshes,
        &mut materials,
        &asset_server,
        true,
    );

    // Create a bunch of asteroids
    const NUM_ASTEROIDS: usize = 10000;
    for i in 0..NUM_ASTEROIDS {
        // Put them in a circle around the sun
        // at radius 5000 with a tangent velocity of 600
        let angle = (i as f32 / NUM_ASTEROIDS as f32) * 2.0 * std::f32::consts::PI;
        // random radius between 5000.0 and 6000.0
        let r = 5000.0 + 1000.0 * rand::random::<f32>();
        let pos = r * Vec2::new(angle.cos(), angle.sin());
        let vel = Vec2::new(angle.sin(), -angle.cos()) * 2000.0;
        commands.spawn((
            Velocity(vel),
            Mass(1.0),
            MaterialMesh2dBundle {
                mesh: meshes.add(shape::Circle::new(20.).into()).into(),
                material: materials.add(ColorMaterial::from(Color::PURPLE)),
                transform: Transform::from_translation(pos.extend(0.0)),
                ..default()
            },
        ));
    }

    // Parent the camera to the sun
    commands.entity(sun_id).push_children(&[camera]);
}

fn rect_overlaps(this: &Rect, other: &Rect) -> bool {
    this.min.x < other.max.x
        && this.max.x > other.min.x
        && this.min.y < other.max.y
        && this.max.y > other.min.y
}

fn rect_add(this: &Rect, other: &Vec2) -> Rect {
    Rect::new(
        this.min.x + other.x,
        this.min.y + other.y,
        this.max.x + other.x,
        this.max.y + other.y,
    )
}

fn frustum_culling_2d(
    mut commands: Commands,
    camera: Query<(&Camera2d, &GlobalTransform)>,
    mut mesh_entities: Query<(Entity, &MeshBoundingBox, &Visibility, &Transform)>,
    windows: Query<&Window>,
) {
    let (_, camera_transform) = camera.single();
    let camera_transform = camera_transform.compute_transform();
    let window = windows.single();

    let width = window.resolution.width();
    let height = window.resolution.height();

    // Get the camera rect in world coordinates using the translation and scale
    let camera_rect = Rect::new(
        camera_transform.translation.x,
        camera_transform.translation.y,
        width * camera_transform.scale.x,
        height * camera_transform.scale.y,
    );

    for (entity, mesh_bb, visible, transform) in mesh_entities.iter_mut() {
        let overlaps = rect_overlaps(
            &camera_rect,
            &rect_add(&mesh_bb.0, &transform.translation.truncate()),
        );
        if overlaps && *visible == Visibility::Hidden {
            commands.entity(entity).insert(Visibility::Visible);
        } else if !overlaps && *visible == Visibility::Visible {
            commands.entity(entity).insert(Visibility::Hidden);
        }
    }
}
