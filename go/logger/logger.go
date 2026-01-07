package logger

import (
	"fmt"
	"log"
	"os"
	"time"
)

var logger *log.Logger
var logDebug bool

const DATE_FORMAT = "2006/01/02 15:04:05"

func Initialize(logFileName string, logDebugEnabled bool) func() {
	logFile, err := os.OpenFile(logFileName, os.O_CREATE|os.O_WRONLY|os.O_APPEND, 0666)
	if err != nil {
		log.Fatalf("Error with %s, %v", logFileName, err)
	}

	logger = log.New(logFile, "", 0)
	logDebug = logDebugEnabled

	return func() {
		defer logFile.Close()
	}
}

func Debug(message string) {
	if logDebug {
		timestamp := time.Now().Format(DATE_FORMAT)
		logger.Printf("[DEBUG] [%s] [GO] %s\n", timestamp, message)
	}
}

func Debugf(format string, a ...any) {
	if logDebug {
		timestamp := time.Now().Format(DATE_FORMAT)
		finalFormat := fmt.Sprintf("[DEBUG] [%s] [GO] ", timestamp) + format
		logger.Printf(finalFormat, a...)
	}
}

func Error(message string) {
	timestamp := time.Now().Format(DATE_FORMAT)
	logger.Printf("[ERROR] [%s] [GO] %s\n", timestamp, message)
}

func Errorf(format string, a ...any) {
	timestamp := time.Now().Format(DATE_FORMAT)
	finalFormat := fmt.Sprintf("[ERROR] [%s] [GO] ", timestamp) + format
	logger.Printf(finalFormat, a...)
}
