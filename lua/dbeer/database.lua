local popcorn = require 'popcorn'
local borders = require 'popcorn.borders'
local constants = require 'dbeer.constants'
local engines = require 'dbeer.engines'
local setup = require 'dbeer'.SETTINGS
local util = require 'dbeer.util'

local M = {}

function M.show()
    local content = {}
    local db = setup.db
    local default = require 'dbeer'.default_db
    if db.connections then
        content = { { "󰆼 Database", "Type" } }
        for i, v in pairs(db.connections) do
            local name = constants.UNCHECKED_ICON .. v.name
            if i == default then
                name = constants.CHECKED_ICON .. v.name
            end
            table.insert(content, { name })
        end
    end

    if #content == 0 then
        content = { { "No databases available", "ErrorMsg" } }
    end

    local popup_opts = {
        width = 50,
        height = 30,
        border = borders.simple_thick_border,
        title = { "  DBeer - Select DB", "Boolean" },
        footer = { setup.commands.select_db .. " to select", "String" },
        content = content,
        do_after = function()

            local expand_db_command = setup.commands.expand_db
            local line = string.format("%s (press %s to toggle)", vim.fn.getline(1), expand_db_command)
            vim.fn.setline(1, line)

            vim.cmd("syn match dbeerConnData '󱘖 Connection Data' | hi link dbeerConnData Type")
            vim.cmd(string.format("syn match dbeerExpand '(press %s to toggle)' | hi link dbeerExpand Comment", expand_db_command))

            util.disable_editing_popup()

            if #content > 0 then
                vim.api.nvim_win_set_cursor(0, { default + 1, 0 })
                vim.api.nvim_buf_set_keymap(0, 'n', setup.commands.select_db,
                    '<cmd>lua require("dbeer.database").set()<CR>', { noremap = true, silent = true })
                vim.api.nvim_buf_set_keymap(0, 'n', expand_db_command,
                    '<cmd>lua require("dbeer.database").expand()<CR>', { noremap = true, silent = true })

                vim.api.nvim_create_autocmd({ "CursorMoved" }, {
                    pattern = { "<buffer>" },
                    callback = function()
                        local pos = vim.api.nvim_win_get_cursor(0)
                        if pos[1] < 2 then
                            vim.api.nvim_win_set_cursor(0, { 2, 0 })
                        end
                        if pos[2] > 0 then
                            vim.api.nvim_win_set_cursor(0, { pos[1], 0 })
                        end
                    end
                })
            end
        end
    }
    popcorn:new(popup_opts):pop()
end

local function select_or_unselect(lines, line_nr)
    for _, v in pairs(lines) do
        if v == line_nr then
            local selected = vim.fn.getline('.')
            local final = tostring(selected):gsub(constants.UNCHECKED_ICON, constants.CHECKED_ICON)
            vim.fn.setline(line_nr, final)
            require 'dbeer'.default_db = v - 1
            local connection = setup.db.connections[v - 1]
            util.logger:info(string.format("Database set to [%s]", connection.name))
        else
            local unselected = vim.fn.getline(v)
            local final = tostring(unselected):gsub(constants.CHECKED_ICON, constants.UNCHECKED_ICON)
            vim.fn.setline(v, final)
        end
    end
end

function M.set()
    vim.cmd [[setl ma]]
    local line_nr = vim.fn.line('.')
    local len = #setup.db.connections + 2

    if line_nr > 1 and line_nr < len then
        local lines = {}
        for i = 2, len do table.insert(lines, i) end
        select_or_unselect(lines, line_nr)
    end

    vim.cmd [[setl noma]]
end

function M.expand()
    local line_nr = vim.fn.line('.')
    local lines = vim.api.nvim_buf_get_lines(0, 0, -1, false)
    if string.find(lines[line_nr], constants.CHECKED_ICON) or string.find(lines[line_nr], constants.UNCHECKED_ICON) then
        vim.cmd [[setl ma]]
        if lines[line_nr + 1] == "     󱘖 Connection Data" then
            if string.find(lines[line_nr + 2], "DSN") then
                for _ = 1, 3 do
                    table.remove(lines, line_nr + 1)
                end
            else
                for _ = 1, 8 do
                    table.remove(lines, line_nr + 1)
                end
            end
        else
            table.insert(lines, line_nr + 1, "     󱘖 Connection Data")
            local connection = {}
            local connection_name = lines[line_nr]:gsub(constants.UNCHECKED_ICON, ""):gsub(constants.CHECKED_ICON, "")

            for _, v in ipairs(setup.db.connections) do
                if v.name == connection_name then
                    connection = v
                    break
                end
            end

            local db_const_data = engines.db[connection.engine]
            if connection.dbname == "odbc" then
                table.insert(lines, line_nr + 2, "       DSN      󰁕 " .. connection.name .. " (ODBC)")
                table.insert(lines, line_nr + 3, "")
            else
                table.insert(lines, line_nr + 2, "       NAME     󰁕 " .. connection.name)
                table.insert(lines, line_nr + 3, "       HOST     󰁕 " .. (connection.host or db_const_data.default_host))
                table.insert(lines, line_nr + 4, "       PORT     󰁕 " .. (connection.port or db_const_data.default_port))
                table.insert(lines, line_nr + 5, "       DB NAME  󰁕 " .. connection.dbname)
                table.insert(lines, line_nr + 6,
                    "       USER     󰁕 " ..
                    (((connection.user and setup.view.show_user and connection.user) or connection.user and "********") or "-"))
                table.insert(lines, line_nr + 7,
                    "       PASSWORD 󰁕 " ..
                    (((connection.password and setup.view.show_password and connection.password) or connection.password and "********") or "-"))
                table.insert(lines, line_nr + 8, "")
            end
        end
        vim.api.nvim_buf_set_lines(0, 0, -1, false, lines)
        vim.cmd [[setl noma]]
    end
end

return M
