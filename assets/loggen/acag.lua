local jarl = require("jarl")

local bands = {
    { code = "19", label = "1.9MHz", band = "160m", cabrillo = "160M", power = true },
    { code = "35", label = "3.5MHz", band = "80m", cabrillo = "80M", power = true },
    { code = "7", label = "7MHz", band = "40m", cabrillo = "40M", power = true },
    { code = "14", label = "14MHz", band = "20m", cabrillo = "20M", power = true },
    { code = "21", label = "21MHz", band = "15m", cabrillo = "15M", power = true },
    { code = "28", label = "28MHz", band = "10m", cabrillo = "10M", power = true },
    { code = "50", label = "50MHz", band = "6m", cabrillo = "6M", power = true },
    { code = "144", label = "144MHz", band = "2m", cabrillo = "2M" },
    { code = "430", label = "430MHz", band = "70cm", cabrillo = "432" },
    { code = "1200", label = "1200MHz", band = "23cm", cabrillo = "1.2G" },
    { code = "2400", label = "2400MHz", band = "13cm", cabrillo = "2.3G" },
    { code = "5600", label = "5600MHz", band = "6cm", cabrillo = "5.7G" },
}
local powers = {
    H = { label = "100W超", cabrillo = "HIGH" },
    M = { label = "100W以下", cabrillo = "LOW" },
    P = { label = "5W以下", cabrillo = "QRP" },
}
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
    for power, p in pairs(powers) do
        categories[prefix .. "A" .. power] = category(
            string.format("%s部門 シングルオペ オールバンド %s", mode.name, p.label),
            { operator = "SINGLE-OP", band = "ALL", mode = mode.cabrillo, power = p.cabrillo },
            { modes = mode.groups }
        )
        if power ~= "P" then
            categories[prefix .. "MA" .. power] = category(
                string.format("%s部門 マルチオペ オールバンド %s", mode.name, p.label),
                { operator = "MULTI-OP", band = "ALL", mode = mode.cabrillo, power = p.cabrillo },
                { modes = mode.groups, requires = { "multi_op_list" } }
            )
        end
    end

    for _, b in ipairs(bands) do
        local name = string.format("%s部門 シングルオペ %sバンド", mode.name, b.label)
        local cabrillo = { operator = "SINGLE-OP", band = b.cabrillo, mode = mode.cabrillo }
        if b.power then
            for power, p in pairs(powers) do
                local powered = { operator = cabrillo.operator, band = cabrillo.band, mode = cabrillo.mode, power = p.cabrillo }
                categories[prefix .. b.code .. power] = category(name .. " " .. p.label, powered, { modes = mode.groups, bands = { b.band } })
            end
        else
            categories[prefix .. b.code] = category(name, cabrillo, { modes = mode.groups, bands = { b.band } })
        end
    end
end

for _, b in ipairs(bands) do
    if b.power and b.code ~= "14" then
        categories["P" .. b.code] = category(
            string.format("電話部門 シングルオペ %sバンド", b.label),
            { operator = "SINGLE-OP", band = b.cabrillo, mode = "SSB" },
            { modes = { "PH" }, bands = { b.band } }
        )
    end
end

---@type ContestDefinition
return {
    name = "全市全郡コンテスト",
    contest_id = "JARL-ACAG",
    categories = categories,
    dupe_scope = { "band" },
    multiplier_scope = { "band" },

    score_qso = function(entry)
        local number = jarl.parse_number(entry.exchange.rx_number)
        local jcx = number and jarl.jcx(number.area)
        if jcx == nil or jcx.kind == "prefecture" then
            return { point = 0 }, "invalid number"
        end
        if jcx.has_wards then
            return { point = 1 }, "city divided into wards is not a multiplier"
        end
        return { point = 1, multiplier = number.area }
    end,
}
