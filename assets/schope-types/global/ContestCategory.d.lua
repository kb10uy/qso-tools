---@meta

---@alias SummaryItem "callsign"|"op_callsign"|"operator"|"name"|"address"|"tel"|"email"|"license_date"|"age"|"power"|"fd_coeff"|"op_place"|"power_supply"|"comments"|"multi_op_list"|"reg_club_number"|"date"

---@class ContestCategory
---@field code string Category code; set from the key of categories.
---@field name string Category name.
---@field modes string[]|nil Mode groups of QSOs accepted; defaults to all.
---@field bands string[]|nil ADIF bands of QSOs accepted; defaults to all.
---@field exclude_bands string[]|nil ADIF bands of QSOs not accepted.
---@field requires SummaryItem[]|nil Summary items required by this category.
---@field cabrillo table<string, string>|nil `CATEGORY-*` values of Cabrillo like `{ operator = "SINGLE-OP", power = "LOW" }`.
local ContestCategory = {}
