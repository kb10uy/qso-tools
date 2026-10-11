---@meta cty

---@class cty
local cty = {}

---@class CtyEntity
---@field name string Entity name.
---@field prefix string Primary prefix.
---@field continent string
---@field cq_zone integer
---@field itu_zone integer

--- Looks up DXCC entity of callsign in cty.dat; raises an error if cty.dat does not exist.
---@param call string Callsign.
---@return CtyEntity|nil entity
function cty.entity(call) end

return cty
