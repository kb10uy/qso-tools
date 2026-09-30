---@meta

--- Mark a table as a list (array) for serialization.
--- The table will be treated as a JSON array on the Rust side.
---@param tbl table The table to mark
---@return table tbl The same table with list metadata
function list(tbl) end

--- Mark a table as a map (object) for serialization.
--- The table will be treated as a JSON object on the Rust side.
---@param tbl table The table to mark
---@return table tbl The same table with map metadata
function map(tbl) end
