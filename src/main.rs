use std::{collections::LinkedList, time::Instant};
use bevy::{prelude::*, render::{camera::{Camera, PerspectiveProjection}, mesh::Indices, pipeline::PrimitiveTopology}};
use rand::Rng;
use voronoice::*;

mod into_triangle_list;
mod utils;
mod voronoi_mesh_generator;
mod voronoi_cell_mesh_generator;

use voronoi_mesh_generator::*;
use voronoi_cell_mesh_generator::VoronoiCellMeshGenerator;

const STRING_UI_COUNT: usize = 8;

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::rgb(0., 0., 0.))) //background
        .add_startup_system(setup.system())
        .add_system(calculate_mouse_world_coords.system())
        .add_system(handle_input.system())
        .add_system(move_camera.system())
        .run();
}

fn color_white(_i: usize) -> Color {
    Color::WHITE
}

fn color_red(_i: usize) -> Color {
    Color::RED
}

struct VoronoiMeshOptions {
    voronoi_topoloy: PrimitiveTopology,
    delauney_topoloy: PrimitiveTopology,
}

impl Default for VoronoiMeshOptions {
    fn default() -> Self {
        VoronoiMeshOptions {
            voronoi_topoloy: PrimitiveTopology::LineList,
            delauney_topoloy: PrimitiveTopology::LineList
        }
    }
}

struct Object;

fn spawn_voronoi(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, voronoi: &Voronoi, options: &VoronoiMeshOptions) {
    let start = Instant::now();
    let voronoi_generator = VoronoiMeshGenerator { voronoi: &voronoi, coloring: color_red, topology: options.voronoi_topoloy };
    let triangle_generator = VoronoiMeshGenerator { voronoi: &voronoi, coloring: color_white, topology: options.delauney_topoloy };

    commands
        .spawn_bundle(
            PbrBundle {
                mesh: meshes.add(voronoi_generator.build_voronoi_mesh()),
                transform: Transform::from_translation(Vec3::new(
                    0.0,
                    0.0,
                    0.0,
                )),
                ..Default::default()
            })
            .insert(Object);

    commands
        .spawn_bundle(
            PbrBundle {
                    mesh: meshes.add(triangle_generator.build_delauney_mesh()),
                    transform: Transform::from_translation(Vec3::new(
                        0.0,
                        0.0,
                        0.0,
                    )),
                    ..Default::default()
        })
        .insert(Object);

    println!("Generated new voronoi meshes in {:?}", start.elapsed());
}

struct DisplayVoronoiCell;

fn spawn_voronoi_cell(commands: &mut Commands, meshes: &mut ResMut<Assets<Mesh>>, cell: &VoronoiCell) {
    let mesh_generator = VoronoiCellMeshGenerator {
        cell: cell,
        coloring: color_red
    };

    commands
        .spawn_bundle(
            PbrBundle {
                mesh: meshes.add(mesh_generator.build_voronoi_mesh()),
                transform: Transform::from_translation(Vec3::new(
                    0.0,
                    0.0,
                    0.0,
                )),
                ..Default::default()
        })
        .insert(DisplayVoronoiCell)
        .insert(Object);
}

const CAMERA_Y: f32 = 6.0;
struct StatusDisplay;

fn add_display_lines(commands: &mut ChildBuilder, font: Handle<Font>) {
    commands.spawn_bundle(TextBundle {
        style: Style {
            size: Size::new(Val::Px(500.0), Val::Px(40.0)),
            ..Default::default()
        },
        text: Text::with_section(
            "",
            TextStyle {
                font_size: 25.0,
                color: Color::WHITE,
                font: font,
                ..Default::default()
            },
            TextAlignment::default()),
        ..Default::default()
    })
    .insert(StatusDisplay);
}

