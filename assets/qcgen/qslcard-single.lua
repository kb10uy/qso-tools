---@param text string|nil
---@return string|nil
local function capitalize_if_upper(text)
    if text == nil or text == "" then
        return nil
    end
    if text:upper() ~= text then
        return text
    end
    return (text:lower():gsub("(%a)([%w']*)", function(head, tail)
        return head:upper() .. tail
    end))
end

---@param parks QslPark[]
---@return string|nil
local function format_parks(parks)
    local lines = {}
    for _, park in ipairs(parks) do
        local name = park.name_en or park.name_ja
        if name ~= nil then
            table.insert(lines, park.reference .. " " .. name)
        else
            table.insert(lines, park.reference)
        end
    end
    if #lines == 0 then
        return nil
    end
    return table.concat(lines, ", ")
end

---@param station QslStation
---@return string
local function format_location(station)
    local location = station.location
    local state = location.state and (location.state.name or location.state.code)
    local candidates = {
        capitalize_if_upper(location.city),
        state,
        capitalize_if_upper(location.country),
    }
    local parts = {}
    for i = 1, 3 do
        if candidates[i] ~= nil and candidates[i] ~= "" then
            table.insert(parts, candidates[i])
        end
    end
    local text = table.concat(parts, ", ")

    local parks = format_parks(station.references.pota)
    if parks ~= nil then
        text = text .. "\n" .. parks
    end
    return text
end

---@param args table<string,string>
---@param entries QslCardEntry[]
local function generate(args, entries)
    local converted_entries = {}
    for i, e in ipairs(entries) do
        local bureau_call = e.qso.call
        local call = e.qso.call
        local call_suffix_index = call:find("/")
        local via = false
        local datetime = e.qso.datetime
        local timezone = "UTC"

        if e.qsl.via then
            bureau_call = e.qsl.via
            via = true
        elseif call_suffix_index ~= nil then
            bureau_call = bureau_call:sub(1, call_suffix_index - 1)
        end

        if e.qso.mode ~= "FT8" then
            datetime = datetime:to_offset("+09:00")
            timezone = "JST"
        end

        table.insert(converted_entries, {
            via = via,
            station_call = e.station.callsign or "",
            bureau_call = bureau_call,
            call = call,
            date = datetime.date_str,
            time = datetime.time_str,
            timezone = timezone,
            report = e.exchange.tx_report or "",
            freq = e.qso.freq_str,
            mode = e.qso.mode,
            rig = e.instrument.rig or "",
            power = e.instrument.power and tostring(e.instrument.power) or "",
            antenna = e.instrument.antenna or "",
            location = format_location(e.station),
            operator = e.operator.name or "",
            operator_call = e.operator.callsign or "",
            received = e.qsl.received,
        })
    end

    return converted_entries
end

return {
    generate = generate,
}
