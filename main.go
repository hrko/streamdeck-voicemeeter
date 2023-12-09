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
	"github.com/fogleman/gg"
	"github.com/fufuok/cmap"
	"github.com/onyx-and-iris/voicemeeter/v2"

	"github.com/hrko/streamdeck-voicemeeter/pkg/icon"
)

const (
	streamDeckKeyResolutionX = 72
	streamDeckKeyResolutionY = 72
)

var (
	chGlobalSettings   chan *GlobalSettings
	action1InstanceMap *cmap.MapOf[string, Action1InstanceProperty]
	action1RenderCh    chan *Action1RenderParams
)

type GlobalSettings struct {
	VoiceMeeterKind string `json:"voiceMeeterKind"`
}

type ActionInstanceCommonProperty struct {
	Controller  string `json:"controller,omitempty"` // "Keypad" | "Encoder"
	Coordinates struct {
		Column int `json:"column,omitempty"`
		Row    int `json:"row,omitempty"`
	} `json:"coordinates,omitempty"`
	IsInMultiAction bool `json:"isInMultiAction,omitempty"`
}

type Action1InstanceProperty struct {
	ActionInstanceCommonProperty
	Settings Action1InstanceSettings `json:"settings,omitempty"`
}

type Action1InstanceSettings struct {
	ShowText       bool                           `json:"showText,omitempty"`
	IconCodePoint  string                         `json:"iconCodePoint,omitempty"`
	IconFontParams icon.MaterialSymbolsFontParams `json:"iconFontParams,omitempty"`
}

type Action1RenderParams struct {
	TargetContext string
	Title         *string
	Settings      *Action1InstanceSettings
	Levels        *[]float64
	Gain          *float64
}

func main() {
	log.SetPrefix("package main: ")
	streamdeck.Log().SetOutput(os.Stderr)
	streamdeck.Log().SetPrefix("package streamdeck: ")

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

	registerNoActionHandlers(client)
	action1SetupPreClientRun(client)

	chErr := make(chan error)
	go func() {
		log.Println("Starting client")
		chErr <- client.Run(ctx)
	}()

	waitClientConnected(client)
	globalSettings, err := fetchGlobalSettings(ctx, client)
	if err != nil {
		log.Fatal(err)
	}
	log.Printf("Global settings: %v\n", globalSettings)

	vm, err := loginVoicemeeter(globalSettings.VoiceMeeterKind)
	if err != nil {
		log.Fatal(err)
	}
	defer vm.Logout()
	vm.EventAdd("ldirty")

	go action1SetupPostClientRun(client, vm)

	return <-chErr
}

func registerNoActionHandlers(client *streamdeck.Client) {
	chGlobalSettings = make(chan *GlobalSettings)
	client.RegisterNoActionHandler(streamdeck.DidReceiveGlobalSettings, func(ctx context.Context, client *streamdeck.Client, event streamdeck.Event) error {
		b, _ := json.MarshalIndent(event, "", "	")
		log.Printf("event:%s\n", b)
		payload := new(struct {
			Settings *GlobalSettings `json:"settings"`
		})
		err := json.Unmarshal(event.Payload, payload)
		if err != nil {
			log.Printf("error unmarshaling payload: %v\n", err)
			return err
		}
		select {
		case chGlobalSettings <- payload.Settings:
			log.Println("global settings received and sent to channel")
		default:
			log.Println("global settings received but no one is waiting for channel")
		}
		return nil
	})
}

func fetchGlobalSettings(ctx context.Context, client *streamdeck.Client) (*GlobalSettings, error) {
	if !client.IsConnected() {
		return nil, fmt.Errorf("client is not connected")
	}
	var gs *GlobalSettings
	eventReceived := make(chan struct{})
	defer close(eventReceived)
	go func() {
		gs = <-chGlobalSettings
		eventReceived <- struct{}{}
	}()
	ctx = sdcontext.WithContext(ctx, client.UUID())
	if err := client.GetGlobalSettings(ctx); err != nil {
		return nil, err
	}
	select {
	case <-eventReceived:
		return gs, nil
	case <-ctx.Done():
		return nil, ctx.Err()
	}
}

func loginVoicemeeter(kindId string) (*voicemeeter.Remote, error) {
	switch kindId {
	case "basic", "banana", "potato":
	default:
		log.Printf("unknown kindId: '%v', fallback to 'basic'\n", kindId)
		kindId = "basic"
	}
	vm, err := voicemeeter.NewRemote(kindId, 0)
	if err != nil {
		return nil, err
	}
	log.Println("Login to voicemeeter")
	err = vm.Login()
	if err != nil {
		return nil, err
	}
	return vm, nil
}

func waitClientConnected(client *streamdeck.Client) error {
	if !client.IsConnected() {
		log.Println("Waiting for client to connect")
		for !client.IsConnected() {
			time.Sleep(time.Second / 10)
		}
	}
	return nil
}

