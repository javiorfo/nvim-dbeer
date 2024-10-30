local logger = require 'dbeer.util'.logger
local setup = require 'dbeer'.SETTINGS
local db = setup.db

if db.connections then
    local connection = db.connections[require 'dbeer'.default_db]
    if connection.name and connection.dbname and connection.engine and require 'dbeer.engines'.db[connection.engine] then
        logger:info(string.format("Database set to [%s]", connection.name))
    end
else
    logger:info("No database configured.")
end
