package main

import (
	"context"
	"encoding/json"
	"fmt"
	"image"
	"image/color"
	"log"
	"os"
	"strconv"

	"github.com/FlowingSPDG/streamdeck"
)

type Settings struct {
	Counter int `json:"counter"`
}

func main() {
	f, err := os.CreateTemp("", "voicemeeter-streamdeck-plugin.*.log")
	if err != nil {
		log.Fatal(err)
	}
	defer f.Close()
	log.SetOutput(f)

	ctx := context.Background()
	log.Println("Starting voicemeeter-streamdeck-plugin")
	if err := run(ctx); err != nil {
		panic(err)
	}
}

func run(ctx context.Context) error {
	params, err := streamdeck.ParseRegistrationParams(os.Args)
	if err != nil {
		return err
	}
	log.Printf("Registration params: %v", params)

	client := streamdeck.NewClient(ctx, params)
	log.Println("Client created")
	setup(client)
	log.Println("Setup done")

	log.Println("Running client")
	return client.Run(ctx)
}

func setup(client *streamdeck.Client) {
	action := client.Action("jp.hrko.voicemeeter.action")
	// This is not goroutine safe
	// Use sync.Map instead for goroutine safe map
	settings := make(map[string]*Settings)

	action.RegisterHandler(streamdeck.WillAppear, func(ctx context.Context, client *streamdeck.Client, event streamdeck.Event) error {
		log.Println(event.Event, event.Action, event.Context, event.Device, string(event.Payload))

		p := streamdeck.WillAppearPayload[Settings]{}
		if err := json.Unmarshal(event.Payload, &p); err != nil {
			return err
		}

		s, ok := settings[event.Context]
		if !ok {
			s = &Settings{Counter: 0}
			settings[event.Context] = s
		}

		bg, err := streamdeck.Image(background())
		if err != nil {
			return err
		}

		if err := client.SetImage(ctx, bg, streamdeck.HardwareAndSoftware); err != nil {
			return err
		}

		return client.SetTitle(ctx, strconv.Itoa(s.Counter), streamdeck.HardwareAndSoftware)
	})

	action.RegisterHandler(streamdeck.WillDisappear, func(ctx context.Context, client *streamdeck.Client, event streamdeck.Event) error {
		log.Println(event.Event, event.Action, event.Context, event.Device, string(event.Payload))

		s, _ := settings[event.Context]
		s.Counter = 0
		return client.SetSettings(ctx, s)
	})

	action.RegisterHandler(streamdeck.KeyDown, func(ctx context.Context, client *streamdeck.Client, event streamdeck.Event) error {
		log.Println(event.Event, event.Action, event.Context, event.Device, string(event.Payload))

		s, ok := settings[event.Context]
		if !ok {
			return fmt.Errorf("couldn't find settings for context %v", event.Context)
		}

		s.Counter++
		log.Println("Counter:", s.Counter)
		if err := client.SetSettings(ctx, s); err != nil {
			log.Println("Cannot set settings")
			return err
		}

		return client.SetTitle(ctx, strconv.Itoa(s.Counter), streamdeck.HardwareAndSoftware)
	})
}

func background() image.Image {
	img := image.NewRGBA(image.Rect(0, 0, 72, 72))
	for x := 0; x < 72; x++ {
		for y := 0; y < 72; y++ {
			img.Set(x, y, color.Black)
		}
	}
	return img
}
