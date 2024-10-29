local util = require 'dbeer.util'
local core = require 'dbeer.core'
local setup = require 'dbeer'.SETTINGS
local popcorn = require 'popcorn'
local borders = require 'popcorn.borders'
local engines = require 'dbeer.engines'
local finders = require 'telescope.finders'
local pickers = require 'telescope.pickers'
local M = {}

local function get_file_line_info(filePath)
    local file = io.open(filePath, "r")
    if not file then
        print("Error: Could not open file.")
        return nil, nil
    end

    local first_line_length = nil
    local total_lines = 0

    for line in file:lines() do
        total_lines = total_lines + 1
        if total_lines == 1 then
            first_line_length = util.count_UTF8_characters(line)
        end
    end

    file:close()

    return first_line_length, total_lines
end

local function show_table_info(table_selected)
    util.logger:debug(table_selected)
    if table_selected == "" then
        return
    else
        table_selected = string.lower(table_selected)
    end

    local conn = (setup.db and setup.db.connections and setup.db.connections[require 'dbeer'.default_db]) or nil

    if not conn then
        return
    end

    local result = vim.fn.system(string.format(
        "%s -option 3 -engine %s -conn-str \"%s\" -queries %s -border-style %d -header-style-link %s -dbeer-log-file %s -dbname %s -log-debug %s",
        engines.db[conn.engine].executor, conn.engine, core.get_connection_string(), table_selected,
        setup.output.border_style,
        setup.output.header_style_link, util.dbeer_log_file, conn.dbname, setup.internal.log_debug))

    util.logger:debug(result)

    local line_1, dbeer_file

    for line in string.gmatch(result, "[^\r\n]+") do
        if not line_1 then
            line_1 = line
        elseif not dbeer_file then
            dbeer_file = line
            break
        end
    end

    if string.sub(line_1, 1, 7) ~= "[ERROR]" then
        if dbeer_file then
            local line_len, row_len = get_file_line_info(dbeer_file)
            local opts = {
                width = line_len + 4,
                height = row_len + 2,
                border = borders.simple_thick_border,
                title = { "  DBeer - Table Info", "Boolean" },
                footer = { ((conn.engine ~= "mongo" and "Table: ") or "Collection: ") .. string.upper(table_selected), "String" },
                content = dbeer_file,
                do_after = function()
                    vim.cmd [[ setlocal nowrap ]]
                    vim.cmd [[ setl noma ]]
                    vim.cmd(line_1)
                end
            }

            popcorn:new(opts):pop()
        else
            util.logger:error("Problem ocurred opening popup with table info.")
        end
    else
        util.logger:error(line_1)
    end
end

local function get_tables()
    local conn = (setup.db and setup.db.connections and setup.db.connections[require 'dbeer'.default_db]) or nil

    if not conn then
        return
    end
    local result = vim.fn.system(string.format(
        "%s -option 2 -engine %s -conn-str \"%s\" -dbeer-log-file %s -dbname %s -log-debug %s",
        engines.db[conn.engine].executor, conn.engine,
        core.get_connection_string(), util.dbeer_log_file, conn.dbname, setup.internal.log_debug))

    if result ~= "" then
        util.logger:debug(result)

        local str = result:gsub("%[", ""):gsub("%]", ""):gsub("^%s*(.-)%s*$", "%1"):gsub(",", "")

        local table_names = {}
        for word in str:gmatch("%S+") do
            table.insert(table_names, word)
        end
        return true, table_names
    else
        util.logger:error("Could not get system tables.")
        return false, nil
    end
end

function M.show()
    local ok, tables = get_tables()
    if not ok then
        return
    end

    pickers.new({
        prompt_title = "  DBeer - Table Picker",
        finder = finders.new_table({
            results = tables,
            entry_maker = function(entry)
                return {
                    value = entry,
                    display = entry,
                    ordinal = entry,
                }
            end,
        }),
        sorter = require 'telescope.sorters'.get_generic_fuzzy_sorter(),
        attach_mappings = function(_, map)
            map("i", "<CR>", function(prompt_bufnr)
                local selection = require("telescope.actions.state").get_selected_entry(prompt_bufnr)
                show_table_info(selection.value)
                --             require("telescope.actions").close(prompt_bufnr)
            end)
            return true
        end,
    }):find()
end

return M
