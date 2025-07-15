local setup = require 'dbeer'.SETTINGS
local engines = require 'dbeer.engines'
local util = require 'dbeer.util'
local spinetta = require 'spinetta'
local M = {}

function M.init_msg()
    local db = setup.db
    local logger = require 'dbeer.util'.logger

    if db.connections then
        local connection = db.connections[require 'dbeer'.default_db]
        if connection.name and connection.dbname and connection.engine and require 'dbeer.engines'.db[connection.engine] then
            logger:info(string.format("Database set to [%s]", connection.name))
        end
    else
        logger:info("No database configured.")
    end
end

function M.get_connection_string()
    local db = setup.db
    if db.connections then
        local connection = db.connections[require 'dbeer'.default_db]
        local conn_str = engines.db[connection.engine].get_connection_string(connection)
        util.logger:debug(conn_str)
        return conn_str
    end
end

local function get_buffer_content()
    local mode = vim.api.nvim_get_mode().mode

    if mode == 'v' or mode == 'V' or mode == '\22' then
        vim.cmd("normal " .. vim.api.nvim_replace_termcodes("<esc>", true, true, true))
        local start_pos = vim.fn.getpos("'<")
        local end_pos = vim.fn.getpos("'>")

        local lines = vim.api.nvim_buf_get_lines(0, start_pos[2] - 1, end_pos[2], false)
        if #lines == 0 then
            return ""
        end

        if start_pos[2] == end_pos[2] then
            return lines[1]:sub(start_pos[3], end_pos[3])
        else
            lines[1] = lines[1]:sub(start_pos[3])
            lines[#lines] = lines[#lines]:sub(1, end_pos[3])
            return table.concat(lines, " ")
        end
    else
        local buf_number = vim.api.nvim_get_current_buf()
        local lines = vim.api.nvim_buf_get_lines(buf_number, 0, -1, false)
        local content = table.concat(lines, " ")
        return content
    end
end

function M.run()
    local file_with_extension = vim.fn.expand("%:p")
    if setup.output.override then
        M.close()
    end

    local queries = get_buffer_content()
    local conn = (setup.db and setup.db.connections and setup.db.connections[require 'dbeer'.default_db]) or nil

    if not conn then
        return
    end

    local dest_folder = setup.output.dest_folder
    local format_query = ((conn.engine == "mongo" or conn.engine == "redis") and "'%s'") or '\"%s\"'
    local script = string.format(
        "%s -engine %s -conn-str \"%s\" -queries " ..
        format_query ..
        " -dest-folder %s -border-style %d -header-style-link %s -dbeer-log-file %s -dbname %s -log-debug %s",
        engines.db[conn.engine].executor, conn.engine, M.get_connection_string(), queries, dest_folder,
        setup.output.border_style,
        setup.output.header_style_link, util.dbeer_log_file, conn.dbname, setup.internal.log_debug)

    util.logger:debug(script)
    local result = {}
    local elapsed_time = 0
    local spinner = spinetta:new {
        main_msg = "  DBeer   Executing query ",
        speed_ms = 200,
        spinner = util.get_numeral_sprinner(),
        on_success = function()
            util.logger:debug("results from backend: ", vim.inspect(result))
            if #result == 0 then
                util.logger:error("Internal error")
                return
            end

            if string.sub(result[1], 1, 7) ~= "[ERROR]" then
                if result[2] then
                    vim.cmd(string.format("%dsp %s", setup.output.buffer_height, result[2]))
                    vim.cmd("setlocal nowrap")
                    vim.cmd("setlocal noma")
                    util.logger:info(string.format("  Query executed correctly [%.2f secs]", elapsed_time))
                    vim.cmd(result[1])
                else
                    util.logger:info(result[1])
                end
            else
                util.logger:error(result[1])
            end
        end,
        on_interrupted = function()
            vim.cmd("redraw")
            util.logger:info("Process cancelled by the user")
        end
    }

    local function job_to_run(command)
        local output = {}
        local start_time = os.time()
        local job_id = vim.fn.jobpid(vim.fn.jobstart(command, {
            on_stdout = function(_, data, _)
                for _, line in ipairs(data) do
                    if line ~= "" then
                        table.insert(output, line)
                    end
                end
            end,
            on_exit = function(_, _)
                result = output
                local end_time = os.time()
                elapsed_time = os.difftime(end_time, start_time)
            end,
        }))
        return spinetta.break_when_pid_is_complete(job_id)
    end

    spinner:start(job_to_run(script))

    vim.cmd(vim.fn.bufwinnr(file_with_extension) .. " wincmd w")
end

function M.build()
    if vim.fn.executable("cargo") == 0 then
        util.logger:warn("Cargo (Rust) is required. Install it to use this plugin and then execute manually :dbeerBuild")
        return false
    end

    local root_path = util.dbeer_root_path
    local script = string.format(
        "%sscript/build.sh %s 2> >( while read line; do echo \"[ERROR][$(date '+%%m/%%d/%%Y %%T')]: ${line}\"; done >> %s)",
        root_path,
        root_path, util.dbeer_log_file)
    local spinner = spinetta:new {
        main_msg = "  DBeer   Building plugin... Rust build could take some time ",
        speed_ms = 100,
        on_success = function()
            util.logger:info("  Plugin ready to be used!")
        end
    }

    spinner:start(spinetta.job_to_run(script))
end

function M.close()
    for _, nr in ipairs(vim.api.nvim_list_bufs()) do
        local buf_name = vim.api.nvim_buf_get_name(nr)
        if vim.api.nvim_buf_is_loaded(nr) and (buf_name:find(".dbeer$") or buf_name:find(".dbeer.json$")) then
            vim.cmd("bd! " .. buf_name)
        end
    end
    if setup.output.dest_folder == "/tmp" then
        os.execute("rm -f /tmp/*.dbeer*")
    end
end

function M.show_logs()
    vim.cmd(string.format("vsp %s | normal G", util.dbeer_log_file))
end

return M
