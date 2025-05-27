package table

import (
	"bufio"
	"fmt"
	"os"
	"strings"
	"time"
	"unicode/utf8"

	"github.com/javiorfo/nvim-dbeer/go/database/table/border"
	"github.com/javiorfo/nvim-dbeer/go/logger"
)

const dbeer_extension = "dbeer"

type Header struct {
	Name   string
	Length int
}

type DBeer struct {
	DestFolder      string
	HeaderStyleLink string
	BorderStyle     int
	Headers         map[int]Header
	Rows            [][]string
}

func (t DBeer) Generate() {
	b := border.GetBorder(border.BorderOption(t.BorderStyle))

	headerUp := b.CornerUpLeft
	headerMid := b.Vertical
	headerBottom := b.VerticalLeft

	headers := t.Headers
	headersLength := len(headers)
	for key := 1; key < headersLength+1; key++ {
		length := headers[key].Length
		headerUp += strings.Repeat(b.Horizontal, length)
		headerBottom += strings.Repeat(b.Horizontal, length)
		headerMid += addSpaces(headers[key].Name, length)
		headerMid += b.Vertical

		if key < headersLength {
			headerUp += b.DivisionUp
			headerBottom += b.Intersection
		} else {
			headerUp += b.CornerUpRight
			headerBottom += b.VerticalRight
		}
	}

	rows := t.Rows
	table := make([]string, 3, (len(rows)*2)+3)
	table[0] = headerUp + "\n"
	table[1] = headerMid + "\n"
	table[2] = headerBottom + "\n"

	rowsLength := len(rows) - 1
	rowFieldsLength := len(rows[0]) - 1
	for i, row := range rows {
		value := b.Vertical
		var line string

		if i < rowsLength {
			line += b.VerticalLeft
		} else {
			line += b.CornerBottomLeft
		}

		for j, field := range row {
			value += addSpaces(field, headers[j+1].Length)
			value += b.Vertical

			line += strings.Repeat(b.Horizontal, headers[j+1].Length)
			if i < rowsLength {
				if j < rowFieldsLength {
					line += b.Intersection
				} else {
					line += b.VerticalRight
				}
			} else if j < rowFieldsLength {
				line += b.DivisionBottom
			} else {
				line += b.CornerBottomRight
			}
		}
		table = append(table, value+"\n", line+"\n")
	}

	filePath := CreateDBeerFileFormat(t.DestFolder)
	logger.Debugf("File path: %s", filePath)
	fmt.Println(highlighting(t.Headers, t.HeaderStyleLink))
	fmt.Println(filePath)

	WriteToFile(filePath, table...)
}

func highlighting(headers map[int]Header, style string) string {
	result := ""
	for k, v := range headers {
		result += fmt.Sprintf("syn match header%d '\\<%s\\>' | hi link header%d %s |", k, strings.TrimSpace(v.Name), k, style)
	}
	logger.Debugf("Highlight matches: %s", result)
	return result
}

func addSpaces(inputString string, length int) string {
	result := inputString
	lengthInputString := utf8.RuneCountInString(inputString)

	if length > lengthInputString {
		diff := length - lengthInputString
		result += strings.Repeat(" ", diff)
	}

	return result
}

func WriteToFile(filePath string, values ...string) {
	file, err := os.Create(filePath)
	if err != nil {
		logger.Errorf("Error creating file: %v", err)
		fmt.Printf("[ERROR] %v", err)
		return
	}
	defer file.Close()

	writer := bufio.NewWriter(file)

	for _, v := range values {
		_, err := writer.WriteString(v)
		if err != nil {
			logger.Errorf("Error writing to file: %v", err)
			fmt.Printf("[ERROR] %v", err)
			return
		}
	}

	if err := writer.Flush(); err != nil {
		logger.Errorf("Error flushing writer: %v", err)
		fmt.Printf("[ERROR] %v", err)
		return
	}
}

func CreateDBeerFileFormat(destFolder string) string {
	return fmt.Sprintf("%s/%s.%s", destFolder, time.Now().Format("20060102-150405"), dbeer_extension)
}

func CreateDBeerMongoFileFormat(destFolder string) string {
	return fmt.Sprintf("%s/%s.%s.%s", destFolder, time.Now().Format("20060102-150405"), dbeer_extension, "json")
}
