package macro

import (
	"context"
	"encoding/json"
	"image/color"
	"log"
	"strconv"

	"github.com/fufuok/cmap"
	"github.com/go-playground/colors"
	"github.com/hrko/streamdeck"
	sdcontext "github.com/hrko/streamdeck/context"
	"github.com/onyx-and-iris/voicemeeter/v2"

	"github.com/hrko/streamdeck-voicemeeter/pkg/graphics"
)

const (
	ActionUUID       = "jp.hrko.streamdeck.voicemeeter.macro"
	ButtonTypeToggle = "toggle"
	ButtonTypePush   = "push"
)

var (
	shownInstances *cmap.MapOf[string, instanceProperty]
	renderCh       chan *renderParams
)

type instanceProperty streamdeck.WillAppearPayload[instanceSettings]

type instanceSettings struct {
	LogicalId        string                             `json:"logicalId,omitempty"`
	ButtonType       string                             `json:"buttonType,omitempty"`
	IconFontParams   graphics.MaterialSymbolsFontParams `json:"iconFontParams,omitempty"`
	IconCodePointOn  string                             `json:"iconCodePointOn,omitempty"`
	IconCodePointOff string                             `json:"iconCodePointOff,omitempty"`
	BgColorOn        string                             `json:"bgColorOn,omitempty"`
	BgColorOff       string                             `json:"bgColorOff,omitempty"`
}

type renderParams struct {
	targetContext string
	state         bool
}

func (s *instanceSettings) getSafeLogicalId(vm *voicemeeter.Remote) (int, error) {
	logicalId, err := strconv.Atoi(s.LogicalId)
	if err != nil {
		return 0, err
	}

	if logicalId < 0 || logicalId > len(vm.Button) {
		return 0, nil
	}

	return logicalId, nil
}

func (s *instanceSettings) setImages(client *streamdeck.Client, actionContext string) error {
	ctx := context.Background()
	ctx = sdcontext.WithContext(ctx, actionContext)

	iconColor := color.White
	borderColor := color.Transparent
	bgColorOn, _ := colors.ParseHEX(s.BgColorOn)
	bgColorOff, _ := colors.ParseHEX(s.BgColorOff)

	iconSize := 36
	imgSize := 72
	offsetX := (imgSize - iconSize) / 2
	offsetY := offsetX
	borderWidth := 0

	svgOn, err := s.IconFontParams.RenderIconSVG(s.IconCodePointOn, iconSize, imgSize, offsetX, offsetY, iconColor, borderColor, bgColorOn, borderWidth)
	if err != nil {
		log.Printf("error rendering icon: %v\n", err)
		return err
	}
	svgOff, err := s.IconFontParams.RenderIconSVG(s.IconCodePointOff, iconSize, imgSize, offsetX, offsetY, iconColor, borderColor, bgColorOff, borderWidth)
	if err != nil {
		log.Printf("error rendering icon: %v\n", err)
		return err
	}

	imgStringOn := streamdeck.ImageSvg(svgOn)
	imgStringOff := streamdeck.ImageSvg(svgOff)

	err = client.SetImage(ctx, imgStringOn, streamdeck.HardwareAndSoftware, ptr(0))
	if err != nil {
		log.Printf("error setting image: %v\n", err)
		return err
	}
	err = client.SetImage(ctx, imgStringOff, streamdeck.HardwareAndSoftware, ptr(1))
	if err != nil {
		log.Printf("error setting image: %v\n", err)
		return err
	}

	return nil
}

func defaultInstanceSettings() instanceSettings {
	return instanceSettings{
		LogicalId:  "0",
		ButtonType: ButtonTypePush,
		IconFontParams: graphics.MaterialSymbolsFontParams{
			Style: "Outlined",
			Opsz:  "48",
			Wght:  "400",
			Fill:  "0",
			Grad:  "0",
		},
		IconCodePointOn:  "e837",
		IconCodePointOff: "e836",
		BgColorOn:        "#067ba2",
		BgColorOff:       "#004162",
	}
}