// right hand
// triangulation anti-clockwise
fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let camera_pos = Vec3::new(0.000001, CAMERA_Y, 0.0);
    let mut camera_t = Transform::from_translation(camera_pos)
        .looking_at(Vec3::default(), Vec3::Y);
    // roll camera so Z point up, and X right
    camera_t.rotate(Quat::from_rotation_ypr(0.0, 0.0, 180f32.to_radians()));

    let font_handle = asset_server.load("fonts/FiraSans-Bold.ttf");
    let font_handle2 = font_handle.clone();
    commands.spawn_bundle(NodeBundle{
        style: Style {
            size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
            align_content: AlignContent::Center,
            flex_direction: FlexDirection::ColumnReverse,
            ..Default::default()
        },
        material: materials.add(Color::NONE.into()),
        ..Default::default()
    }).with_children(|mut parent| {
        let font = font_handle2;
        for _i in 0..STRING_UI_COUNT {
            add_display_lines(&mut parent, font.clone());
        }
    });

    // ui camera
    commands.spawn_bundle(UiCameraBundle::default());

    // camera
    commands.spawn_bundle(PerspectiveCameraBundle {
            transform: camera_t,
            ..Default::default()
        });

    commands.spawn_bundle(TextBundle {
            style: Style {
                position_type: PositionType::Absolute,
                ..Default::default()
            },
            text: Text::with_section(
                    "(0, 0)".to_string(),
                TextStyle {
                        font: font_handle,
                        font_size: 25.0,
                        color: Color::WHITE,
                        ..Default::default()
                    },
                    TextAlignment::default()),
            ..Default::default()
        })
        .insert(Mouse::default());

    commands.spawn_bundle(PbrBundle {
            mesh: meshes.add(get_bounding_box(2.0)),
            ..Default::default()
        })
        .insert(BoundingBox::new_centered_square(43.0)) // this value does not matter
        .insert(Object);
}

fn get_bounding_box(size: f32) -> Mesh {
    let edge = size / 2.0;
    let pos = vec![
        [-edge, 0.0, -edge], // bottom left
        [-edge, 0.0, edge], // bottom right
        [edge, 0.0, edge], // top right
        [edge, 0.0, -edge], // top left
    ];

    let mut m = Mesh::new(PrimitiveTopology::LineStrip);
    m.set_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 1.0, 0.0]; pos.len()]);
    m.set_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 0.0]; pos.len()]);
    m.set_attribute(Mesh::ATTRIBUTE_POSITION, pos);
    m.set_indices(Some(Indices::U32(vec![0, 1, 2, 3, 0])));

    m
}

fn get_closest_site(voronoi: &Voronoi, pos: Vec3) -> Option<(usize, f32)> {
    voronoi.sites().iter().enumerate().map(|(i, p)| (i, Vec3::new(p.y as f32, 0.0, p.x as f32).distance(pos)))
        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
}

#[derive(Default)]
#[derive(Debug)]
struct Mouse {
    world_pos : Vec3
}
const MOUSE_TEXT_OFFSET: f32 = 15.0;
fn calculate_mouse_world_coords(mut mouse_query: Query<(&mut Mouse, &mut Text, &mut Style)>, query: Query<(&Transform, &Camera), With<PerspectiveProjection>>, windows: Res<Windows>) {
    let (mut mouse,  mut text, mut text_style) = mouse_query.iter_mut().next().unwrap();

    for ((camera_transform, camera), window) in query.iter().zip(windows.iter()) {
        let screen_size = Vec2::from([window.width() as f32, window.height() as f32]);
        let cursor_screen_pos = window.cursor_position().unwrap_or(Vec2::ZERO);

        // normalize cursor coords (-1 to 1)
        let cursor_pos_normalized = (2.0 * (cursor_screen_pos / screen_size) - Vec2::new(1.0, 1.0)).extend(1.0);
        let view_matrix = camera_transform.compute_matrix();
        let screen_normal_coords_to_world = view_matrix * camera.projection_matrix.inverse();

        let cursor_world_pos = screen_normal_coords_to_world.transform_point3(cursor_pos_normalized);
        let ray: Vec3 = cursor_world_pos - camera_transform.translation;

        // FIXME I put this together to debug voronoi generator
        // this is assuming camera looking down Y
        // genealize this ray-plane intersection logic based on the camera forward vector
        let mut world_pos = -camera_transform.translation.y * (ray / ray.y);
        world_pos.y = 0.0;
        mouse.world_pos = world_pos;
        text.sections[0].value = format!("({:.2}, {:.2})", mouse.world_pos.z, mouse.world_pos.x);

        text_style.position.left = Val::Px(cursor_screen_pos.x + MOUSE_TEXT_OFFSET);
        text_style.position.top = Val::Px(window.height() - cursor_screen_pos.y + MOUSE_TEXT_OFFSET);
    }
}

