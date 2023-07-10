use crate::{Config, State};
use cw_storage_plus::Item;

pub const CONFIG: Item<Config> = Item::new("config");
pub const STATE: Item<State> = Item::new("state");
