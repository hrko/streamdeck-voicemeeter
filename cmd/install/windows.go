//go:build windows

package main

import "os"

func getPluginsDir() string {
	return os.Getenv("APPDATA") + "\\Elgato\\StreamDeck\\Plugins"
}
