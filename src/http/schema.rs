use serde::{Deserialize, Serialize};

use crate::data::{EntryID, TileID};

#[derive(Debug, Serialize, Deserialize)]
pub struct TileRequest {
    pub entry_id: EntryID,
    pub tile_id: TileID,
}
