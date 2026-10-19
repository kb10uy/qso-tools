local cty = require("cty")

local bands = { "80m", "40m", "20m", "15m", "10m" }
local powers = {
    HP = { label = "ハイパワー", cabrillo = "HIGH" },
    LP = { label = "ローパワー", cabrillo = "LOW" },
    QRP = { label = "QRP", cabrillo = "QRP" },
}
local area_entities = { JA = "JA", K = "W", VE = "VE", VK = "VK" }
local non_area_designators = { P = true, M = true, AM = true, QRP = true, A = true }

---@param name string
---@param cabrillo table<string, string>
---@param requires SummaryItem[]|nil
---@return ContestCategory
local function category(name, cabrillo, requires)
    cabrillo.band = "ALL"
    cabrillo.mode = "RTTY"
    return { name = name, modes = { "RTTY" }, bands = bands, cabrillo = cabrillo, requires = requires }
end

---@type table<string, ContestCategory>
local categories = {
    CHECKLOG = category("チェックログ", { operator = "CHECKLOG" }),
}

for code, p in pairs(powers) do
    local name = "シングルオペ オールバンド " .. p.label
    categories["SO" .. code] = category(name, { operator = "SINGLE-OP", power = p.cabrillo })
    categories["SO" .. code .. "-YOUTH"] = category(
        name .. " YOUTH",
        { operator = "SINGLE-OP", power = p.cabrillo, overlay = "YOUTH" },
        { "age" }
    )
    if code ~= "QRP" then
        categories["MM" .. code] = category(
            "マルチオペ オールバンド " .. p.label,
            { operator = "MULTI-OP", power = p.cabrillo },
            { "multi_op_list" }
        )
    end
end

---@param call string
---@return string
local function call_area(call)
    local home
    local designators = {}
    for segment in call:gmatch("[^/]+") do
        if segment:match("%d%a+$") and (home == nil or #segment > #home) then
            if home ~= nil then
                table.insert(designators, home)
            end
            home = segment
        else
            table.insert(designators, segment)
        end
    end

    for _, designator in ipairs(designators) do
        if not non_area_designators[designator] then
            return designator:match("^.*(%d)") or "0"
        end
    end
    return home and home:match("^.*(%d)%a+$") or "0"
end

---@type ContestDefinition
return {
    name = "JARL World Wide RTTYコンテスト",
    contest_id = "JARTS-WW-RTTY",
    offset = "+00:00",
    categories = categories,
    dupe_scope = { "band" },
    multiplier_scope = { "band" },

    score_qso = function(entry, ctx)
        local call = entry.qso.call:upper()
        local warning = nil
        if not (entry.exchange.rx_number or ""):match("^%d+$") then
            warning = "received age is invalid"
        end
        if call:match("/MM$") then
            return { point = 2 }, warning
        end

        local station = entry.station_callsign or ctx.args.callsign
        if station == nil then
            error("give STATION_CALLSIGN in ADIF or -A callsign=CALL")
        end
        local mine = cty.entity(station)
        if mine == nil then
            error("entity of " .. station .. " is unknown")
        end
        local theirs = cty.entity(call)
        if theirs == nil then
            return { point = 0 }, "unknown entity"
        end

        local point = theirs.continent == mine.continent and 2 or 3
        local area_entity = area_entities[theirs.prefix]
        if area_entity ~= nil then
            return { point = point, multipliers = { area = area_entity .. call_area(call) } }, warning
        end
        return { point = point, multipliers = { entity = theirs.prefix } }, warning
    end,
}
