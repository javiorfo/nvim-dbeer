local util = require 'dbeer.util'

local host = "127.0.0.1"
local default_posgres_port = "5432"
local default_mongo_port = "27017"
local default_redis_port = "6379"
local default_mysql_port = "3306"
local exe = util.dbeer_root_path .. "bin/dbeer"

local function odbc(connection)
    return string.format("DSN=%s;%s%s",
        connection.dbname,
        connection.user and "UID=" .. connection.user .. ";" or "",
        connection.password and "PWD=" .. connection.password .. ";" or ""
    )
end

return {
    db = {
        postgres = {
            title = "PostgreSQL",
            default_port = default_posgres_port,
            default_host = host,
            executor = exe,
            get_connection_string = function(connection)
                return string.format("host=%s port=%s dbname=%s %s %s sslmode=disable",
                    connection.host or host,
                    connection.port or default_posgres_port,
                    connection.dbname,
                    connection.user and "user=" .. connection.user or "",
                    connection.password and "password=" .. connection.password or ""
                )
            end
        },
        mysql = {
            title = "MySQL",
            default_port = default_mysql_port,
            default_host = host,
            executor = exe,
            get_connection_string = function(connection)
                return string.format("mysql://%s%s%s:%s/%s",
                    connection.user and connection.user .. ":" or "",
                    connection.password and connection.password .. "@" or "",
                    connection.host or host,
                    connection.port or default_mysql_port,
                    connection.dbname
                )
            end
        },
        mongo = {
            title = "MongoDB",
            default_port = default_mongo_port,
            default_host = host,
            executor = exe,
            get_connection_string = function(connection)
                return string.format("mongodb://%s%s%s:%s/%s",
                    connection.user and connection.user or "",
                    connection.password and ":" .. connection.password .. "@" or "",
                    connection.host or host,
                    connection.port or default_mongo_port,
                    connection.dbname
                )
            end
        },
        redis = {
            title = "Redis",
            default_port = default_redis_port,
            default_host = host,
            executor = exe,
            get_connection_string = function(connection)
                return string.format("redis://%s%s%s:%s",
                    connection.user and connection.user or "",
                    connection.password and ":" .. connection.password .. "@" or "",
                    connection.host or host,
                    connection.port or default_redis_port
                )
            end
        },
        mssql = {
            title = "MS-SQL",
            default_port = "-",
            default_host = "-",
            executor = exe,
            get_connection_string = odbc
        },
        oracle = {
            title = "Oracle",
            default_port = "-",
            default_host = "-",
            executor = exe,
            get_connection_string = odbc
        },
        informix = {
            title = "Informix",
            default_port = "-",
            default_host = "-",
            executor = exe,
            get_connection_string = odbc
        },
        db2 = {
            title = "DB2",
            default_port = "-",
            default_host = "-",
            executor = exe,
            get_connection_string = odbc
        },
        sqlite = {
            title = "SQLite",
            default_port = "-",
            default_host = "-",
            executor = exe,
            get_connection_string = function(connection)
                return connection.dbname
            end
        },
    }
}
