package engine

import (
	"fmt"

	_ "github.com/denisenkom/go-mssqldb"
	"github.com/javiorfo/nvim-dbeer/go/database/engine/model"
)

type MSSql struct {
	model.ProtoSQL
}

func (ms *MSSql) GetTables() {
	ms.Queries = "SELECT name AS table_name FROM sys.tables order by name;"
	ms.ProtoSQL.GetTables()
}

func (ms *MSSql) GetTableInfo() {
	db, closer, err := ms.GetDB()
	if err != nil {
		fmt.Print(err.Error())
		return
	}
	defer closer()

	ms.Queries = ms.GetTableInfoQuery(ms.Queries)
	ms.ExecuteSelect(db)
}

func (ms *MSSql) GetTableInfoQuery(tableName string) string {
	return `SELECT 
                UPPER(c.COLUMN_NAME) AS column_name,
                c.DATA_TYPE,
                CASE
                    WHEN c.IS_NULLABLE = 'YES' THEN NCHAR(0xE640) + ' '
                    ELSE NCHAR(0xF4A7) + ' '
                END AS not_null,
                CASE
                    WHEN c.CHARACTER_MAXIMUM_LENGTH IS NULL THEN '-'
                    ELSE CAST(c.CHARACTER_MAXIMUM_LENGTH AS VARCHAR)
                END AS length,
                CASE  
                    WHEN tc.CONSTRAINT_TYPE = 'PRIMARY KEY' THEN NCHAR(0xEB11) + '  PRIMARY KEY'
                    WHEN tc.CONSTRAINT_TYPE = 'FOREIGN KEY' THEN NCHAR(0xEB11) + '  FOREIGN KEY'
                    ELSE '-'
                END AS constraint_type,
                CASE 
                    WHEN tc.CONSTRAINT_TYPE = 'FOREIGN KEY' THEN 
                        NCHAR(0xEBB7) + ' ' + kcu2.TABLE_NAME + '.' + kcu2.COLUMN_NAME
                    ELSE 
                        '-'
                END AS referenced_table_column
            FROM 
                INFORMATION_SCHEMA.COLUMNS AS c
            LEFT JOIN 
                INFORMATION_SCHEMA.KEY_COLUMN_USAGE AS kcu 
                ON c.COLUMN_NAME = kcu.COLUMN_NAME 
                AND c.TABLE_NAME = kcu.TABLE_NAME
            LEFT JOIN 
                INFORMATION_SCHEMA.TABLE_CONSTRAINTS AS tc 
                ON kcu.CONSTRAINT_NAME = tc.CONSTRAINT_NAME 
                AND kcu.TABLE_NAME = tc.TABLE_NAME
            LEFT JOIN 
                INFORMATION_SCHEMA.REFERENTIAL_CONSTRAINTS AS rc 
                ON tc.CONSTRAINT_NAME = rc.CONSTRAINT_NAME 
                AND tc.TABLE_SCHEMA = rc.UNIQUE_CONSTRAINT_SCHEMA
            LEFT JOIN 
                INFORMATION_SCHEMA.KEY_COLUMN_USAGE AS kcu2 
                ON rc.UNIQUE_CONSTRAINT_NAME = kcu2.CONSTRAINT_NAME 
                AND rc.UNIQUE_CONSTRAINT_SCHEMA = kcu2.TABLE_SCHEMA
            WHERE 
                c.TABLE_NAME = '` + tableName + `';`
}