fn move_camera(input: Res<Input<KeyCode>>, mut camera_query: Query<&mut Transform, (With<Camera>, With<PerspectiveProjection>)>) {
    if input.pressed(KeyCode::W) {
        for mut t in camera_query.iter_mut() {
            let y_move = 0.1f32.min((t.translation.y - 0.7).powf(10.0));
            t.translation.y -= y_move;
        }
    } else if input.pressed(KeyCode::S) {
        for mut t in camera_query.iter_mut() {
            t.translation.y += 0.1;
        }
    } else if input.pressed(KeyCode::R) {
        for mut t in camera_query.iter_mut() {
            t.translation.y = CAMERA_Y;
        }
    }
}

fn create_random_sites(size: usize, bounding_box: &BoundingBox) -> Vec<Point> {
    let mut rng = rand::thread_rng();
    let x_range = rand::distributions::Uniform::new(-bounding_box.width(), bounding_box.width());
    let y_range = rand::distributions::Uniform::new(-bounding_box.height(), bounding_box.height());

    (0..size)
        .map(|_| Point { x: rng.sample(x_range), y: rng.sample(y_range) })
        .collect()
}

#[derive(Debug)]
enum SiteType {
    Random,
    Circle,
    Square
}
impl Default for SiteType {
    fn default() -> Self {
        SiteType::Random
    }
}
#[derive(Default)]
struct State {
    voronoi_opts: VoronoiMeshOptions,
    voronoi: Option<Voronoi>,
    clip_behavior: ClipBehavior,
    size: usize,
    undo_list: LinkedList<Voronoi>,
    forward_list: LinkedList<Voronoi>,
    bounding_box: BoundingBox,
    site_type: SiteType,
    show_boundingbox: bool,
    path_start_site: Option<usize>,
    path_end_site: Option<usize>,
}
impl State {
    fn replace(&mut self, v: Option<Voronoi>) -> Option<&Voronoi> {
        let old = if let Some(new) = v  {
            self.voronoi.replace(new)
        } else {
            self.voronoi.take()
        };

        if let Some(old) = old {
            self.undo_list.push_front(old);

            self.undo_list.front()
        } else {
            None
        }
    }

    fn undo(&mut self) -> Option<&Voronoi> {
        if let Some(prev) = self.undo_list.pop_front() {
            if let Some(curr) = self.voronoi.replace(prev) {
                self.forward_list.push_front(curr);
                self.forward_list.front()
            } else {
                None
            }
        } else {
            None
        }
    }

    fn undo_forward(&mut self) -> Option<&Voronoi> {
        if let Some(prev) = self.forward_list.pop_front() {
            if let Some(curr) = self.voronoi.replace(prev) {
                self.undo_list.push_front(curr);
                self.undo_list.front()
            } else {
                None
            }
        } else {
            None
        }
    }

    fn clear(&mut self) {
        self.voronoi.take();
        self.undo_list.clear();
        self.forward_list.clear();
        self.bounding_box = BoundingBox::new_centered_square(2.0);
        self.show_boundingbox = false;
        self.path_start_site = None;
        self.path_end_site = None;
    }

    fn new_builder(&self) -> VoronoiBuilder {
        VoronoiBuilder::default()
            .set_bounding_box(self.bounding_box.clone())
    }

    fn new_voronoi(&mut self, size: usize) {
        let start = Instant::now();
        self.size = size;
        let mut builder = self.new_builder();

        builder = match self.site_type {
            SiteType::Random => builder.set_sites(create_random_sites(size, &self.bounding_box)),
            SiteType::Circle => builder.generate_circle_sites(self.size, 1.0),
            SiteType::Square => builder.generate_square_sites(self.size),
        };

        let voronoi = builder.build();
        println!("Generated new voronoi of size {} in {:?}", self.size, start.elapsed());

        self.replace(voronoi);
    }

    fn refresh(&mut self) {
        if let Some(v) = self.voronoi.as_ref() {
            let vv = self.new_builder()
                .set_sites(v.sites().clone())
                .build();
            self.replace(vv);
        }
    }

    fn add_site_to_voronoi(&mut self, site: Point) {
        let mut sites = self.voronoi.as_ref().unwrap().sites().clone();
        sites.push(site);

        let v = self.new_builder()
            .set_sites(sites)
            .build();
        self.replace(v);
    }

    fn remove_site_to_voronoi(&mut self, site_index: usize) {
        let mut sites = self.voronoi.as_ref().unwrap().sites().clone();
        sites.remove(site_index);

        let v = self.new_builder()
            .set_sites(sites)
            .build();
        self.replace(v);
    }
}

