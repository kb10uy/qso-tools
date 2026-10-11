---@meta jarl

---@class jarl
local jarl = {}

---@class JarlNumber
---@field area string Area number like `100116`.
---@field power string|nil Power code (`P`, `L`, `M` or `H`).

---@class JarlJcx
---@field code string JCC/JCG code.
---@field kind string|nil `prefecture`, `city`, `gun` or `ward`.
---@field name string|nil Full Japanese name like `東京都豊島区`.
---@field prefecture string Prefecture number containing the code.
---@field area integer|nil Call area of the prefecture.
---@field has_wards boolean Whether the code is a city divided into wards.

--- Splits contest number like `100116L` into area number and power code.
---@param number string|nil Contest number without RST.
---@return JarlNumber|nil parsed
function jarl.parse_number(number) end

--- Looks up JCC/JCG code (or prefecture number) in japan-jcx.toml.
---@param code string|nil JCC/JCG code.
---@return JarlJcx|nil found
function jarl.jcx(code) end

--- Returns call area of prefecture number, including Hokkaido sub-prefectures (101-114) and Ogasawara (48).
---@param code string Prefecture number.
---@return integer|nil area
function jarl.area_of_prefecture(code) end

--- Returns call area of Japanese callsign, or the area of portable suffix like `/3`.
---@param call string Callsign.
---@return integer|nil area
function jarl.area_of_call(call) end

return jarl
