---@meta

---@alias ScopeDimension "band"|"band_group"|"mode"|"mode_group"

---@class ContestDefinition
---@field name string CONTESTNAME of JARL eLog.
---@field contest_id string|nil CONTEST of Cabrillo; defaults to name.
---@field offset string|nil UTC offset of JARL log sheet and datetimes without offset like `+09:00`; defaults to JST.
---@field categories table<string, ContestCategory> Categories keyed by category code.
---@field mode_groups table<string, string[]>|nil Modes or submodes in each mode group; defaults to `CW` and `PH` (SSB, AM, FM and DIGITALVOICE). A mode not listed forms its own group.
---@field band_groups table<string, string[]>|nil ADIF bands in each band group. A band not listed forms its own group.
---@field dupe_scope ScopeDimension[]|nil Attributes separating duplicate checks of the same callsign; defaults to `{ "band" }`.
---@field multiplier_scope ScopeDimension[]|nil Attributes separating multipliers; defaults to `{ "band" }`.
---@field score_qso fun(entry: ContestEntry, ctx: ContestContext): QsoScore|nil, string|nil Scores QSO, or returns nil to exclude it from the log. The second value is the reason shown in log messages.
---@field dupe_key (fun(entry: ContestEntry, ctx: ContestContext): string)|nil Builds key to detect duplicate QSOs instead of dupe_scope.
---@field multiplier_key (fun(entry: ContestEntry, kind: string, multiplier: string, ctx: ContestContext): string)|nil Builds key to count the multiplier only once instead of multiplier_scope.
---@field total (fun(summary: ContestSummary, ctx: ContestContext): integer)|nil Calculates total score; defaults to points times multipliers.
local ContestDefinition = {}
