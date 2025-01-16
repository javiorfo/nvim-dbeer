package engine

import (
	"fmt"

	_ "github.com/glebarez/go-sqlite"
	"github.com/javiorfo/nvim-dbeer/go/database/engine/model"
)

type Sqlite struct {
	model.ProtoSQL
}

func (s *Sqlite) GetTables() {
	s.Queries = "select name from sqlite_master where type = 'table' order by name;"
	s.ProtoSQL.GetTables()
}

func (s *Sqlite) GetTableInfo() {
	db, closer, err := s.GetDB()
	if err != nil {
		fmt.Print(err.Error())
		return
	}
	defer closer()

	s.Queries = s.GetTableInfoQuery(s.Queries)
	s.ExecuteSelect(db)
}

func (s *Sqlite) GetTableInfoQuery(tableName string) string {
    return "PRAGMA table_info(" + tableName + ")";
}
