use crate::features::map::map_model::MapData;
use crate::features::path_finding::path_finding::{spawn_pathfinding_task, PathFollowFinished, PathFollowResult, PathfindingTask};
use crate::features::position::WorldPosition;
use beet::prelude::{Action, OnRun, OnRunResult, TargetEntity};
use bevy::math::{IVec2, Vec2};
use bevy::prelude::{Commands, Component, Query, Reflect, Res, Trigger};
use crate::features::ai::{BehaviourTree, PathFollow};
use crate::features::misc_components::gltf_asset::GltfAnimation;
use crate::features::path_finding::grid::PathingGridResource;

#[derive(Component, Action, Reflect)]
#[require(ContinueRun, Name(|| "GoToAction"))]
#[observers(go_to_action)]
pub struct GoToAction {
    pub(crate) target: IVec2,
}

#[allow(clippy::too_many_arguments)]
fn go_to_action(
    trigger: Trigger<OnRun>,
    target_agents: Query<&TargetEntity>,
    world_positions: Query<&WorldPosition>,
    goto_action: Query<&GoToAction>,
    mut commands: Commands,
    map_data: Query<&MapData>,
    pathing_grid: Res<PathingGridResource>,
) {
    let target_agent = target_agents.get(trigger.entity()).unwrap().0;
    let world_position = world_positions.get(target_agent).unwrap();
    let goto_action = goto_action.get(trigger.entity()).unwrap();
    let target_coordinate = goto_action.target;
    println!("Ensure path entered NEW, to {}", target_coordinate);
    let target_position = WorldPosition(Vec2::new(
        target_coordinate.x as f32,
        target_coordinate.y as f32,
    ));
    spawn_pathfinding_task(
        &mut commands,
        target_agent,
        &pathing_grid,
        map_data.single(),
        *world_position,
        target_position,
        Some(trigger.entity()),
    );

    let trigger_entity = trigger.entity();

    // Cleanup on BT remove
    commands.entity(trigger_entity).observe(move |_trigger: Trigger<OnRemove, TargetEntity>, mut commands: Commands| {
        commands.entity(target_agent).remove::<PathFollow>().remove::<PathfindingTask>();
    });
    
    commands.entity(target_agent).observe(
        move |path_follow_trigger: Trigger<PathFollowFinished>, mut commands: Commands| {
            if path_follow_trigger.related_task != Some(trigger_entity) {
                return;
            }
            
            if commands.get_entity(trigger_entity).is_none() {
                return;
            }

            match path_follow_trigger.result {
                PathFollowResult::Success => {
                    commands
                        .entity(trigger_entity)
                        .trigger(OnRunResult::success());
                    println!("GoTo action finished, success!");
                }
                PathFollowResult::Failure => {
                    commands
                        .entity(trigger_entity)
                        .trigger(OnRunResult::failure());
                    println!("GoTo action finished, failure!");
                }
            }
        },
    );
}
