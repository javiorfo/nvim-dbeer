package model

import (
	"database/sql"
	"fmt"
	"strings"
	"unicode/utf8"

	"github.com/araddon/dateparse"
	"github.com/javiorfo/nvim-dbeer/go/database/query"
	"github.com/javiorfo/nvim-dbeer/go/database/table"
	"github.com/javiorfo/nvim-dbeer/go/logger"
)

type ProtoSQL struct {
	Engine          string
	ConnStr         string
	DbName          string
	Queries         string
	BorderStyle     int
	DestFolder      string
	HeaderStyleLink string
	Option          Option
}

type Option int

const (
	RUN Option = iota + 1
	TABLES
	TABLE_INFO
	PING
)

func (p *ProtoSQL) GetDB() (*sql.DB, func(), error) {
	db, err := sql.Open(p.Engine, p.ConnStr)
	if err != nil {
		logger.Errorf("Error initializing %s, connStr: %s", p.Engine, p.ConnStr)
		return nil, nil, fmt.Errorf("[ERROR] %v", err)
	}
	return db, func() { db.Close() }, nil
}

func (p *ProtoSQL) Ping() {
	db, closer, err := p.GetDB()
	if err != nil {
		fmt.Print(err.Error())
		return
	}
	defer closer()

	err = db.Ping()
	if err != nil {
		fmt.Printf("[ERROR] %v", err)
		return
	}

	fmt.Println("Successfully connected to the database!")
}

func (p *ProtoSQL) Run() {
	db, closer, err := p.GetDB()
	if err != nil {
		fmt.Print(err.Error())
		return
	}
	defer closer()

	logger.Debugf("Query %s", p.Queries)
	if query.IsSelectQuery(p.Queries) {
		logger.Debug("is select...")
		p.ExecuteSelect(db)
	} else {
		logger.Debug("is NOT select...")
		p.execute(db)
	}
}

func (p *ProtoSQL) execute(db *sql.DB) {
	if !query.ContainsSemicolonInMiddle(p.Queries) {
		res, err := db.Exec(p.Queries)
		if err != nil {
			logger.Errorf("Error executing query %v", err)
			fmt.Printf("[ERROR] %v", err)
			return
		}

		rowsAffected, err := res.RowsAffected()
		if err != nil {
			logger.Errorf("Error getting rows affected %v", err)
			fmt.Printf("[ERROR] %v", err)
			return
		}

		if query.IsInsertUpdateOrDelete(p.Queries) {
			fmt.Printf("  Row(s) affected: %d", rowsAffected)
		} else {
			fmt.Print("  Statement executed correctly.")
		}
	} else {
		queries := query.SplitQueries(p.Queries)
		results := make([]string, len(queries))
		for i, q := range queries {
			if res, err := db.Exec(q); err != nil {
				logger.Errorf("Error executing query %v", err)
				results[i] = fmt.Sprintf("%d)   %v\n", i+1, err)
			} else {
				if rowsAffected, err := res.RowsAffected(); err != nil {
					logger.Errorf("Error getting rows affected %v", err)
					results[i] = fmt.Sprintf("%d)   %v\n", i+1, err)
				} else {
					if query.IsInsertUpdateOrDelete(q) {
						results[i] = fmt.Sprintf("%d)   Row(s) affected: %d\n", i+1, rowsAffected)
					} else {
						results[i] = fmt.Sprintf("%d)   Statement executed correctly.\n", i+1)
					}
				}
			}
		}
		filePath := table.CreateDBeerFileFormat(p.DestFolder)
		fmt.Println("syn match dbeerStmtErr ' ' | hi link dbeerStmtErr ErrorMsg")
		fmt.Println(filePath)

		table.WriteToFile(filePath, results...)
	}
}

func (p *ProtoSQL) ExecuteSelect(db *sql.DB) {
	rows, err := db.Query(p.Queries)
	if err != nil {
		logger.Errorf("Error executing query %v", err)
		fmt.Printf("[ERROR] %v", err)
		return
	}
	defer rows.Close()

	columns, err := rows.Columns()
	if err != nil {
		logger.Errorf("Could not get columns %v", err)
		fmt.Printf("[ERROR] %v", err)
		return
	}
	lenColumns := len(columns)

	dbeer := table.DBeer{
		DestFolder:      p.DestFolder,
		BorderStyle:     p.BorderStyle,
		HeaderStyleLink: p.HeaderStyleLink,
		Headers:         make(map[int]table.Header, lenColumns),
		Rows:            make([][]string, 0),
	}

	for i, value := range columns {
		name := " 󰠵 " + strings.ToUpper(value)
		dbeer.Headers[i+1] = table.Header{
			Name:   name,
			Length: utf8.RuneCountInString(name) + 1,
		}
	}

	values := make([]any, lenColumns)
	for i := range values {
		var value any
		values[i] = &value
	}

	for rows.Next() {
		err := rows.Scan(values...)
		if err != nil {
			logger.Errorf("Error getting rows %v", err)
			fmt.Printf("[ERROR] %v", err)
			return
		}

		results := make([]string, lenColumns)
		for i, value := range values {
			var strValue string
			if bytesValue, ok := (*value.(*any)).([]byte); ok {
				strValue = string(bytesValue)
			} else {
				strValue = fmt.Sprintf("%v", *value.(*any))
			}

			field, fieldLength := formattedField(strValue)
			results[i] = field
			index := i + 1

			if dbeer.Headers[index].Length < fieldLength {
				dbeer.Headers[index] = table.Header{
					Name:   dbeer.Headers[index].Name,
					Length: fieldLength,
				}
			}
		}
		dbeer.Rows = append(dbeer.Rows, results)
	}

	if len(dbeer.Rows) > 0 {
		logger.Debug("Generating dbeer table...")
		dbeer.Generate()
	} else {
		fmt.Print("  Query has returned 0 results.")
	}
}

