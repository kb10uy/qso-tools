---@meta

---@class ContestSummary
---@field qsos integer Number of QSOs except duplicates.
---@field dupes integer Number of duplicate QSOs.
---@field points integer Sum of points except duplicates.
---@field multipliers integer Number of distinct multipliers.
---@field multipliers_by_kind table<string, integer> Number of distinct multipliers by kind.
---@field bands table<string, BandSummary> Tally by ADIF band.
---@field total integer Points times multipliers, or points if there is no multiplier.
local ContestSummary = {}

---@class BandSummary
---@field qsos integer
---@field points integer
---@field multipliers integer
local BandSummary = {}
