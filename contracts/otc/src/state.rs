use cw_storage_plus::{index_list, IndexedMap, Item, MultiIndex};
use otcer_pkg::otc::definitions::{Config, OtcPosition};

pub const CONFIG: Item<Config> = Item::new("config");

pub type PositionMap<'a> = IndexedMap<'a, u64, OtcPosition, OtcPositionIndexer<'a>>;

#[index_list(OtcPosition)]
pub struct OtcPositionIndexer<'a> {
    pub owner: MultiIndex<'a, String, OtcPosition, u64>,
    pub dealer: MultiIndex<'a, String, OtcPosition, u64>,
}

pub fn active_positions<'a>() -> PositionMap<'a> {
    let indexer = OtcPositionIndexer {
        owner: MultiIndex::new(
            |_, val| val.owner.to_string(),
            "active_position",
            "active_position_owner",
        ),
        dealer: MultiIndex::new(
            |_, val| {
                val.dealer
                    .clone()
                    .map(|val| val.to_string())
                    .unwrap_or("".to_string())
            },
            "active_position",
            "active_position_dealer",
        ),
    };

    IndexedMap::new("active_position", indexer)
}

pub fn execute_positions<'a>() -> PositionMap<'a> {
    let indexer = OtcPositionIndexer {
        owner: MultiIndex::new(
            |_, val| val.owner.to_string(),
            "executed_position",
            "executed_position_owner",
        ),
        dealer: MultiIndex::new(
            |_, val| {
                val.dealer
                    .clone()
                    .map(|val| val.to_string())
                    .unwrap_or("".to_string())
            },
            "executed_position",
            "executed_position_dealer",
        ),
    };

    IndexedMap::new("executed_position", indexer)
}
