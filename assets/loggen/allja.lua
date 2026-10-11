local jarl = require("jarl")

local bands = {
    { code = "19", label = "1.9MHz", band = "160m", cabrillo = "160M" },
    { code = "35", label = "3.5MHz", band = "80m", cabrillo = "80M" },
    { code = "7", label = "7MHz", band = "40m", cabrillo = "40M" },
    { code = "14", label = "14MHz", band = "20m", cabrillo = "20M" },
    { code = "21", label = "21MHz", band = "15m", cabrillo = "15M" },
    { code = "28", label = "28MHz", band = "10m", cabrillo = "10M" },
    { code = "50", label = "50MHz", band = "6m", cabrillo = "6M" },
}
local powers = { H = "HIGH", M = "LOW", P = "QRP" }
local modes = {
    C = { name = "電信", groups = { "CW" }, cabrillo = "CW" },
    X = { name = "電信電話", cabrillo = "MIXED" },
}

---@param name string
---@param cabrillo table<string, string>
---@param fields table|nil
---@return ContestCategory
local function category(name, cabrillo, fields)
    local c = fields or {}
    c.name = name
    c.cabrillo = cabrillo
    return c
end

---@type table<string, ContestCategory>
local categories = {
    PA = category(
        "電話部門 シングルオペ オールバンド",
        { operator = "SINGLE-OP", band = "ALL", mode = "SSB" },
        { modes = { "PH" }, exclude_bands = { "20m" } }
    ),
    PN = category(
        "電話部門 シングルオペ ニューカマー",
        { operator = "SINGLE-OP", band = "ALL", mode = "SSB" },
        { modes = { "PH" }, exclude_bands = { "20m" }, requires = { "license_date" } }
    ),
    PMA = category(
        "電話部門 マルチオペ オールバンド",
        { operator = "MULTI-OP", band = "ALL", mode = "SSB" },
        { modes = { "PH" }, exclude_bands = { "20m" }, requires = { "multi_op_list" } }
    ),
    CS = category(
        "電信部門 シングルオペ シルバー",
        { operator = "SINGLE-OP", band = "ALL", mode = "CW" },
        { modes = { "CW" }, requires = { "age" } }
    ),
    XS = category(
        "電信電話部門 シングルオペ シルバー",
        { operator = "SINGLE-OP", band = "ALL", mode = "MIXED" },
        { requires = { "age" } }
    ),
    XMJ = category(
        "電信電話部門 マルチオペ ジュニア",
        { operator = "MULTI-OP", band = "ALL", mode = "MIXED" },
        { requires = { "age", "multi_op_list" } }
    ),
}

for prefix, mode in pairs(modes) do
    for power, cabrillo_power in pairs(powers) do
        categories[prefix .. "A" .. power] = category(
            mode.name .. "部門 シングルオペ オールバンド",
            { operator = "SINGLE-OP", band = "ALL", mode = mode.cabrillo, power = cabrillo_power },
            { modes = mode.groups }
        )
        if power ~= "P" then
            categories[prefix .. "MA" .. power] = category(
                mode.name .. "部門 マルチオペ オールバンド",
                { operator = "MULTI-OP", band = "ALL", mode = mode.cabrillo, power = cabrillo_power },
                { modes = mode.groups, requires = { "multi_op_list" } }
            )
        end
        for _, b in ipairs(bands) do
            categories[prefix .. b.code .. power] = category(
                string.format("%s部門 シングルオペ %sバンド", mode.name, b.label),
                { operator = "SINGLE-OP", band = b.cabrillo, mode = mode.cabrillo, power = cabrillo_power },
                { modes = mode.groups, bands = { b.band } }
            )
        end
    end
end

for _, b in ipairs(bands) do
    if b.code ~= "14" then
        categories["P" .. b.code] = category(
            string.format("電話部門 シングルオペ %sバンド", b.label),
            { operator = "SINGLE-OP", band = b.cabrillo, mode = "SSB" },
            { modes = { "PH" }, bands = { b.band } }
        )
    end
end

---@param area string
---@return boolean
local function is_prefecture_or_subprefecture(area)
    return #area <= 3 and area ~= "01" and jarl.area_of_prefecture(area) ~= nil
end

---@type ContestDefinition
return {
    name = "ALL JAコンテスト",
    contest_id = "JARL-ALL-JA",
    categories = categories,
    dupe_scope = { "band" },
    multiplier_scope = { "band" },

    score_qso = function(entry)
        local number = jarl.parse_number(entry.exchange.rx_number)
        if number == nil or not is_prefecture_or_subprefecture(number.area) then
            return { point = 0 }, "invalid number"
        end
        return { point = 1, multiplier = number.area }
    end,
}
