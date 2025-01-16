package mongo

import (
	"context"
	"fmt"
	"reflect"
	"strings"
	"time"
	"unicode/utf8"

	"github.com/javiorfo/nvim-dbeer/go/database/engine/model"
	"github.com/javiorfo/nvim-dbeer/go/database/table"
	"github.com/javiorfo/nvim-dbeer/go/logger"
	"go.mongodb.org/mongo-driver/bson"
	"go.mongodb.org/mongo-driver/mongo"
	"go.mongodb.org/mongo-driver/mongo/options"
)

type Mongo struct {
	model.ProtoSQL
}

func (m *Mongo) getDB(c context.Context) (*mongo.Database, func(), error) {
	clientOptions := options.Client().ApplyURI(m.ConnStr)

	client, err := mongo.Connect(c, clientOptions)
	db := client.Database(m.DbName)
	if err != nil {
		logger.Errorf("Error initializing %s, connStr: %s", m.Engine, m.ConnStr)
		return nil, nil, fmt.Errorf("[ERROR] %v", err)
	}
	closer := func() {
		if err = client.Disconnect(c); err != nil {
			logger.Errorf("Error disconnecting from MongoDB: %v", err)
			return
		}
	}
	return db, closer, nil
}

func (m *Mongo) Run() {
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()
	db, closer, err := m.getDB(ctx)
	if err != nil {
		fmt.Print(err.Error())
		return
	}
	defer closer()

	mongoCommand, err := getQuerySections(m.Queries)
	if err != nil {
		fmt.Printf("[ERROR] %v", err)
		return
	}

	switch mongoCommand.FuncParam.Func {
	case Find:
        find(ctx, mongoCommand, db, m.DestFolder)
	case FindOne:
        findOne(ctx, mongoCommand, db, m.DestFolder)
	case CountDocuments:
        countDocuments(ctx, mongoCommand, db)
	case InsertOne:
        insertOne(ctx, mongoCommand, db)
	case InsertMany:
        insertMany(ctx, mongoCommand, db)
	case DeleteOne:
        deleteOne(ctx, mongoCommand, db)
	case DeleteMany:
        deleteMany(ctx, mongoCommand, db)
	case UpdateOne:
        updateOne(ctx, mongoCommand, db)
	case UpdateMany:
        updateMany(ctx, mongoCommand, db)
	case Drop:
        dropCollection(ctx, mongoCommand, db)
	default:
		fmt.Printf("[ERROR] %s is not an available function", mongoCommand.FuncParam.Func)
		return
	}

}

func (m *Mongo) GetTables() {
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()
	db, closer, err := m.getDB(ctx)
	if err != nil {
		return
	}
	defer closer()

	collections, err := db.ListCollections(ctx, bson.D{})
	if err != nil {
		logger.Errorf("Error listing collection:", err)
		return
	}

	values := make([]string, 0)
	for collections.Next(ctx) {
		var collection bson.M
		err := collections.Decode(&collection)
		if err != nil {
			logger.Errorf("Error decoding collection:", err)
			return
		}
		values = append(values, collection["name"].(string))
	}

	if err := collections.Err(); err != nil {
		logger.Errorf("Error iterating over rows:", err)
		return
	}

	fmt.Print(values)
}

func (m *Mongo) GetTableInfo() {
	ctx, cancel := context.WithTimeout(context.Background(), 3*time.Second)
	defer cancel()

	db, closer, err := m.getDB(ctx)
	if err != nil {
		fmt.Print(err.Error())
		return
	}
	defer closer()

	collection := db.Collection(m.Queries)

	cursor, err := collection.Find(ctx, bson.D{})
	if err != nil {
		logger.Errorf("Error listing collection:", err)
		fmt.Printf("[ERROR] %v", err)
		return
	}
	defer cursor.Close(ctx)

	var maxKeysDoc bson.M
	maxKeysCount := 0

	for cursor.Next(ctx) {
		var result bson.M
		if err := cursor.Decode(&result); err != nil {
			logger.Errorf("Error decoding:", err)
			fmt.Printf("[ERROR] %v", err)
			return
		}

		keysCount := len(result)
		if keysCount > maxKeysCount {
			maxKeysCount = keysCount
			maxKeysDoc = result
		}
	}

	if err := cursor.Err(); err != nil {
		logger.Errorf("Error in collection cursor:", err)
		fmt.Printf("[ERROR] %v", err)
		return
	}

	dbeer := table.DBeer{
		DestFolder:      m.DestFolder,
		BorderStyle:     m.BorderStyle,
		HeaderStyleLink: m.HeaderStyleLink,
		Headers: map[int]table.Header{
			1: {Name: " 󰠵 KEY", Length: 7},
			2: {Name: " 󰠵 DATA_TYPE", Length: 13},
		},
		Rows: make([][]string, len(maxKeysDoc)),
	}

	index := 0
	for key, value := range maxKeysDoc {
		valueKey := " " + strings.ToUpper(key)
		valueType := " " + reflect.TypeOf(value).String()
		dbeer.Rows[index] = []string{valueKey, valueType}

		valueKeyLength := utf8.RuneCountInString(valueKey) + 2
		if dbeer.Headers[1].Length < valueKeyLength {
			dbeer.Headers[1] = table.Header{
				Name:   dbeer.Headers[1].Name,
				Length: valueKeyLength,
			}
		}
		valueTypeLength := utf8.RuneCountInString(valueType) + 2
		if dbeer.Headers[2].Length < valueTypeLength {
			dbeer.Headers[2] = table.Header{
				Name:   dbeer.Headers[2].Name,
				Length: valueTypeLength,
			}
		}
		index++
	}

	dbeer.Generate()
}
