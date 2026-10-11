---@meta

---@class QsoScore
---@field key string Key to detect duplicate QSOs.
---@field point integer|nil Point of the QSO; defaults to 0.
---@field multiplier string|nil Multiplier shown in log sheet.
---@field multiplier_key string|nil Key to count the multiplier only once; defaults to multiplier.
---@field band string|nil Band in log sheet; defaults to MHz notation of the band.
---@field mode string|nil Mode in log sheet; defaults to the mode.
---@field tx_report string|nil Sent RST; defaults to exchange.
---@field tx_number string|nil Sent number; defaults to exchange.
---@field rx_report string|nil Received RST; defaults to exchange.
---@field rx_number string|nil Received number; defaults to exchange.
local QsoScore = {}
