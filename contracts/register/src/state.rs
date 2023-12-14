use cw_storage_plus::Item;
use otcer_pkg::register::definitions::Config;

pub const CONFIG: Item<Config> = Item::new("config_key");