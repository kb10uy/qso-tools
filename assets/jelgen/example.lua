local datetime = require("datetime")
local prompt = require("prompt")

local contest_start = datetime.from_parts_offset("2026-06-06", "21:00:00", "09:00")
local contest_end = datetime.from_parts_offset("2026-06-07", "21:00:00", "09:00")

local categories = { "XMA", "CMA", "PMA" }

---@param args table<string, string>
---@return ContestConfig
local function initialize(args)
    return {
        contest_name = "Example Contest",
        category_code = args.category or prompt.choice("Category", categories),
        comments = args.comments or prompt.text("Comments", ""),
        offset = "+09:00",
    }
end

---@param mode string
---@return string
local function mode_group(mode)
    if mode == "CW" then
        return "CW"
    else
        return "PH"
    end
end

---@param entry ContestEntry
---@return QsoScore|nil
local function process_qso(entry)
    local qso = entry.qso
    if qso.datetime < contest_start or contest_end <= qso.datetime then
        return nil
    end

    local area = string.match(entry.exchange.rx_number or "", "^(%d+)")
    if area == nil then
        print("no area number", qso.call)
    end

    return {
        key = string.format("%s-%s-%s", qso.band, mode_group(qso.mode), qso.call),
        point = string.sub(area or "", 1, 2) == "15" and 2 or 1,
        multiplier = area,
        multiplier_key = area and string.format("%s-%s", qso.band, area),
    }
end

---@param summary ContestSummary
---@return integer
local function summarize(summary)
    return summary.points * summary.multipliers
end

return {
    initialize = initialize,
    process_qso = process_qso,
    summarize = summarize,
}
