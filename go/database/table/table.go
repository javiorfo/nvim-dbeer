package table

import (
	"bufio"
	"fmt"
	"os"
	"strings"

	"github.com/javiorfo/nvim-tabula/go/database/table/border"
	"github.com/javiorfo/nvim-tabula/go/logger"
)

type Header struct {
	Name   string
	Length int
}

type Tabula struct {
	DestFolder      string
	HeaderStyleLink string
	BorderStyle     int
	Headers         map[int]Header
	Rows            [][]string
}

func (t Tabula) Generate() {
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
	fmt.Print(highlighting(t.Headers, t.HeaderStyleLink))

	WriteToFile(t.DestFolder, "tabula", table...)
}

func highlighting(headers map[int]Header, style string) string {
	result := ""
	for k, v := range headers {
		result += fmt.Sprintf("syn match header%d '%s' | hi link header%d %s |", k, v.Name, k, style)
	}
	return result
}

func addSpaces(inputString string, length int) string {
	result := inputString

	if length > len(inputString) {
		diff := length - len(inputString)
		result += strings.Repeat(" ", diff)
	}

	return result
}

func WriteToFile(destFolder, filename string, values ...string) {
	file, err := os.Create(fmt.Sprintf("%s/%s", destFolder, filename))
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