func (p *ProtoSQL) GetTables() {
	db, closer, err := p.GetDB()
	if err != nil {
		return
	}
	defer closer()

	logger.Debugf("Query to get tables: %s", p.Queries)

	rows, err := db.Query(p.Queries)
	if err != nil {
		logger.Errorf("Error executing query:", err)
		return
	}
	defer rows.Close()

	values := make([]string, 0)
	for rows.Next() {
		var table string
		if err := rows.Scan(&table); err != nil {
			logger.Errorf("Error scanning row:", err)
			return
		}
		values = append(values, strings.ToUpper(table))
	}

	if err := rows.Err(); err != nil {
		logger.Errorf("Error iterating over rows:", err)
		return
	}

	fmt.Print(values)
}

func (p *ProtoSQL) GetTableInfo() {
	db, closer, err := p.GetDB()
	if err != nil {
		fmt.Print(err.Error())
		return
	}
	defer closer()

	p.Queries = p.GetTableInfoQuery(p.Queries)
	logger.Debugf("Query to get table info: %s", p.Queries)

	p.ExecuteSelect(db)
}

func (ProtoSQL) GetTableInfoQuery(tableName string) string {
	return `SELECT 
                UPPER(c.column_name) AS column_name,
                c.data_type,
                CASE
                    WHEN c.is_nullable = 'YES' THEN ' '
                    ELSE ' '
                END AS not_null,
                CASE
                    WHEN c.character_maximum_length IS NULL THEN '-'
                    ELSE CAST(c.character_maximum_length AS CHAR)
                END AS length,
                CASE  
                    WHEN tc.constraint_type = 'PRIMARY KEY' THEN '  PRIMARY KEY'
                    WHEN tc.constraint_type = 'FOREIGN KEY' THEN '  FOREIGN KEY'
                    ELSE '-'
                END AS constraint_type,
                CASE 
                    WHEN tc.constraint_type = 'FOREIGN KEY' THEN 
                       '  ' || kcu2.table_name || '.' || kcu2.column_name
                    ELSE 
                        '-'
                END AS referenced_table_column
                FROM 
                    information_schema.columns AS c
                LEFT JOIN 
                    information_schema.key_column_usage AS kcu 
                    ON c.column_name = kcu.column_name 
                    AND c.table_name = kcu.table_name
                LEFT JOIN 
                    information_schema.table_constraints AS tc 
                    ON kcu.constraint_name = tc.constraint_name 
                    AND kcu.table_name = tc.table_name
                LEFT JOIN 
                    information_schema.referential_constraints AS rc 
                    ON tc.constraint_name = rc.constraint_name 
                    AND tc.table_schema = rc.constraint_schema
                LEFT JOIN 
                    information_schema.key_column_usage AS kcu2 
                    ON rc.unique_constraint_name = kcu2.constraint_name 
                    AND rc.unique_constraint_schema = kcu2.table_schema
                WHERE 
                    c.table_name = '` + tableName + `';`
}

func formattedField(str string) (string, int) {
	if str == "<nil>" {
		return stringAndLength("NULL")
	}

	if date, err := dateparse.ParseAny(str); err == nil {
		if date.Hour() == 0 && date.Minute() == 0 && date.Second() == 0 && date.Nanosecond() == 0 {
			return stringAndLength(date.Format("2006-01-02"))
		} else {
			return stringAndLength(date.Format("2006-01-02 15:04:05"))
		}
	}

	if len(str) > 100 {
		return stringAndLength(str[:100] + "...")
	}

	if i := strings.IndexAny(str, "\n\r"); i != -1 {
		return stringAndLength(str[:i] + "...")
	}

	return stringAndLength(str)
}

func stringAndLength(str string) (string, int) {
	return " " + str, utf8.RuneCountInString(str) + 2
}
