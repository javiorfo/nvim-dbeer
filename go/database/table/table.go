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

	var headerUp strings.Builder
	headerUp.WriteString(b.CornerUpLeft)

	var headerMid strings.Builder
	headerMid.WriteString(b.Vertical)

	var headerBottom strings.Builder
	headerBottom.WriteString(b.VerticalLeft)

	headers := t.Headers
	headersLength := len(headers)
	for key := 1; key < headersLength+1; key++ {
		length := headers[key].Length
		headerUp.WriteString(strings.Repeat(b.Horizontal, length))
		headerBottom.WriteString(strings.Repeat(b.Horizontal, length))
		headerMid.WriteString(addSpaces(headers[key].Name, length))
		headerMid.WriteString(b.Vertical)

		if key < headersLength {
			headerUp.WriteString(b.DivisionUp)
			headerBottom.WriteString(b.Intersection)
		} else {
			headerUp.WriteString(b.CornerUpRight)
			headerBottom.WriteString(b.VerticalRight)
		}
	}

	rows := t.Rows
	table := make([]string, 3, (len(rows)*2)+3)

	headerUp.WriteByte('\n')
	table[0] = headerUp.String()

	headerMid.WriteByte('\n')
	table[1] = headerMid.String()

	headerBottom.WriteByte('\n')
	table[2] = headerBottom.String()

	rowsLength := len(rows) - 1
	rowFieldsLength := len(rows[0]) - 1
	for i, row := range rows {
		var value strings.Builder
		value.WriteString(b.Vertical)

		var line strings.Builder

		if i < rowsLength {
			line.WriteString(b.VerticalLeft)
		} else {
			line.WriteString(b.CornerBottomLeft)
		}

		for j, field := range row {
			value.WriteString(addSpaces(field, headers[j+1].Length))
			value.WriteString(b.Vertical)

			line.WriteString(strings.Repeat(b.Horizontal, headers[j+1].Length))
			if i < rowsLength {
				if j < rowFieldsLength {
					line.WriteString(b.Intersection)
				} else {
					line.WriteString(b.VerticalRight)
				}
			} else if j < rowFieldsLength {
				line.WriteString(b.DivisionBottom)
			} else {
				line.WriteString(b.CornerBottomRight)
			}
		}
		line.WriteByte('\n')
		value.WriteByte('\n')
		table = append(table, value.String(), line.String())
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
