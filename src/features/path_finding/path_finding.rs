use crate::bundles::settler::Settler;
use crate::features::ai::PathFollow;
use crate::features::movement::Velocity;
use crate::features::path_finding::grid::PathingGridResource;
use crate::features::position::WorldPosition;
use bevy::prelude::*;
use bevy::tasks::futures_lite::future;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use pathfinding::prelude::bfs;

#[derive(Debug)]
pub struct Path {
    pub steps: Vec<IVec2>,
    pub related_task: Option<Entity>,
}

#[derive(Component)]
pub struct PathfindingTask(Task<Option<Path>>);

pub fn spawn_pathfinding_task(
    commands: &mut Commands,
    target_entity: Entity,
    grid: &PathingGridResource,
    start: WorldPosition,
    end: WorldPosition,
    related_task: Option<Entity>,
) {
    let thread_pool = AsyncComputeTaskPool::get();
    let grid = Box::new(grid.clone());

    let task = thread_pool.spawn(async move {
        let start = grid.get_nearest_available_vertex(start.as_coordinate());
        let end = grid.get_nearest_available_vertex(end.as_coordinate());

        if start.is_none() || end.is_none() {
            println!("start or end not found, returning None from Pathfinding task");
            return None;
        }

        let points = bfs(
            &start.unwrap(),
            |original_point| {
                grid.neighbours(true, (original_point.x as usize, original_point.y as usize))
                    .iter()
                    .map(|p| UVec2::new(p.0 as u32, p.1 as u32))
                    .collect::<Vec<_>>()
            },
            |p| *p == end.unwrap(),
        );

        println!(
            "from: {:?} to: {:?}, found path: {:?}",
            start,
            end,
            points.is_some()
        );
        //println!("grid: {:?}", grid);
        points.map(|points| Path {
            steps: points
                .iter()
                .map(|p| grid.convert_to_centered_coordinate(*p))
                .collect::<Vec<_>>(),
            related_task,
        })
    });

    println!("Pathfinding task spawned for agent: {:?}", target_entity);
    commands.entity(target_entity).insert(PathfindingTask(task));
}

pub fn apply_pathfinding_result(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut PathfindingTask)>,
) {
    for (task_entity, mut task) in &mut tasks {
        if let Some(result) = future::block_on(future::poll_once(&mut task.0)) {
            commands.entity(task_entity).remove::<PathfindingTask>();

            if let Some(path) = result {
                commands
                    .entity(task_entity)
                    .insert(PathFollow {
                        path,
                        ..Default::default()
                    })
                    .observe(
                        move |_trigger: Trigger<OnRemove, PathFollow>,
                              mut velocity_query: Query<&mut Velocity>| {
                            let mut velocity = velocity_query
                                .get_mut(task_entity)
                                .expect("No velocity for path follow entity");

                            velocity.0 = Vec2::ZERO;
                        },
                    );
            }
        }
    }
}

pub enum PathFollowResult {
    Success,
    Failure,
}

#[derive(Event)]
pub struct PathFollowFinished {
    pub result: PathFollowResult,
    pub related_task: Option<Entity>,
}

pub fn follow_path(
    mut query: Query<(Entity, &mut PathFollow, &WorldPosition, &mut Velocity)>,
    mut commands: Commands,
) {
    const AT_POINT_THRESHOLD: f32 = 1.0;

    for (entity, mut path_follow, world_position, mut velocity) in query.iter_mut() {
        if path_follow.path.steps.len() == 1 {
            follow_path_succeed(&mut commands, entity, path_follow, &mut velocity);
            continue;
        }

        let current_index = path_follow.current_path_index;
        let next_point = path_follow.path.steps[current_index + 1];

        let direction = (next_point.as_vec2() - world_position.0).normalize_or_zero();
        let speed = 3.0;
        let final_vector = Vec2::new(direction.x, direction.y) * speed;
        velocity.0 = final_vector;

        if world_position.0.distance(next_point.as_vec2()) <= AT_POINT_THRESHOLD {
            if current_index < path_follow.path.steps.len() - 2 {
                path_follow.current_path_index += 1;
            } else {
                follow_path_succeed(&mut commands, entity, path_follow, &mut velocity);
            }
        }
    }
}

fn follow_path_succeed(
    commands: &mut Commands,
    entity: Entity,
    path_follow: Mut<PathFollow>,
    velocity: &mut Mut<Velocity>,
) {
    velocity.0 = Vec2::ZERO;
    commands
        .entity(entity)
        .trigger(PathFollowFinished {
            result: PathFollowResult::Success,
            related_task: path_follow.path.related_task,
        })
        .remove::<PathFollow>();
}

#[allow(unused, reason = "For testing")]
pub fn test_add_pathfinding_task_to_settler(
    added_settler: Query<(Entity, &WorldPosition), Added<Settler>>,
    mut commands: Commands,
    pathing_grid: Res<PathingGridResource>,
) {
    for (entity, world_position) in added_settler.iter() {
        println!("Adding pathfinding to settler");
        let end = WorldPosition(Vec2::new(2.0, 2.0));
        spawn_pathfinding_task(
            &mut commands,
            entity,
            &pathing_grid,
            *world_position,
            end,
            None,
        );
    }
}