fn handle_input(
    mut state: Local<State>,
    input: Res<Input<KeyCode>>,
    mouse_button_input: Res<Input<MouseButton>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
    query: Query<Entity, With<Object>>,
    mut query_text: Query<&mut Text, With<StatusDisplay>>,
    mut query_box: Query<(&mut Transform, &mut Visible), With<BoundingBox>>,
    query_path: Query<Entity, With<DisplayVoronoiCell>>,
    mouse_query: Query<&Mouse>) {

    let mut respawn = false;

    // no voronoi, generate random one
    if !state.voronoi.is_some() && state.undo_list.is_empty() {
        respawn = true;
        state.clear();
        state.bounding_box = BoundingBox::new_centered_square(2.0);
        state.new_voronoi(20);
    }

    if input.just_pressed(KeyCode::PageUp) || input.just_pressed(KeyCode::PageDown) {
        let increment = if input.pressed(KeyCode::LShift) {
            1.0
        } else {
            0.1
        };

        if input.just_pressed(KeyCode::PageUp) {
            let size = state.bounding_box.width() + increment;
            state.bounding_box = BoundingBox::new(state.bounding_box.center().clone(), size, size);
        } else if input.just_pressed(KeyCode::PageDown) {
            let size = state.bounding_box.width().max(increment + 0.1) -increment;
            state.bounding_box = BoundingBox::new(state.bounding_box.center().clone(), size, size);
        }

        respawn = true;
        state.refresh();
    } else if input.just_pressed(KeyCode::V) {
        state.show_boundingbox = !state.show_boundingbox;
    }

    for (mut box_t, mut visible) in query_box.iter_mut() {
        box_t.scale = Vec3::splat((state.bounding_box.width() / 2.0) as f32);
        visible.is_visible = state.show_boundingbox;
    }

    // span new voronoi with new rendering but same points
    if input.just_pressed(KeyCode::P) {
        let options = &mut state.voronoi_opts;
        options.voronoi_topoloy = match options.voronoi_topoloy {
            PrimitiveTopology::TriangleList => PrimitiveTopology::LineList,
            PrimitiveTopology::LineList => PrimitiveTopology::PointList,
            _ => PrimitiveTopology::TriangleList,
        };

        respawn = true;
    } else if input.just_pressed(KeyCode::O) {
        let options = &mut state.voronoi_opts;
        options.delauney_topoloy = match options.delauney_topoloy {
            PrimitiveTopology::TriangleList => PrimitiveTopology::LineList,
            PrimitiveTopology::LineList => PrimitiveTopology::PointList,
            _ => PrimitiveTopology::TriangleList,
        };

        respawn = true;
    } else if input.pressed(KeyCode::L) {
        // run loyd relaxation
        if let Some(existing_voronoi) = state.voronoi.as_ref() {
            let builder: VoronoiBuilder = existing_voronoi.into();
            state.replace(builder.set_lloyd_relaxation_iterations(1).build());
            respawn = true;
        }
    } else if input.just_pressed(KeyCode::C) {
        // change hull behavior
        state.clip_behavior = match state.clip_behavior {
            ClipBehavior::Clip => ClipBehavior::None,
            ClipBehavior::None => ClipBehavior::RemoveSitesOutsideBoundingBoxOnly,
            ClipBehavior::RemoveSitesOutsideBoundingBoxOnly => ClipBehavior::Clip
        };
        println!("Clip behavior set to {:?}", state.clip_behavior);

        state.refresh();
        respawn = true;
    }

    let mouse = mouse_query.iter().next().unwrap();
    if mouse_button_input.just_pressed(MouseButton::Left) || mouse_button_input.just_pressed(MouseButton::Right) || mouse_button_input.just_pressed(MouseButton::Middle) {
        // take sites and change based on type of click
        let point = Point { x: mouse.world_pos.z as f64, y: mouse.world_pos.x  as f64 };

        let (closest_site, num_of_sites) = if let Some(voronoi) = state.voronoi.as_ref() {
            (get_closest_site(voronoi, mouse.world_pos), voronoi.sites().len())
        } else {
            (None, 0)
        };

        if mouse_button_input.just_pressed(MouseButton::Left) {
            if input.pressed(KeyCode::LShift) {
                // LeftShift + left button sets the starting path
                if let Some((site, _)) = closest_site {
                    state.path_start_site = Some(site);
                }
            } else {
                // do not let adding points extremelly close as this degenerate triangulation
                if closest_site.is_none() || closest_site.unwrap().1 > 0.001 {
                    state.add_site_to_voronoi(point);
                    info!("Site added: {:?}", mouse.world_pos);
                    respawn = true;
                }
            }
        } else if mouse_button_input.just_pressed(MouseButton::Right) && num_of_sites > 3 { // don't let it go below 3 as it won't triangulate
        // LeftShift + right button sets the ending path
            if input.pressed(KeyCode::LShift) {
                if let Some(path_start_site) = state.path_start_site {
                    if let Some((site, _)) = closest_site {
                        state.path_end_site = Some(site);

                        // remove existing path path
                        for e in query_path.iter() {
                            commands.entity(e).despawn();
                        }

                        // add new path
                        if let Some(v) = state.voronoi.as_ref() {
                            for s in v.cell(path_start_site).iter_path(&point) {
                                let cell = v.cell(s);
                                spawn_voronoi_cell(&mut commands, &mut meshes, &cell);
                            }
                        }
                    }
                }
            } else {
                // if right click, get closest point and remove it
                if let Some((i, dist)) = closest_site {
                    if dist < 0.2 {
                        state.remove_site_to_voronoi(i);
                        info!("Site removed: {}", i);
                        respawn = true;
                    }
                }
            }
        } else if mouse_button_input.just_pressed(MouseButton::Middle) {
            // print info for closest site
            if let Some((site, dist)) = closest_site {
                if dist < 0.2 {
                    if let Some(v) = state.voronoi.as_ref() {
                        let cell = v.cell(site);
                        println!("{:#?}", cell);
                    } else {
                        println!("No voronoi");
                    }
                }
            }
        }
    }

    // change number of points
    let size = state.size;
    let change = if input.pressed(KeyCode::LShift) { 1000 } else { 100 };
    if input.just_pressed(KeyCode::Up) {
        respawn = true;
        state.new_voronoi(size + change);
    } else if input.just_pressed(KeyCode::Down) {
        respawn = true;
        state.new_voronoi((size as i64 - change as i64).max(120) as usize);
    } else if input.just_pressed(KeyCode::Home) {
        state.site_type = match state.site_type {
            SiteType::Circle => SiteType::Random,
            SiteType::Random => SiteType::Square,
            SiteType::Square => SiteType::Circle,
        };
        respawn = true;
        state.new_voronoi(size);
    }

    if input.pressed(KeyCode::LControl) {
        if input.just_pressed(KeyCode::Z) {
            println!("Undoing. Undo list size: {}, forward list size: {}", state.undo_list.len(), state.forward_list.len());
            state.undo();
            respawn = true;
        } else if input.just_pressed(KeyCode::Y) {
            println!("Undoing forward. Undo list size: {}, forward list size: {}", state.undo_list.len(), state.forward_list.len());
            state.undo_forward();
            respawn = true;
        }
    }

    // span new voronoi with new points
    if input.just_pressed(KeyCode::G) {
        respawn = true;
        // clean up state so it gets fully regenerated
        state.clear();
    }

    if respawn {
        for e in query.iter() {
            commands.entity(e).despawn();
        }

        // may not exist after clean up
        if let Some(voronoi) = &state.voronoi {
            spawn_voronoi_cell(&mut commands, &mut meshes, &voronoi.cell(0));

            spawn_voronoi(commands, meshes, voronoi, &state.voronoi_opts);
        }
    }

    if input.just_pressed(KeyCode::B) {
        println!("{:#?}", state.voronoi);
    }

    let updates: [String; STRING_UI_COUNT] = [
        format!("[C] Clip mode: {:?}", state.clip_behavior),
        format!("[P] Voronoi mesh render mode: {:?}", state.voronoi_opts.voronoi_topoloy),
        format!("[O] Delauney mesh render mode: {:?}", state.voronoi_opts.delauney_topoloy),
        format!("[PgUp/PgDown] Bounding box: {:.2}", state.bounding_box.width()),
        format!("[Home] Site type: {:?}", state.site_type),
        format!("[ArrowUp/ArrowDown/G/MouseClick] # of Sites: {}", state.voronoi.as_ref().map_or(0, |v| v.sites().len())),
        "[W/S/R] Camera Movement".to_string(),
        "[L] Lloyd relaxation".to_string(),
    ];

    for (mut text, update) in query_text.iter_mut().zip(&updates) {
        text.sections[0].value = update.clone();
    }
}