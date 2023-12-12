//go:build windows

package main

import (
	"log"
	"os/exec"
)

func startStreamDeck() {
	cmd := exec.Command("C:\\Program Files\\Elgato\\StreamDeck\\StreamDeck.exe")
	err := cmd.Start()
	if err != nil {
		log.Println(err)
	}
}

func killStreamDeck() {
	cmd := exec.Command("taskkill", "/t", "/f", "/im", "StreamDeck.exe")
	err := cmd.Start()
	if err != nil {
		log.Println(err)
	}
}
