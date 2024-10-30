local logger = require 'dbeer.util'.logger
local setup = require 'dbeer'.SETTINGS
local db = setup.db

if db.connections then
    local connection = db.connections[require 'dbeer'.default_db]
    if connection.name and connection.dbname and connection.engine and require 'dbeer.engines'.db[connection.engine] and connection.engine == "mongo" then
        logger:info(string.format("Database set to [%s]", connection.name))
        vim.api.nvim_set_keymap('v', setup.commands.execute, '<cmd>lua require("dbeer.core").run()<CR>',
            { noremap = true, silent = true })
        vim.api.nvim_set_keymap('n', setup.commands.execute, '<cmd>lua require("dbeer.core").run()<CR>',
            { noremap = true, silent = true })
        vim.api.nvim_set_keymap('n', setup.commands.close, '<cmd>lua require("dbeer.core").close()<CR>',
            { noremap = true, silent = true })
    else
        pcall(vim.keymap.del, 'v', setup.commands.execute)
        pcall(vim.keymap.del, 'n', setup.commands.execute)
        pcall(vim.keymap.del, 'n', setup.commands.close)
    end
else
    logger:info("No database configured.")
end
