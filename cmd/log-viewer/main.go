package main

import (
	"bufio"
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"time"
)

const (
	logName = "voicemeeter-streamdeck-plugin.*.log"
)

func main() {
	if len(os.Args) > 1 && os.Args[1] == "clear" {
		files := getLogFiles(false)
		for _, file := range files {
			err := os.Remove(file)
			if err != nil {
				fmt.Println("Error removing file:", err)
			}
		}
		return
	}

	files := getLogFiles(true)

	sort.Slice(files, func(i, j int) bool {
		fileInfoI, errI := os.Stat(files[i])
		fileInfoJ, errJ := os.Stat(files[j])
		if errI != nil || errJ != nil {
			return false
		}
		return fileInfoI.ModTime().After(fileInfoJ.ModTime())
	})

	if len(files) > 0 {
		latestLogFile := files[0]
		file, err := os.Open(latestLogFile)
		if err != nil {
			fmt.Println("Error opening file:", err)
			return
		}
		defer file.Close()

		reader := bufio.NewReader(file)
		for {
			line, err := reader.ReadString('\n')
			if err != nil {
				time.Sleep(time.Second / 10)
				continue
			}
			fmt.Print(line)
		}
	} else {
		fmt.Println("No log files found.")
	}
}

func getLogFiles(wait bool) []string {
	logPattern := filepath.Join(os.TempDir(), logName)
	if !wait {
		files, err := filepath.Glob(logPattern)
		if err != nil {
			fmt.Println("Error finding log files:", err)
			return nil
		}
		return files
	}
	fmt.Print("Waiting for log files")
	for {
		files, err := filepath.Glob(logPattern)
		if err != nil {
			fmt.Println("Error finding log files:", err)
			return nil
		}
		if len(files) > 0 {
			fmt.Println()
			return files
		}
		fmt.Print(".")
		time.Sleep(time.Second / 10)
	}
}
