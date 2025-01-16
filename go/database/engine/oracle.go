package engine

import (
	"fmt"

    _ "github.com/sijms/go-ora/v2"
	"github.com/javiorfo/nvim-dbeer/go/database/engine/model"
)

type Oracle struct {
	model.ProtoSQL
}

func (o *Oracle) GetTables() {
	o.Queries = "select table_name from all_tables where owner = 'PUBLIC' order by table_name;"
	o.ProtoSQL.GetTables()
}

func (o *Oracle) GetTableInfo() {
	db, closer, err := o.GetDB()
	if err != nil {
		fmt.Print(err.Error())
		return
	}
	defer closer()

	o.Queries = o.GetTableInfoQuery(o.Queries)
	o.ExecuteSelect(db)
}

func (o *Oracle) GetTableInfoQuery(tableName string) string {
	return `SELECT 
                UPPER(c.column_name) AS column_name,
                c.data_type,
                CASE
                    WHEN c.nullable = 'Y' THEN ' '
                    ELSE ' '
                END AS not_null,
                CASE
                    WHEN c.data_type IN ('VARCHAR2', 'CHAR', 'NCHAR', 'NVARCHAR2') THEN 
                        COALESCE(c.data_length, '-') 
                    ELSE 
                        '-'
                END AS length,
                CASE  
                    WHEN con.constraint_type = 'P' THEN '  PRIMARY KEY'
                    WHEN con.constraint_type = 'R' THEN '  FOREIGN KEY'
                    ELSE '-'
                END AS constraint_type,
                CASE 
                    WHEN con.constraint_type = 'R' THEN 
                        '  ' || rcc.table_name || '.' || rcc.column_name
                    ELSE 
                        '-'
                END AS referenced_table_column
                FROM user_tab_columns c
                LEFT JOIN user_cons_columns kcu ON c.column_name = kcu.column_name 
                    AND c.table_name = kcu.table_name
                LEFT JOIN user_constraints con ON kcu.constraint_name = con.constraint_name 
                    AND kcu.table_name = con.table_name
                LEFT JOIN user_constraints rc ON con.constraint_name = rc.constraint_name 
                    AND con.table_name = rc.table_name AND con.constraint_type = 'R'
                LEFT JOIN user_cons_columns rcc ON rc.r_constraint_name = rcc.constraint_name
                WHERE c.table_name = '` + tableName + `';`
}
