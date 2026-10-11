---@meta

---@class QsoScore
---@field point integer|nil Point of the QSO; defaults to 0.
---@field multiplier string|nil Multiplier of kind `default`.
---@field multipliers table<string, string>|nil Multipliers keyed by kind like `{ area = "1", day = "2026-08-01" }`.
---@field tx_report string|nil Sent RST; defaults to RST_SENT.
---@field tx_number string|nil Sent number; defaults to STX or STX_STRING.
---@field rx_report string|nil Received RST; defaults to RST_RCVD.
---@field rx_number string|nil Received number; defaults to SRX or SRX_STRING.
local QsoScore = {}