func action1SetupPreClientRun(client *streamdeck.Client) {
	action := client.Action("jp.hrko.voicemeeter.action")
	action1InstanceMap = cmap.NewOf[string, Action1InstanceProperty]() // key: context of action instance
	action1RenderCh = make(chan *Action1RenderParams, 32)

	action.RegisterHandler(streamdeck.DidReceiveSettings, func(ctx context.Context, client *streamdeck.Client, event streamdeck.Event) error {
		b, _ := json.MarshalIndent(event, "", "	")
		log.Printf("event:%s\n", b)
		var prop Action1InstanceProperty
		err := json.Unmarshal(event.Payload, &prop)
		if err != nil {
			log.Printf("error unmarshaling payload: %v\n", err)
			return err
		}
		action1InstanceMap.Set(event.Context, prop)
		action1RenderCh <- &Action1RenderParams{
			TargetContext: event.Context,
			Settings:      &prop.Settings,
		}
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
		var prop Action1InstanceProperty
		err := json.Unmarshal(event.Payload, &prop)
		if err != nil {
			log.Printf("error unmarshaling payload: %v\n", err)
			return err
		}
		action1InstanceMap.Set(event.Context, prop)
		action1RenderCh <- &Action1RenderParams{
			TargetContext: event.Context,
			Settings:      &prop.Settings,
		}
		return nil
	})

	action.RegisterHandler(streamdeck.WillDisappear, func(ctx context.Context, client *streamdeck.Client, event streamdeck.Event) error {
		b, _ := json.MarshalIndent(event, "", "	")
		log.Printf("event:%s\n", b)
		action1InstanceMap.Remove(event.Context)
		return nil
	})
}

func action1SetupPostClientRun(client *streamdeck.Client, vm *voicemeeter.Remote) error {
	go func() {
		for renderParam := range action1RenderCh {
			action1Render(client, renderParam)
		}
	}()

	go func() {
		const refreshInterval = time.Second / 15
		for range time.Tick(refreshInterval) {
			for item := range action1InstanceMap.IterBuffered() {
				actionContext := item.Key
				go func() {
					const busIndex = 5
					busCount := len(vm.Bus)
					if busIndex >= busCount {
						log.Printf("busIndex %v is out of range\n", busIndex)
						return
					}
					levels := vm.Bus[busIndex].Levels().All()
					levels = levels[:2]

					title := vm.Bus[busIndex].Label()
					gain := vm.Bus[busIndex].Gain()
					renderParam := &Action1RenderParams{
						TargetContext: actionContext,
						Title:         &title,
						Levels:        &levels,
						Gain:          &gain,
					}
					action1RenderCh <- renderParam
				}()
			}
		}
	}()

	return nil
}

func action1Render(client *streamdeck.Client, renderParam *Action1RenderParams) error {
	const levelMaxDb = 12.0
	const levelGoodDb = -24.0
	const levelMinDb = -60.0

	ctx := context.Background()
	ctx = sdcontext.WithContext(ctx, renderParam.TargetContext)

	instProps, ok := action1InstanceMap.Get(renderParam.TargetContext)
	if !ok {
		return fmt.Errorf("action1InstanceMap has no key '%v'", renderParam.TargetContext)
	}

	switch instProps.Controller {
	case "Keypad":
		if renderParam.Levels != nil {
			img := levelMeterHorizontal(*renderParam.Levels, levelMinDb, levelGoodDb, levelMaxDb, streamDeckKeyResolutionX, streamDeckKeyResolutionY, 2, 1, 1)
			imgBase64, err := streamdeck.Image(img)
			if err != nil {
				log.Printf("error creating image: %v\n", err)
				return err
			}
			if err := client.SetImage(ctx, imgBase64, streamdeck.HardwareAndSoftware); err != nil {
				log.Printf("error setting image: %v\n", err)
				return err
			}

			title := ""
			levelAvgDb := 0.0
			for _, lvDb := range *renderParam.Levels {
				levelAvgDb += lvDb
			}
			levelAvgDb /= float64(len(*renderParam.Levels))
			if instProps.Settings.ShowText {
				title = fmt.Sprintf("%.1f dB", levelAvgDb)
			}
			if err := client.SetTitle(ctx, title, streamdeck.HardwareAndSoftware); err != nil {
				log.Printf("error setting title: %v\n", err)
				return err
			}
		}

	case "Encoder":
		payload := struct {
			Title      *string `json:"title,omitempty"`
			Icon       *string `json:"icon,omitempty"`
			LevelMeter *string `json:"levelMeter,omitempty"`
			GainValue  *string `json:"gainValue,omitempty"`
		}{}

		if renderParam.Title != nil {
			payload.Title = renderParam.Title
		}
		if renderParam.Settings != nil {
			const defaultIconCodePoint = "e050" // volume_up
			fontParams := renderParam.Settings.IconFontParams
			fontParams.SetDefaultsForEmptyParam()
			if err := fontParams.Assert(); err != nil {
				log.Printf("invalid iconFontParams: %v\n", err)
				fontParams = icon.MaterialSymbolsFontParams{}
				fontParams.SetDefaultsForEmptyParam()
			}
			iconCodePoint := renderParam.Settings.IconCodePoint
			if iconCodePoint == "" {
				iconCodePoint = defaultIconCodePoint
			}
			img, err := icon.GetMaterialSymbolsIcon(fontParams, iconCodePoint)
			if err != nil {
				log.Printf("error creating image: %v\n", err)
				return err
			}
			imgBase64, err := streamdeck.Image(img)
			if err != nil {
				log.Printf("error converting image to base64: %v\n", err)
			}
			payload.Icon = &imgBase64
		}
		if renderParam.Levels != nil {
			img := levelMeterHorizontal(*renderParam.Levels, levelMinDb, levelGoodDb, levelMaxDb, 108, 8, 1, 1, 1)
			imgBase64, err := streamdeck.Image(img)
			if err != nil {
				log.Printf("error creating image: %v\n", err)
				return err
			}
			payload.LevelMeter = &imgBase64
		}
		if renderParam.Gain != nil {
			str := fmt.Sprintf("%.1f dB", *renderParam.Gain)
			payload.GainValue = &str
		}

		if err := client.SetFeedback(ctx, payload); err != nil {
			log.Printf("error setting feedback: %v\n", err)
			return err
		}

	default:
		log.Printf("unknown controller: %v\n", instProps.Controller)
		return fmt.Errorf("unknown controller: %v", instProps.Controller)
	}

	return nil
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
