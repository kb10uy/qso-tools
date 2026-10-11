---@meta

---@class ContestSummary
---@field qsos integer Number of QSOs except duplicates.
---@field dupes integer Number of duplicate QSOs.
---@field points integer Sum of points except duplicates.
---@field multipliers integer Number of distinct multipliers.
---@field total integer Points times multipliers, or points if there is no multiplier.
local ContestSummary = {}
