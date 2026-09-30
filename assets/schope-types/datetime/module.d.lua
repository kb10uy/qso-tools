---@meta datetime

---@class datetime
local datetime = {}

--- Returns current datetime in UTC.
---@return DateTime now
function datetime.now_utc() end

--- Returns current datetime in local offset.
---@return DateTime now
function datetime.now_local() end

--- Parses datetime from RFC3339.
---@param dt string RFC3339 datetime string.
---@return DateTime parsed
function datetime.from_rfc3339(dt) end

--- Constructs datetime from date and time.
---@param d string date string.
---@param t string time string.
---@return DateTime parsed
function datetime.from_parts_utc(d, t) end

--- Constructs datetime from date, time, and offset.
---@param d string date string.
---@param t string time string.
---@param o string offset string.
---@return DateTime parsed
function datetime.from_parts_offset(d, t, o) end

return datetime
