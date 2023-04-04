use bevy::prelude::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

//use bevy_inspector_egui::quick::WorldInspectorPlugin;

const TILE_SIZE: u32 = 4;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.0, 0.05, 0.0)))
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_startup_system(spawn_points)
        //.add_plugin(WorldInspectorPlugin::default())
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn spawn_points(mut commands: Commands) {
    let map_size = 120;
    let num_rooms = 20;

    let mut rooms = Vec::<Rectangle>::new();
    //let seed = rand::thread_rng().gen_range(0..std::u64::MAX);
    let seed = 18212822918530262970;
    info!("Seed: {}", seed);
    let mut rng = ChaCha8Rng::seed_from_u64(seed);

    let mut trial_ctr = 0;
    while rooms.len() < num_rooms {
        trial_ctr += 1;
        info!("Room generation trial {}", trial_ctr);
        let maybe_rooms = generate_non_overlapping_rectangles(&mut rng, num_rooms, map_size, 4, 15);
        if maybe_rooms.is_some() {
            rooms = maybe_rooms.unwrap();
        }
    }
    info!("Rooms created");
    rooms = compress_rooms(&mut rooms);
    let intersections = find_intersections(&rooms);

    for room in rooms {
        commands.spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::YELLOW,
                custom_size: Some(Vec2::new((room.w*TILE_SIZE) as f32, (room.h*TILE_SIZE) as f32)),
                anchor: bevy::sprite::Anchor::BottomLeft,
                ..default()
            },
            transform: Transform::from_translation(Vec3::new((room.x*TILE_SIZE as i32) as f32, (room.y*TILE_SIZE as i32) as f32, 0.0)),
            ..default()
        });

        // inner
        commands.spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::MIDNIGHT_BLUE,
                custom_size: Some(Vec2::new(((room.w-2)*TILE_SIZE) as f32, ((room.h-2)*TILE_SIZE) as f32)),
                anchor: bevy::sprite::Anchor::BottomLeft,
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(((room.x+1)*TILE_SIZE as i32) as f32, ((room.y+1)*TILE_SIZE as i32) as f32, 0.0)),
            ..default()
        });
    }

    for wall in intersections {
        commands.spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::RED,
                custom_size: Some(Vec2::new((wall.w*TILE_SIZE) as f32, (wall.h*TILE_SIZE) as f32)),
                anchor: bevy::sprite::Anchor::BottomLeft,
                ..default()
            },
            transform: Transform::from_translation(Vec3::new((wall.x*TILE_SIZE as i32) as f32, (wall.y*TILE_SIZE as i32) as f32, 0.0)),
            ..default()
        });
    }
}
fn find_intersections(rectangles: &[Rectangle]) -> Vec<Rectangle> {
    let mut intersections = vec![];

    for i in 0..rectangles.len() {
        for j in i + 1..rectangles.len() {
            let rect1 = &rectangles[i];
            let rect2 = &rectangles[j];

            if let Some(intersection) = intersection(rect1, rect2) {
                intersections.push(intersection);
            }
        }
    }

    intersections
}

fn move_group(rects: &mut Vec<Rectangle>, dx: i32, dy: i32) {
    for rect in rects.iter_mut() {
        rect.x += dx;
        rect.y += dy;
    }
}

fn move_and_merge_groups(groups: &mut Vec<Vec<Rectangle>>) {
    let mut i = 0;
    while i < groups.len() {
        let (dx, dy) = if groups[i].iter().any(|rect| intersection(rect, &Rectangle { x: 0, y: 0, w: 1, h: 1 }).is_some()) {
            (0, 0)
        } else {
            (if groups[i][0].x < 0 { 1 } else if groups[i][0].x > 0 { -1 } else { 0 },
             if groups[i][0].y < 0 { 1 } else if groups[i][0].y > 0 { -1 } else { 0 })
        };
        move_group(&mut groups[i], dx, dy);
        for j in (i + 1)..groups.len() {
            if groups[i].iter().any(|rect1| groups[j].iter().any(|rect2| intersection(rect1, rect2).is_some())) {
                let mut new_group = Vec::new();
                new_group.extend_from_slice(&groups[i]);
                new_group.extend_from_slice(&groups[j]);
                groups.remove(j);
                groups[i] = new_group;
                i -= 1;
                break;
            }
        }
        i += 1;
    }
}

fn compress_rooms(rooms: &mut Vec<Rectangle>) -> Vec<Rectangle>
{
    let mut groups = Vec::<Vec<Rectangle>>::new();
    for room in rooms.iter() {
        groups.push(vec![room.clone()]);
    }

    while groups.len() > 1 {
        move_and_merge_groups(&mut groups);
    }
    groups[0].clone()
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Rectangle {
    x: i32,
    y: i32,
    w: u32,
    h: u32,
}

fn generate_non_overlapping_rectangles(rng: &mut ChaCha8Rng, n: usize, map_size: u32, min_size: u32, max_size: u32) -> Option<Vec<Rectangle>> {
    let mut rectangles = vec![];
    for _ in 0..n {
        let mut rect = Rectangle {
            x: rng.gen_range(0..map_size) as i32 - (map_size/2) as i32,
            y: rng.gen_range(0..map_size) as i32 - (map_size/2) as i32,
            w: rng.gen_range(min_size..max_size),
            h: rng.gen_range(min_size..max_size),
        };
        let mut trial = 0usize;
        while rectangles.iter_mut().any(|r| intersection(&rect, r).is_some()) {
            if trial > 3000 { return None };
            trial += 1;
            rect.x = rng.gen_range(0..map_size) as i32 - (map_size/2) as i32;
            rect.y = rng.gen_range(0..map_size) as i32 - (map_size/2) as i32;
        }
        rectangles.push(rect);
    }
    Some(rectangles)
}

fn intersection(rect1: &Rectangle, rect2: &Rectangle) -> Option<Rectangle> {
    let max_l = rect1.x.max(rect2.x);
    let min_r = (rect1.x + rect1.w as i32).min(rect2.x + rect2.w as i32);
    let max_b = rect1.y.max(rect2.y);
    let min_t = (rect1.y + rect1.h as i32).min(rect2.y + rect2.h as i32);

    if max_l > min_r || max_b > min_t {
        // the rectangles do not intersect
        None
    } else {
        // calculate the intersection rectangle
        let x = max_l;
        let y = max_b;
        let w = (min_r - max_l) as u32;
        let h = (min_t - max_b) as u32;
        if w == 0 || h == 0 {
            None
        } else {
            Some(Rectangle { x, y, w, h })
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;
    #[test]
    fn test_edge_intersection() {
        let r1 = Rectangle {x:-3, y:0, w:4, h:4};
        let r2 = Rectangle {x:0, y:1, w:4, h:4};

        assert_eq!(intersection(&r1, &r2), Some(Rectangle{x:0, y:1, w:1, h:3}));
    }
}
