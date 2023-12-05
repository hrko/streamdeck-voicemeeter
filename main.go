package main

import (
	"context"
	"encoding/json"
	"fmt"
	"image"
	"image/color"
	"log"
	"os"
	"time"

	"github.com/FlowingSPDG/streamdeck"
	sdcontext "github.com/FlowingSPDG/streamdeck/context"
	"github.com/fufuok/cmap"
	"github.com/onyx-and-iris/voicemeeter/v2"
)

const (
	imgX   = 72
	imgY   = 72
	kindId = "potato"
)

type PropertyInspectorSettings struct {
	ShowText bool `json:"showText,omitempty"`
}

func main() {
	f, err := os.CreateTemp("", "voicemeeter-streamdeck-plugin.*.log")
	if err != nil {
		log.Fatal(err)
	}
	defer f.Close()
	log.SetOutput(f)

	vm, err := voicemeeter.NewRemote(kindId, 0)
	if err != nil {
		log.Fatal(err)
	}
	err = vm.Login()
	if err != nil {
		log.Fatal(err)
	}
	defer vm.Logout()
	vm.EventAdd("ldirty")

	ctx := context.Background()
	log.Println("Starting voicemeeter-streamdeck-plugin")
	if err := run(ctx, vm); err != nil {
		panic(err)
	}
}

func run(ctx context.Context, vm *voicemeeter.Remote) error {
	params, err := streamdeck.ParseRegistrationParams(os.Args)
	if err != nil {
		return err
	}
	log.Printf("Registration params: %v", params)

	client := streamdeck.NewClient(ctx, params)
	log.Println("Client created")
	setup(client, vm)
	log.Println("Setup done")

	log.Println("Running client")
	return client.Run(ctx)
}

func setup(client *streamdeck.Client, vm *voicemeeter.Remote) {
	action := client.Action("jp.hrko.voicemeeter.action")

	pi := &PropertyInspectorSettings{}
	sdContexts := cmap.NewOf[string, struct{}]()

	action.RegisterHandler(streamdeck.SendToPlugin, func(ctx context.Context, client *streamdeck.Client, event streamdeck.Event) error {
		b, _ := json.MarshalIndent(event, "", "	")
		log.Printf("event:%s\n", b)
		return json.Unmarshal(event.Payload, pi)
	})

	action.RegisterHandler(streamdeck.KeyDown, func(ctx context.Context, client *streamdeck.Client, event streamdeck.Event) error {
		b, _ := json.MarshalIndent(event, "", "	")
		log.Printf("event:%s\n", b)
		return json.Unmarshal(event.Payload, pi)
	})

	action.RegisterHandler(streamdeck.KeyUp, func(ctx context.Context, client *streamdeck.Client, event streamdeck.Event) error {
		b, _ := json.MarshalIndent(event, "", "	")
		log.Printf("event:%s\n", b)
		return json.Unmarshal(event.Payload, pi)
	})

	action.RegisterHandler(streamdeck.WillAppear, func(ctx context.Context, client *streamdeck.Client, event streamdeck.Event) error {
		b, _ := json.MarshalIndent(event, "", "	")
		log.Printf("event:%s\n", b)
		sdContexts.Set(event.Context, struct{}{})
		return nil
	})

	action.RegisterHandler(streamdeck.WillDisappear, func(ctx context.Context, client *streamdeck.Client, event streamdeck.Event) error {
		b, _ := json.MarshalIndent(event, "", "	")
		log.Printf("event:%s\n", b)
		sdContexts.Remove(event.Context)
		return nil
	})

	readings := make([]float64, imgX, imgX)

	go func() {
		for range time.Tick(time.Second / 30) {
			for i := 0; i < imgX-1; i++ {
				readings[i] = readings[i+1]
			}

			const busIndex = 5
			const levelMaxDb = 0.0
			const levelMinDb = -60.0
			log.Println("Getting levels")
			busCount := len(vm.Bus)
			log.Printf("busCount: %v\n", busCount)
			if busIndex >= busCount {
				log.Printf("busIndex %v is out of range\n", busIndex)
				continue
			}
			levels := vm.Bus[busIndex].Levels().All()
			log.Println("Got levels")
			levelDb := levels[0]
			log.Printf("levelDb: %v\n", levelDb)
			level := 0.0
			if levelDb > levelMaxDb {
				level = 1.0
			} else if levelDb > levelMinDb {
				level = (levelDb - levelMinDb) / (levelMaxDb - levelMinDb)
			} else {
				level = 0.0
			}
			readings[imgX-1] = level
			log.Printf("level [dB]: %v\n", levelDb)
			log.Printf("level: %v\n", level)

			log.Printf("sdContexts: %v\n", sdContexts.Keys())
			for item := range sdContexts.IterBuffered() {
				ctxStr := item.Key
				ctx := context.Background()
				ctx = sdcontext.WithContext(ctx, ctxStr)

				img, err := streamdeck.Image(graph(readings))
				if err != nil {
					log.Printf("error creating image: %v\n", err)
					continue
				}

				if err := client.SetImage(ctx, img, streamdeck.HardwareAndSoftware); err != nil {
					log.Printf("error setting image: %v\n", err)
					continue
				}

				title := ""
				if pi.ShowText {
					title = fmt.Sprintf("%.1f dB", levelDb)
				}

				if err := client.SetTitle(ctx, title, streamdeck.HardwareAndSoftware); err != nil {
					log.Printf("error setting title: %v\n", err)
					continue
				}
			}
		}
	}()
}

func graph(readings []float64) image.Image {
	img := image.NewRGBA(image.Rect(0, 0, imgX, imgY))
	for x := 0; x < imgX; x++ {
		reading := readings[x]
		upto := int(float64(imgY) * reading)
		for y := 0; y < upto; y++ {
			img.Set(x, imgY-y, color.RGBA{R: 255, A: 255})
		}
		for y := upto; y < imgY; y++ {
			img.Set(x, imgY-y, color.Black)
		}
	}
	return img
}
