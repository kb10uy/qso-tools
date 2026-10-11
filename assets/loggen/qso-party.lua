---@type ContestDefinition
return {
    name = "QSOパーティ",
    categories = {
        ["30"] = { name = "アマチュア局" },
    },
    dupe_scope = {},

    score_qso = function(entry, ctx)
        return {
            point = 1,
            tx_number = ctx.args.my_name,
            rx_number = entry.name or "-",
        }
    end,

    total = function(summary)
        return summary.qsos
    end,
}
