---@meta prompt

---@class prompt
local prompt = {}

--- Asks the user for text.
---@param text string Prompt text.
---@param default string|nil Default value.
---@return string Input.
function prompt.text(text, default) end

--- Asks the user for integer.
---@param text string Prompt text.
---@param default integer|nil Default value.
---@return integer Input.
function prompt.integer(text, default) end

--- Asks the user for float.
---@param text string Prompt text.
---@param default number|nil Default value.
---@return number Input.
function prompt.float(text, default) end

--- Asks the user to select one of choices.
---@param text string Prompt text.
---@param choices string[] Choices.
---@param default integer|nil Default value index.
---@return string Selected choice.
function prompt.choice(text, choices, default) end

--- Asks the user for confirmation.
---@param text string Prompt text.
---@param default boolean|nil Default value.
---@return boolean Input.
function prompt.confirm(text, default) end
