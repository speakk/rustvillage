use crate::bundles::buildables::Buildable;
use crate::bundles::{Id, ItemId};
use crate::features::misc_components::simple_mesh::{SimpleMesh, SimpleMeshType};
use crate::features::misc_components::ItemAmount;
use crate::features::path_finding::grid::Solid;
use bevy::prelude::*;

#[derive(Component, Default, Reflect, Clone)]
#[require(
    Id(|| Id(ItemId::WoodenWall)),
    Name(|| "Wooden Wall"),
    Solid,
    Buildable(|| Buildable {
        item_requirements: vec![
            ItemAmount {
                item_id: ItemId::Lumber,
                amount: 2,
            }
        ],
        ..Default::default()
    }),
    SimpleMesh(|| SimpleMesh(SimpleMeshType::Cuboid))
)]
#[reflect(Component)]
pub struct WoodenWall;
