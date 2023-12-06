package main

import (
	"context"
	"encoding/json"
	"fmt"
	"image"
	"image/color"
	"log"
	"os"
	"path/filepath"
	"strconv"
	"time"

	"github.com/FlowingSPDG/streamdeck"
	sdcontext "github.com/FlowingSPDG/streamdeck/context"
	"github.com/fogleman/gg"
	"github.com/fufuok/cmap"
	"github.com/onyx-and-iris/voicemeeter/v2"
	"github.com/tdewolff/canvas"
	"github.com/tdewolff/canvas/renderers/rasterizer"
)

const (
	imgX   = 72
	imgY   = 72
	kindId = "potato"
)

type ActionInstanceSettings struct {
	ShowText bool `json:"showText,omitempty"`
}

type ActionInstanceCoordinates struct {
	Column int `json:"column,omitempty"`
	Row    int `json:"row,omitempty"`
}

type ActionInstanceProperty struct {
	Controller      string                    `json:"controller,omitempty"` // "Keypad" | "Encoder"
	Coordinates     ActionInstanceCoordinates `json:"coordinates,omitempty"`
	IsInMultiAction bool                      `json:"isInMultiAction,omitempty"`
	Settings        ActionInstanceSettings    `json:"settings,omitempty"`
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

	actionInstanceMap := cmap.NewOf[string, ActionInstanceProperty]() // key: context of action instance

	action.RegisterHandler(streamdeck.DidReceiveSettings, func(ctx context.Context, client *streamdeck.Client, event streamdeck.Event) error {
		b, _ := json.MarshalIndent(event, "", "	")
		log.Printf("event:%s\n", b)
		var prop ActionInstanceProperty
		err := json.Unmarshal(event.Payload, &prop)
		if err != nil {
			log.Printf("error unmarshaling payload: %v\n", err)
			return err
		}
		actionInstanceMap.Set(event.Context, prop)
		return nil
	})

	action.RegisterHandler(streamdeck.SendToPlugin, func(ctx context.Context, client *streamdeck.Client, event streamdeck.Event) error {
		b, _ := json.MarshalIndent(event, "", "	")
		log.Printf("event:%s\n", b)
		return nil
	})

	action.RegisterHandler(streamdeck.KeyDown, func(ctx context.Context, client *streamdeck.Client, event streamdeck.Event) error {
		b, _ := json.MarshalIndent(event, "", "	")
		log.Printf("event:%s\n", b)
		return nil
	})

	action.RegisterHandler(streamdeck.KeyUp, func(ctx context.Context, client *streamdeck.Client, event streamdeck.Event) error {
		b, _ := json.MarshalIndent(event, "", "	")
		log.Printf("event:%s\n", b)
		return nil
	})

	action.RegisterHandler(streamdeck.WillAppear, func(ctx context.Context, client *streamdeck.Client, event streamdeck.Event) error {
		b, _ := json.MarshalIndent(event, "", "	")
		log.Printf("event:%s\n", b)
		var prop ActionInstanceProperty
		err := json.Unmarshal(event.Payload, &prop)
		if err != nil {
			log.Printf("error unmarshaling payload: %v\n", err)
			return err
		}
		actionInstanceMap.Set(event.Context, prop)
		return nil
	})

	action.RegisterHandler(streamdeck.WillDisappear, func(ctx context.Context, client *streamdeck.Client, event streamdeck.Event) error {
		b, _ := json.MarshalIndent(event, "", "	")
		log.Printf("event:%s\n", b)
		actionInstanceMap.Remove(event.Context)
		return nil
	})

	go func() {
		const refreshInterval = time.Second / 30
		for range time.Tick(refreshInterval) {
			for item := range actionInstanceMap.IterBuffered() {
				const busIndex = 5
				const levelMaxDb = 12.0
				const levelGoodDb = -24.0
				const levelMinDb = -60.0
				busCount := len(vm.Bus)
				if busIndex >= busCount {
					log.Printf("busIndex %v is out of range\n", busIndex)
					continue
				}
				levels := vm.Bus[busIndex].Levels().All()
				levels = levels[:2]

				ctxStr := item.Key
				ctx := context.Background()
				ctx = sdcontext.WithContext(ctx, ctxStr)

				switch item.Val.Controller {
				case "Keypad":
					img := levelMeterHorizontal(levels, levelMinDb, levelGoodDb, levelMaxDb, imgX, imgY, 2, 1, 1)
					imgBase64, err := streamdeck.Image(img)
					if err != nil {
						log.Printf("error creating image: %v\n", err)
						continue
					}
					err = client.SetImage(ctx, imgBase64, streamdeck.HardwareAndSoftware)
					if err != nil {
						log.Printf("error setting image: %v\n", err)
						continue
					}
					title := ""
					levelAvgDb := 0.0
					for _, lvDb := range levels {
						levelAvgDb += lvDb
					}
					levelAvgDb /= float64(len(levels))
					if item.Val.Settings.ShowText {
						title = fmt.Sprintf("%.1f dB", levelAvgDb)
					}

					if err := client.SetTitle(ctx, title, streamdeck.HardwareAndSoftware); err != nil {
						log.Printf("error setting title: %v\n", err)
						continue
					}

				case "Encoder":
					imgIcon, err := getMaterialIcon("e050")
					if err != nil {
						log.Printf("error creating image: %v\n", err)
						continue
					}
					imgIconBase64, err := streamdeck.Image(imgIcon)
					if err != nil {
						log.Printf("error creating image: %v\n", err)
						continue
					}

					imgLevelMeter := levelMeterHorizontal(levels, levelMinDb, levelGoodDb, levelMaxDb, 108, 8, 1, 1, 1)
					imgLevelMeterBase64, err := streamdeck.Image(imgLevelMeter)
					if err != nil {
						log.Printf("error creating image: %v\n", err)
						continue
					}
					payload := struct {
						Title      string `json:"title"`
						Icon       string `json:"icon"`
						LevelMeter string `json:"levelMeter"`
						GainValue  string `json:"gainValue"`
					}{
						Title:      vm.Bus[busIndex].Label(),
						Icon:       imgIconBase64,
						LevelMeter: imgLevelMeterBase64,
						GainValue:  fmt.Sprintf("%.1f dB", vm.Bus[busIndex].Gain()),
					}
					b, _ := json.MarshalIndent(payload, "", "	")
					log.Printf("payload: %s\n", b)
					err = client.SetFeedback(ctx, payload)
					if err != nil {
						log.Printf("error setting feedback: %v\n", err)
						continue
					}

				default:
					log.Printf("unknown controller: %v\n", item.Val.Controller)
					continue
				}
			}
		}
	}()
}