func SetupPreClientRun(client *streamdeck.Client) {
	action := client.Action(ActionUUID)
	shownInstances = cmap.NewOf[string, instanceProperty]() // key: context of action instance
	renderCh = make(chan *renderParams, 32)

	action.RegisterHandler(streamdeck.DidReceiveSettings, func(ctx context.Context, client *streamdeck.Client, event streamdeck.Event) error {
		var p streamdeck.DidReceiveSettingsPayload[instanceSettings]
		p.Settings = defaultInstanceSettings()
		err := json.Unmarshal(event.Payload, &p)
		if err != nil {
			log.Printf("error unmarshaling payload: %v\n", err)
			return err
		}

		if shownInstances.Has(event.Context) {
			var dummy instanceProperty
			shownInstances.Upsert(event.Context, dummy, func(exist bool, valueInMap, _ instanceProperty) instanceProperty {
				valueInMap.Settings = p.Settings
				return valueInMap
			})
		}

		p.Settings.setImages(client, event.Context)

		return nil
	})

	action.RegisterHandler(streamdeck.WillAppear, func(ctx context.Context, client *streamdeck.Client, event streamdeck.Event) error {
		var p streamdeck.WillAppearPayload[instanceSettings]
		p.Settings = defaultInstanceSettings()
		err := json.Unmarshal(event.Payload, &p)
		if err != nil {
			log.Printf("error unmarshaling payload: %v\n", err)
			return err
		}
		shownInstances.Set(event.Context, instanceProperty(p))
		p.Settings.setImages(client, event.Context)
		if err := client.SetSettings(ctx, p.Settings); err != nil {
			log.Printf("error setting settings: %v\n", err)
			return err
		}
		return nil
	})

	action.RegisterHandler(streamdeck.WillDisappear, func(ctx context.Context, client *streamdeck.Client, event streamdeck.Event) error {
		shownInstances.Remove(event.Context)
		return nil
	})
}

func SetupPostClientRun(client *streamdeck.Client, vm *voicemeeter.Remote) error {
	action := client.Action(ActionUUID)

	action.RegisterHandler(streamdeck.KeyDown, func(ctx context.Context, client *streamdeck.Client, event streamdeck.Event) error {
		var p streamdeck.KeyDownPayload[instanceSettings]
		p.Settings = defaultInstanceSettings()
		err := json.Unmarshal(event.Payload, &p)
		if err != nil {
			log.Printf("error unmarshaling payload: %v\n", err)
			return err
		}

		logicalId, err := p.Settings.getSafeLogicalId(vm)
		if err != nil {
			log.Printf("error parsing logicalId: %v\n", err)
			return err
		}
		button := vm.Button[logicalId]

		if p.Settings.ButtonType == ButtonTypeToggle {
			currentState := button.State()
			button.SetState(!currentState)
			renderCh <- &renderParams{
				targetContext: event.Context,
				state:         !currentState,
			}
		} else if p.Settings.ButtonType == ButtonTypePush {
			button.SetState(true)
			renderCh <- &renderParams{
				targetContext: event.Context,
				state:         true,
			}
		}

		return nil
	})

	action.RegisterHandler(streamdeck.KeyUp, func(ctx context.Context, client *streamdeck.Client, event streamdeck.Event) error {
		var p streamdeck.KeyUpPayload[instanceSettings]
		p.Settings = defaultInstanceSettings()
		err := json.Unmarshal(event.Payload, &p)
		if err != nil {
			log.Printf("error unmarshaling payload: %v\n", err)
			return err
		}

		logicalId, err := p.Settings.getSafeLogicalId(vm)
		if err != nil {
			log.Printf("error parsing logicalId: %v\n", err)
			return err
		}
		button := vm.Button[logicalId]

		if p.Settings.ButtonType == ButtonTypePush {
			button.SetState(false)
			renderCh <- &renderParams{
				targetContext: event.Context,
				state:         false,
			}
		}

		return nil
	})

	go func() {
		for renderParam := range renderCh {
			render(client, renderParam)
		}
	}()

	vmEvent := make(chan string)
	vm.Register(vmEvent)
	go func() {
		for e := range vmEvent {
			switch e {
			case "mdirty":
				for item := range shownInstances.IterBuffered() {
					actionContext := item.Key
					actionProps := item.Val
					go func() {
						logicalId, err := actionProps.Settings.getSafeLogicalId(vm)
						if err != nil {
							log.Printf("error parsing logicalId: %v\n", err)
							return
						}
						button := vm.Button[logicalId]
						renderParam := &renderParams{
							targetContext: actionContext,
							state:         button.State(),
						}
						renderCh <- renderParam
					}()
				}
			}
		}
	}()

	return nil
}

func render(client *streamdeck.Client, renderParam *renderParams) {
	ctx := context.Background()
	ctx = sdcontext.WithContext(ctx, renderParam.targetContext)

	if renderParam.state {
		client.SetState(ctx, 0)
	} else {
		client.SetState(ctx, 1)
	}
}

func ptr[T any](v T) *T {
	return &v
}
