local M = {}
local util = require 'dbeer.util'
local engines = require 'dbeer.engines'

M.SETTINGS = {
    commands = {
        select_db = '<CR>',
        expand_db = '<C-space>',
        execute = '<C-t>',
        close = '<C-c>',
    },
    view = {
        show_user = true,
        show_password = true,
    },
    output = {
        dest_folder = "/tmp",
        border_style = 1,
        header_style_link = "Type",
        buffer_height = 20,
        override = false
    },
    db = {
        default = 1,
    },
    internal = {
        log_debug = false
    }
}

M.default_db = M.SETTINGS.db.default

local function validate_default_connection(connections, index)
    if connections then
        return connections[index] ~= nil
    else
        return false
    end
end

function M.setup(opts)
    if opts.commands then
        local commands = opts.commands
        if commands.select_db then
            M.SETTINGS.commands.select_db = (type(commands.select_db) == "string" and commands.select_db) or
                M.SETTINGS.commands.select_db
        end
        if commands.expand_db then
            M.SETTINGS.commands.expand_db = (type(commands.expand_db) == "string" and commands.expand_db) or
                M.SETTINGS.commands.expand_db
        end
        if commands.execute then
            M.SETTINGS.commands.execute = (type(commands.execute) == "string" and commands.execute) or
                M.SETTINGS.commands.execute
        end
        if commands.close then
            M.SETTINGS.commands.close = (type(commands.close) == "string" and commands.close) or
                M.SETTINGS.commands.close
        end
    end
    if opts.view then
        local view = opts.view
        if view.show_user ~= nil then
            M.SETTINGS.view.show_user = (type(view.show_user) == "boolean" and view.show_user) or
                M.SETTINGS.view.show_user
        end
        if view.show_password ~= nil then
            M.SETTINGS.view.show_password = (type(view.show_password) == "boolean" and view.show_password) or
                M.SETTINGS.view.show_password
        end
    end
    if opts.output then
        local op = opts.output
        if op.dest_folder then
            M.SETTINGS.output.dest_folder = (type(op.dest_folder) == "string" and op.dest_folder) or
                M.SETTINGS.output.dest_folder
        end
        if op.header_style_link then
            M.SETTINGS.output.header_style_link = (type(op.header_style_link) == "string" and op.header_style_link) or
                M.SETTINGS.output.header_style_link
        end
        if op.border_style then
            M.SETTINGS.output.border_style = (type(op.border_style) == "number" and op.border_style > 0 and op.border_style < 6 and op.border_style) or
                M.SETTINGS.output.border_style
        end
        if op.buffer_height then
            M.SETTINGS.output.buffer_height = (type(op.buffer_height) == "number" and op.buffer_height > 10 and op.buffer_height < 90 and op.buffer_height) or
                M.SETTINGS.output.buffer_height
        end
        if op.override ~= nil then
            M.SETTINGS.output.override = (type(op.override) == "boolean" and op.override) or
                M.SETTINGS.output.override
        end
    end
    if opts.db then
        local db = opts.db
        if db.default then
            M.default_db = (type(db.default) == "number" and validate_default_connection(db.connections, db.default) and db.default) or
                M.SETTINGS.db.default
        end
        if db.connections then
            for i, conn in pairs(db.connections) do
                if not conn.name then
                    util.logger:warn("db.connections.name missing in connection " .. i)
                end
                if not conn.dbname then
                    util.logger:warn("db.connections.dbname missing in connection " .. i)
                end
                if not conn.engine then
                    util.logger:warn("db.connections.engine missing in connection " .. i)
                elseif not engines.db[conn.engine] then
                    util.logger:warn(string.format("%s engine is not available in connection %d", conn.engine, i))
                end
            end
            M.SETTINGS.db.connections = (type(db.connections) == "table" and #db.connections > 0 and type(db.connections[1]) == "table" and db.connections)
        end
    end
    if opts.internal ~= nil then
        local int = opts.internal
        M.SETTINGS.internal.log_debug = (type(int.log_debug) == "boolean" and int.log_debug) or
            M.SETTINGS.internal.log_debug
    end

    util.logger:debug("Configuration: " .. vim.inspect(M.SETTINGS))
end

return M