func mmToPoints(mm float64) float64 {
	return mm * 2.834645669291339
}

func getMaterialIcon(codePoint string) (image.Image, error) {
	exe, err := os.Executable()
	if err != nil {
		return nil, err
	}
	exeDir := filepath.Dir(exe)
	fontMaterial := canvas.NewFontFamily("Material Symbols Outlined")
	err = fontMaterial.LoadFontFile(filepath.Join(exeDir, "MaterialSymbolsOutlined.woff2"), canvas.FontRegular)
	if err != nil {
		return nil, err
	}
	face := fontMaterial.Face(mmToPoints(48), color.White, canvas.FontRegular, canvas.FontNormal)

	c := canvas.New(48, 48)
	ctx := canvas.NewContext(c)
	codeInt, err := strconv.ParseInt(codePoint, 16, 32)
	if err != nil {
		return nil, err
	}
	codeRune := rune(codeInt)
	ctx.DrawText(0, 0, canvas.NewTextLine(face, string(codeRune), canvas.Left))

	return rasterizer.Draw(c, canvas.DPMM(1.0), canvas.DefaultColorSpace), nil
}

func levelMeterHorizontal(dB []float64, dBMin float64, dBGood float64, dBMax float64, width int, height int, cellWidth int, cellMarginX int, cellMarginY int) image.Image {
	channelCount := len(dB)
	cellHeight := (height - cellMarginY*(channelCount-1)) / channelCount
	cellCount := (width + cellMarginX) / (cellWidth + cellMarginX)
	minGoodCellIndex := int((dBGood - dBMin) / (dBMax - dBMin) * float64(cellCount))
	minClipCellIndex := int((0.0 - dBMin) / (dBMax - dBMin) * float64(cellCount))

	backgroundColor := color.RGBA{R: 0x00, G: 0x00, B: 0x00, A: 0xff}
	cellOnNormalColor := color.RGBA{R: 133, G: 173, B: 185, A: 0xff}
	cellOnGoodColor := color.RGBA{R: 30, G: 254, B: 91, A: 0xff}
	cellOnClipColor := color.RGBA{R: 250, G: 0, B: 0, A: 0xff}
	cellOffNormalColor := color.RGBA{R: 25, G: 27, B: 27, A: 0xff}
	cellOffGoodColor := color.RGBA{R: 25, G: 27, B: 27, A: 0xff}
	cellOffClipColor := color.RGBA{R: 31, G: 23, B: 21, A: 0xff}

	img := image.NewRGBA(image.Rect(0, 0, width, height))
	dc := gg.NewContextForRGBA(img)

	dc.SetColor(backgroundColor)
	dc.DrawRectangle(0, 0, float64(width), float64(height))
	dc.Fill()

	for ch, lvDb := range dB {
		lv := 0.0
		if lvDb > dBMax {
			lv = 1.0
		} else if lvDb > dBMin {
			lv = (lvDb - dBMin) / (dBMax - dBMin)
		} else {
			lv = 0.0
		}
		minOffCellIndex := int(lv * float64(cellCount))
		for i := 0; i < cellCount; i++ {
			x := i * (cellWidth + cellMarginX)
			y := ch * (cellHeight + cellMarginY)
			w := cellWidth
			h := cellHeight
			if i < minOffCellIndex {
				if i < minGoodCellIndex {
					dc.SetColor(cellOnNormalColor)
				} else if i < minClipCellIndex {
					dc.SetColor(cellOnGoodColor)
				} else {
					dc.SetColor(cellOnClipColor)
				}
			} else {
				if i < minGoodCellIndex {
					dc.SetColor(cellOffNormalColor)
				} else if i < minClipCellIndex {
					dc.SetColor(cellOffGoodColor)
				} else {
					dc.SetColor(cellOffClipColor)
				}
			}
			dc.DrawRectangle(float64(x), float64(y), float64(w), float64(h))
			dc.Fill()
		}
	}

	return img
}
