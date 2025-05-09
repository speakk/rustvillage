use crate::features::ai::PathFollow;
use crate::features::map::map_model::MapData;
use crate::features::path_finding::grid::PathingGridResource;
use crate::features::path_finding::path_finding::{Path, PathFollowFinished, PathFollowResult};
use crate::features::position::WorldPosition;
use beet::prelude::*;
use bevy::prelude::*;

#[derive(Component, Reflect)]
#[require(ContinueRun, Name::new("EscapeFromSolidAction"))]
pub struct EscapeFromSolidAction;

#[allow(clippy::too_many_arguments)]
fn escape_from_solid_action(
    world_positions: Query<&WorldPosition>,
    actions: Query<(Entity, &Running), With<EscapeFromSolidAction>>,
    mut commands: Commands,
    map_data: Query<&MapData>,
    pathing_grid: Res<PathingGridResource>,
) {
    for (action_entity, running) in actions.iter() {
        let target_agent = running.origin;
        let world_position = world_positions.get(target_agent).unwrap();

        //let trigger_entity = &trigger.target();

        let free_neighbor_coordinate =
            pathing_grid.get_nearest_available_coordinate(world_position.0.as_ivec2());

        if free_neighbor_coordinate.is_none() {
            println!("No free neighbor found, aborting");
            running.trigger_result(&mut commands, action_entity, RunResult::Failure);
            return;
        }

        let path = vec![
            world_position.0.as_ivec2(),
            free_neighbor_coordinate.unwrap(),
        ];
        // let path_follow = PathFollow {
        //     path: Path {
        //         steps: path,
        //     },
        //     ..Default::default()
        // };
        // 
        // let running_clone = *running;

        // commands.entity(target_agent).observe(
        //     move |path_follow_trigger: Trigger<PathFollowFinished>, mut commands: Commands| {
        //         match path_follow_trigger.result {
        //             PathFollowResult::Success => {
        //                 running_clone.trigger_result(
        //                     &mut commands,
        //                     action_entity,
        //                     RunResult::Success,
        //                 );
        //                 println!("Escape action finished, success!");
        //             }
        //             PathFollowResult::Failure => {
        //                 running_clone.trigger_result(
        //                     &mut commands,
        //                     action_entity,
        //                     RunResult::Failure,
        //                 );
        //                 println!("Escape action finished, failure!");
        //             }
        //         }
        //     },
        // );
    }
}
