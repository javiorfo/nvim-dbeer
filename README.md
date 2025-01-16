# nvim-dbeer
*Minimal Multi database client for Neovim*

## Caveats
- These dependencies are required to be installed: `Go` (1.23), `unixodbc`. 
- For the sake of simplicity, **this plugin is STATELESS**. It does not use database sessions or keep states after Neovim is closed.
- This plugin has been developed on and for `Linux` following open source philosophy.

## Supported Databases
#### Databases not marked will be supported in the future

| Database | Supported | Integrated by | NOTE |
| ------- | ------------- | ------ | ---- |
| IBM DB2 | :heavy_check_mark: | ODBC | Supported operations detailed [here](#sql) |
| IBM Informix | :heavy_check_mark: | ODBC | Supported operations detailed [here](#sql) |
| MariaDB | :heavy_check_mark: | Go | Supported operations detailed [here](#sql) |
| MongoDB | :heavy_check_mark: | Go | Supported operations detailed [here](#nosql) |
| MS-SQL | :heavy_check_mark: | Go | Supported operations detailed [here](#sql) |
| MySQL | :heavy_check_mark: | Go | Supported operations detailed [here](#sql) |
| Neo4j | :x: | Go | Future release |
| Oracle | :heavy_check_mark: | Go | Supported operations detailed [here](#sql) |
| PostgreSQL | :heavy_check_mark: | Go | Supported operations detailed [here](#sql) |
| Redis | :x: | Go | Future release |
| SQLite | :heavy_check_mark: | Go | Supported operations detailed [here](#sql) |


## Demo

<img src="https://github.com/javiorfo/img/blob/master/nvim-dbeer/dbeer-demo.gif?raw=true" alt="nvim-dbeer"/>

**NOTE:** The colorscheme **nox** from [nvim-nyctophilia](https://github.com/javiorfo/nvim-nyctophilia) is used in this image.

---

## Table of Contents
- [Installation](#installation)
- [Configuration](#configuration)
- [Supported Operations](#supported-operations)
- [Usage](#usage)
- [Commands](#commands)
- [Tricks](#tricks)
- [Logs](#logs)

---

## Installation
`Lazy`
```lua
{ 
    'javiorfo/nvim-dbeer',
    dependencies = {
        'javiorfo/nvim-popcorn',
        'javiorfo/nvim-spinetta',
        'nvim-telescope/telescope.nvim',
        'nvim-lua/plenary.nvim',
    },
    lazy = true,
    cmd = { "DBeerBuild" },
    ft = { "sql", "javascript" }, -- javascript if MongoDB is used
    build = function()
        -- Update the backend in every plugin change
        require'dbeer.core'.build()
    end,
    opts = {
        -- This section is not required
        -- Only if you want to change default configurations
      
        -- Default keymaps
        commands = {
            -- Keymap in Normal mode to select DB with command :DBeerDB
            select_db = '<CR>',
        
            -- Keymap in Normal mode to expand and show connection data from DB with command :DBeerDB
            expand_db = '<C-space>',
            
            -- Keymap in Normal and Visual mode to execute a query
            execute = '<C-t>',
            
            -- Keymap in Normal mode to close all buffer results
            close = '<C-c>',
        },

        -- Command :DBeerDB
        view = {
            -- Show the user name
            show_user = true,
            
            -- Show the user password
            show_password = true,
        },

        -- Output buffer
        output = {
            -- Default dest folder where .dbeer files are created
            -- The results will be erased after closing the buffer
            -- If you want to keep the query results, change this to a personal folder
            dest_folder = "/tmp",

            -- Border style of the table result (1 to 6 to choose)
            -- Single border, rounded corners, double border, etc
            border_style = 1,

            -- A "hi link column style" in header table results
            header_style_link = "Type",

            -- Height of the buffer table result
            buffer_height = 20,

            -- Override the results buffer
            -- If false every query opens in a different buffer
            override = false,
        },

        -- Configuration of databases (host, port, credentials, etc)
        db = {
            -- Default DB when open a buffer
            default = 1,

            -- connections are left empty by default
            -- because these values are DB data connections set by the user
            -- connections = {}
        },

        -- For errors and debug purposes if anything goes wrong
        internal = {
            -- Disabled by default
            log_debug = false
        }
    }
}
```

---

## Configuration
#### Configure DB connections and credentials
- In the `setup` show above there is a section left out to be configured by the user (**connections** inside **db** table).
- Here are some examples of different DB configurations
- Engines possible values are: **mongo, postgres, oracle, mysql, sqlite, mssql, db2** and **informix**.

`Lazy`
```lua
opts = {
    db = {
        -- Here when open a sql file (or js file in Mongo case) connection will set to 2nd element (postgres)
        default = 2,
        
        -- Required fields are:
        -- name, engine and dbname
        -- host and port will be the default in each engine if not set
        -- user and password are optional
        connections = {
            {
                name = "MongoDB some name",
                engine = "mongo",
                host = "123.4.1.8",
                port = "27016",
                dbname = "db_dummy",
                user = "admin",
                password = "admin",
            },
            {
                name = "PostgreSQL example",
                engine = "postgres",
                dbname = "db_dummy",
                user = "admin",
                password = "admin",
            },
            {
                name = "Oracle example",
                engine = "oracle",
                dbname = "db_dummy",
                user = "admin",
                password = "admin",
            },
            {
                name = "MS-SQL example",
                engine = "mssql",
                dbname = "db_dummy",
            },
            {
                name = "SQLite example",
                engine = "sqlite",
                dbname = "/path/to/db_dummy.db", -- 'dbname' must contain the absolute path to the .db file or use ":memory:"
            },
            {
                name = "MySQL example",
                engine = "mysql", -- "mysql" also works for MariaDB 
                dbname = "db_dummy",
                user = "admin",
                password = "admin",
            },
            -- IBM Informix needs ODBC connection configured (check unix ODBC docs for this)
            {
                name = "Informix example",
                engine = "informix",
                dbname = "Informix_ODBC", -- 'dbname' must match your DSN
            },
            -- IBM DB2 needs ODBC connection configured (check unix ODBC docs for this)
            {
                name = "DB2 example",
                engine = "db2",
                dbname = "DB2_ODBC", -- 'dbname' must match your DSN
            },
        }
    }
}
```

- I personally recommend having connections in the same folder where the sql or js scripts are stored. So you can check or set connections in the same folder you have database scripts.
`Lazy`
```lua
opts = {
    db = dofile(os.getenv("HOME") .. "/path/to/connections.lua")

    -- connections.lua will have something like
    -- return {
    --     default = 1,
    --     connections = {
    --         {...} -- here complete the connection data
    --     }
    -- }
}
```

---

## Supported Operations
### Sql
- [x] All select and subselect queries
- [x] Commands insert, update, delete, create, modify, etc
- [ ] Comments (queries with comments could not be processed)
- Execution of multiple semicolon-separated queries
    - [x] Commands insert, update, delete, create, modify, etc
    - [ ] Select statements
- [x] Command to list tables
- [x] Command to get table info (fields, pk, fk, data type, etc)

### NoSql
- Operations
    - [x] "find" with filters and subsequet "sort", "skip" or "limit"
    - [x] "countDocuments"
    - [x] "findOne" with filters
    - [x] "insertOne"
    - [x] "deleteOne"
    - [x] "updateOne"
    - [x] "insertMany"
    - [x] "deleteMany"
    - [x] "updateMany"
    - [x] "drop" (drop collection)
    - [ ] Indexes operations
    - [ ] Replace operations
    - [ ] Rename operations
    - [ ] Aggregate operations
- [ ] Comments (queries with comments could not be processed)
- [ ] Execution of multiple semicolon-separated queries
- [x] Command to list collections
- [x] Command to get collection info (fields, data type, etc)

##### Example
```javascript
db.mycollection.find({ "field1": "value1" }).sort({"info": -1})

// "db." is optional in nvim-dbeer. This will work too
mycollection.find({ "field1": "value1" }).sort({"info": -1})
```

<img src="https://github.com/javiorfo/img/blob/master/nvim-dbeer/dbeer-mongo.gif?raw=true" alt="nvim-dbeer"/>

**NOTE:** The colorscheme **nox** from [nvim-nyctophilia](https://github.com/javiorfo/nvim-nyctophilia) is used in this image.

---

## Usage
- When a **sql file** or **js file** (in case of Mongo) is opened, Neovim will print what connection is set by default in nvim-dbeer. The connection to the database is done when the query is executed (open connection, execute statement, close connection), no session is set.
- The keymap `<C-t>` (could be modified by the user, see config above) if executed in **NORMAL mode** will take all the script (semicolon-separated) to process. But maybe it's best to execute it in **VISUAL mode** getting the same experience of a stardard DB IDE where a query can be selected and execute it in isolation instead of the entire script.

---

## Commands
### DBeerBuild
- This is executed when this plugin receives and update, not necessary to be executed manually except if nvim-dbeer informs it.

### DBeerLogs
- Show the logs

### DBeerDB
- Command to change the DB connection to another one.
- Show a expanded info of the DB (name, engine, host, port, credentials)

<img src="https://github.com/javiorfo/img/blob/master/nvim-dbeer/dbeer-selectdb.gif?raw=true" alt="nvim-dbeer"/>

**NOTE:** The colorscheme **nox** from [nvim-nyctophilia](https://github.com/javiorfo/nvim-nyctophilia) is used in this image.

### DBeerTables
- This command uses telescope.nvim showing all the tables of the selected database
- If you press enter after a table was selected, a popup show the "selected table" info

<img src="https://github.com/javiorfo/img/blob/master/nvim-dbeer/dbeer-tableinfo.gif?raw=true" alt="nvim-dbeer"/>

**NOTE:** The colorscheme **nox** from [nvim-nyctophilia](https://github.com/javiorfo/nvim-nyctophilia) is used in this image.

---

## Tricks
As **nvim-dbeer** resets the default database connection everytime Neovim is close. There is a chance to set N scripts to X database everytime threy are opened. Using **Neovim autocmd** in **init.lua**:
```lua
-- Everytime test.sql, other.sql, update.sql are opened the default database is set to '2' 
vim.api.nvim_create_autocmd("BufReadPost", {
    pattern = { "test.sql", "other.sql", "update.sql" },
    callback = function()
        require 'dbeer'.default_db = 2
    end,
})
```

---

## Logs
Logs are saved generally in this path: **/home/your_user/.local/state/nvim/dbeer.log**

- To check the logs execute the command `:dbeerLogs`

**NOTE**: Only error logs are saved. If you want to enable debug phase, enable this on setup configuration:
```lua
require'dbeer'.setup {
    internal = {
       log_debug = true 
   }
}
```

---

## Screenshots
#### Example executing the entire script (not select allowed) semicolon-separated
- Note that in the fourth statement there is a duplicated primary key error reported
<img src="https://github.com/javiorfo/img/blob/master/nvim-dbeer/dbeer-multi.png?raw=true" alt="nvim-dbeer"/>

#### Example example of border style 4 in table result
<img src="https://github.com/javiorfo/img/blob/master/nvim-dbeer/dbeer-border.png?raw=true" alt="nvim-dbeer"/>

**NOTE:** The colorscheme **nox** from [nvim-nyctophilia](https://github.com/javiorfo/nvim-nyctophilia) is used in this image.

---

### Donate
- **Bitcoin** [(QR)](https://raw.githubusercontent.com/javiorfo/img/master/crypto/bitcoin.png)  `1GqdJ63RDPE4eJKujHi166FAyigvHu5R7v`
- [Paypal](https://www.paypal.com/donate/?hosted_button_id=FA7SGLSCT2H8G)
