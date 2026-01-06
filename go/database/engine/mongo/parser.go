package mongo

import (
	"encoding/json"
	"errors"
	"fmt"
	"strings"

	"go.mongodb.org/mongo-driver/bson"
	"go.mongodb.org/mongo-driver/bson/primitive"
)

type mongoCommand struct {
	Collection string
	FuncParam  mongoFuncParam
	FuncParam2 *mongoFuncParam
}

type mongoFuncParam struct {
	Func   mongoFunc
	Params string
}

func newMongoFuncParam(s string) *mongoFuncParam {
	index := strings.Index(s, "(")
	if index != -1 {
		return &mongoFuncParam{
			Func:   mongoFunc(s[:index]),
			Params: s[index+1 : len(s)-1],
		}
	}
	return nil
}

type mongoFunc string

const (
	Find           mongoFunc = "find"
	Sort           mongoFunc = "sort"
	Skip           mongoFunc = "skip"
	Limit          mongoFunc = "limit"
	CountDocuments mongoFunc = "countDocuments"
	FindOne        mongoFunc = "findOne"
	InsertOne      mongoFunc = "insertOne"
	InsertMany     mongoFunc = "insertMany"
	DeleteOne      mongoFunc = "deleteOne"
	DeleteMany     mongoFunc = "deleteMany"
	UpdateOne      mongoFunc = "updateOne"
	UpdateMany     mongoFunc = "updateMany"
	Drop           mongoFunc = "drop"
)

func getQuerySections(query string) (*mongoCommand, error) {
	if len(query) == 0 {
		return nil, errors.New("Query empty.")
	}

	parts := strings.Split(query, ".")

	var collection string
	var function string
	var extra string
	if len(parts) > 0 {
		var index int
		if parts[index] == "db" {
			index = 1
		}
		collection = parts[index]
		function = parts[index+1]
		if len(parts) > index+2 {
			extra = parts[index+2]
		}

		funcParam := newMongoFuncParam(function)
		if funcParam != nil {
			mongoCommand := mongoCommand{
				Collection: collection,
				FuncParam:  *funcParam,
				FuncParam2: newMongoFuncParam(extra),
			}
			return &mongoCommand, nil
		} else {
			return nil, errors.New("Error format: " + function)
		}
	}
	return nil, errors.New("Error format: " + query)
}

func getBsonParsed(s string) (*primitive.M, error) {
	bsonObj := bson.M{}
	if len(s) > 2 {
		if err := json.Unmarshal([]byte(s), &bsonObj); err != nil {
			return nil, fmt.Errorf("parsing bson %v", err)
		}
	}
	return &bsonObj, nil
}

func getArrayParsed(s string) ([]any, error) {
	var array []any
	if len(s) > 2 {
		if err := json.Unmarshal([]byte(s), &array); err != nil {
			return nil, fmt.Errorf("parsing array %v", err)
		}
	}
	return array, nil
}

type parsedBson struct {
	First  primitive.M
	Second primitive.M
}

func getTwoBsonParsed(s string) (*parsedBson, error) {
	parts := strings.SplitN(s, "},", 2)
	length := len(parts)
	if length != 2 {
		return nil, fmt.Errorf("could not split the string. Length: %d", length)
	}

	filterStr := parts[0] + "}"
	updateStr := parts[1][1:]

	var first bson.M
	if err := json.Unmarshal([]byte(filterStr), &first); err != nil {
		return nil, fmt.Errorf("parsing first bson %v", err)
	}

	var second bson.M
	if err := json.Unmarshal([]byte(updateStr), &second); err != nil {
		return nil, fmt.Errorf("parsing second bson %v", err)
	}

	return &parsedBson{First: first, Second: second}, nil
}
