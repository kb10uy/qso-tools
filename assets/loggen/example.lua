---@type ContestDefinition
return {
    name = "Example Contest",
    categories = {
        XAM = { name = "Mixed, single operator, all bands" },
        C7M = { name = "CW, single operator, 7MHz", modes = { "CW" }, bands = { "40m" } },
        PN = { name = "Phone, newcomer", modes = { "PH" }, exclude_bands = { "20m" }, requires = { "license_date" } },
    },
    dupe_scope = { "band" },
    multiplier_scope = { "band" },

    score_qso = function(entry, ctx)
        local area = string.match(entry.exchange.rx_number or "", "^(%d+)[PLMH]$")
        if area == nil then
            return { point = 0 }, "invalid number"
        end
        return {
            point = string.sub(area, 1, 2) == "15" and 2 or 1,
            multiplier = area,
        }
    end,
}
