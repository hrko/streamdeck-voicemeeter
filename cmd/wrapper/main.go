package main

import (
	"log"
	"os"
	"os/exec"
	"path/filepath"
)

const (
	binName = "streamdeck-voicemeeter.exe"
	logName = "voicemeeter-streamdeck-plugin.*.log"
)

func main() {
	exe, err := os.Executable()
	if err != nil {
		log.Fatalf("Failed to get executable path: %v", err)
	}
	exeDir := filepath.Dir(exe)
	commandName := filepath.Join(exeDir, binName)
	args := os.Args[1:]

	cmd := exec.Command(commandName, args...)

	tempFile, err := os.CreateTemp("", logName)
	if err != nil {
		log.Fatalf("Failed to create temp file: %v", err)
	}
	defer tempFile.Close()

	cmd.Stdout = tempFile
	cmd.Stderr = tempFile

	cmd.Run()

	log.Printf("Log written to: %s", tempFile.Name())
}
