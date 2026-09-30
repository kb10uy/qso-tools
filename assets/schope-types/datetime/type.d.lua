---@meta

---@class DateTime
---@field year integer
---@field month integer
---@field day integer
---@field hour integer
---@field minute integer
---@field second integer
---@field date_str string
---@field time_str string
local DateTime = {}

--- Returns new DateTime that points same moment in UTC.
---@return DateTime dt DateTime in UTC.
function DateTime:to_utc() end

--- Returns new DateTime that points same moment in specified offset.
---@param offset string offset value.
---@return DateTime dt DateTime in specified offset.
function DateTime:to_offset(offset) end

--- Formats into string.
---@param desc string format descriptor written in Rust `time` crate style.
function DateTime:format(desc) end
