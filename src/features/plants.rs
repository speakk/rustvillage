use crate::features::inventory::{Inventory, InventoryChanged, InventoryChangedType};
use crate::features::misc_components::gltf_asset::GltfData;
use crate::features::misc_components::ItemAmount;
use crate::features::position::{CoordinateToEntity, WorldPosition};
use bevy::app::{App, Plugin};
use bevy::prelude::*;
use bevy::prelude::{Component, Reflect};
use bevy::time::common_conditions::on_timer;
use rand::Rng;
use std::time::Duration;

pub struct PlantsPlugin;

impl Plugin for PlantsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlantStageAdvanced>()
            .add_systems(Update, initialize_plant)
            .add_systems(
                Update,
                update_growth_process.run_if(on_timer(Duration::from_millis(100))),
            )
            .add_observer(update_gltf_based_on_growth_stage);
    }
}

#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct Plant {
    pub growth_stages: usize,
    pub current_growth_stage: u8,
    pub growth_speed: f32,
    pub growth_requirements: Vec<ItemAmount>,
    pub current_stage_growth_process: f32,
    pub finished_growing: bool,
    pub random_growth_multiplier: f32,
}

#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct Planted;

#[derive(Component, Default, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct GrowthProvider;

#[derive(Event)]
pub struct PlantStageAdvanced;

#[derive(Event)]
pub struct PlantFinishedGrowing;

impl Default for Plant {
    fn default() -> Self {
        Self {
            growth_stages: 1,
            current_growth_stage: 0,
            growth_speed: 1.0,
            growth_requirements: vec![],
            current_stage_growth_process: 0.0,
            finished_growing: false,
            random_growth_multiplier: rand::rng().random_range(0.6..1.2),
        }
    }
}

pub fn initialize_plant(mut commands: Commands, query: Query<Entity, Added<Plant>>) {
    for entity in query.iter() {
        commands.entity(entity).trigger(PlantStageAdvanced);
    }
}

pub fn update_growth_process(
    mut query: Query<(Entity, &mut Plant, &WorldPosition), With<Planted>>,
    time: Res<Time>,
    mut previous_run: Local<f32>,
    coordinate_to_entity: Res<CoordinateToEntity>,
    mut inventories: Query<&mut Inventory, With<GrowthProvider>>,
    mut commands: Commands,
) {
    let current_time = time.elapsed_secs();
    let delta = current_time - *previous_run;
    *previous_run = current_time;

    for (entity, mut plant, world_position) in query.iter_mut() {
        if plant.finished_growing {
            continue;
        }

        if !check_growth_requirements(
            &plant.growth_requirements,
            world_position,
            &coordinate_to_entity,
            &mut inventories,
            &mut commands
        ) {
            continue;
        }

        plant.current_stage_growth_process +=
            plant.growth_speed * plant.random_growth_multiplier * delta;
        if plant.current_stage_growth_process >= 1.0 {
            plant.current_stage_growth_process = 0.0;
            plant.current_growth_stage += 1;

            commands.entity(entity).trigger(PlantStageAdvanced);

            if plant.current_growth_stage >= plant.growth_stages as u8 - 1 {
                plant.finished_growing = true;
                commands.entity(entity).trigger(PlantFinishedGrowing);
            }
        }
    }
}

fn check_growth_requirements(
    growth_requirements: &Vec<ItemAmount>,
    world_position: &WorldPosition,
    coordinate_to_entity: &CoordinateToEntity,
    inventories: &mut Query<&mut Inventory, With<GrowthProvider>>,
    commands: &mut Commands,
) -> bool {
    let entities_at_coordinate = coordinate_to_entity.0.get(&world_position.as_coordinate());
    if let Some(entities_at_coordinate) = entities_at_coordinate {
        for entity in entities_at_coordinate {
            let inventory = inventories.get(*entity);
            if let Ok(inventory) = inventory {
                let mut has_all = true;

                for requirement in growth_requirements {
                    if !inventory.has_amount(requirement.item_id, requirement.amount) {
                        has_all = false;
                        break;
                    }
                }

                // TODO: Consider whether this should happen elsewhere
                if has_all {
                    for requirement in growth_requirements {
                        commands.entity(*entity).trigger(InventoryChanged(InventoryChangedType::Remove(*requirement)));
                    }

                    return true;
                }
            }
        }
    }

    false
}

pub fn update_gltf_based_on_growth_stage(
    trigger: Trigger<PlantStageAdvanced>,
    mut query: Query<(&Plant, &mut GltfData)>,
) {
    if let Ok((plant, mut gltf_data)) = query.get_mut(trigger.entity()) {
        let current_scene_name = format!("stage_{}", plant.current_growth_stage);
        if gltf_data.scene_name != Some(current_scene_name.clone()) {
            gltf_data.scene_name = Some(current_scene_name.clone());
        }
    }
}
