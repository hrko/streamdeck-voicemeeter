package main

import (
	"context"
	"encoding/json"
	"fmt"
	"log"
	"strconv"
	"time"

	"github.com/FlowingSPDG/streamdeck"
	sdcontext "github.com/FlowingSPDG/streamdeck/context"
	"github.com/fufuok/cmap"
	"github.com/onyx-and-iris/voicemeeter/v2"

	"github.com/hrko/streamdeck-voicemeeter/pkg/graphics"
)

var (
	action1InstanceMap   *cmap.MapOf[string, Action1InstanceProperty]
	action1RenderCh      chan *Action1RenderParams
	action1LevelMeterMap *cmap.MapOf[string, *graphics.LevelMeter]
)

type Action1InstanceProperty struct {
	ActionInstanceCommonProperty
	Settings Action1InstanceSettings `json:"settings,omitempty"`
}

type Action1InstanceSettings struct {
	IconCodePoint   string                             `json:"iconCodePoint,omitempty"`
	IconFontParams  graphics.MaterialSymbolsFontParams `json:"iconFontParams,omitempty"`
	StripOrBusKind  string                             `json:"stripOrBusKind,omitempty"` // "Strip" | "Bus"
	StripOrBusIndex int                                `json:"stripOrBusIndex,omitempty"`
	GainDelta       string                             `json:"gainDelta,omitempty"`
}

type Action1RenderParams struct {
	TargetContext string
	Title         *string
	Settings      *Action1InstanceSettings
	Levels        *[]float64
	Gain          *float64
	Status        *StripOrBusStatus
}

type Action1DialRotateCommonPayload struct {
	DialRotateCommonPayload
	Settings Action1InstanceSettings `json:"settings,omitempty"`
}

func action1SetupPreClientRun(client *streamdeck.Client) {
	action := client.Action("jp.hrko.voicemeeter.action1")
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
	action := client.Action("jp.hrko.voicemeeter.action1")
	action1LevelMeterMap = cmap.NewOf[string, *graphics.LevelMeter]() // key: context of action instance

	action.RegisterHandler(streamdeck.DialRotate, func(ctx context.Context, client *streamdeck.Client, event streamdeck.Event) error {
		b, _ := json.MarshalIndent(event, "", "	")
		log.Printf("event:%s\n", b)

		var p Action1DialRotateCommonPayload
		p.Settings.StripOrBusKind = "Strip"
		p.Settings.StripOrBusIndex = 0
		p.Settings.GainDelta = "3.0"
		err := json.Unmarshal(event.Payload, &p)
		if err != nil {
			log.Printf("error unmarshaling payload: %v\n", err)
			return err
		}

		gainDelta, err := strconv.ParseFloat(p.Settings.GainDelta, 64)
		if err != nil {
			log.Printf("error parsing gainDelta: %v\n", err)
			gainDelta = 3.0 // default
		}
		switch p.Settings.StripOrBusKind {
		case "Strip":
			adjustStripGain(vm, p.Settings.StripOrBusIndex, gainDelta*float64(p.Ticks))

		case "Bus":
			adjustBusGain(vm, p.Settings.StripOrBusIndex, gainDelta*float64(p.Ticks))

		default:
			log.Printf("unknown stripOrBusKind: '%v'\n", p.Settings.StripOrBusKind)
		}

		return nil
	})

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
				actionProps := item.Val
				go func() {
					renderParam := &Action1RenderParams{}
					renderParam.TargetContext = actionContext

					stripOrBusKind := actionProps.Settings.StripOrBusKind
					if stripOrBusKind == "" {
						stripOrBusKind = "Strip"
					}

					switch stripOrBusKind {
					case "Strip":
						stripIndex := actionProps.Settings.StripOrBusIndex
						stripCount := len(vm.Strip)
						if stripIndex >= stripCount || stripIndex < 0 {
							log.Printf("stripIndex %v is out of range\n", stripIndex)
							return
						}
						levels := vm.Strip[stripIndex].Levels().PostFader()
						levels = levels[:2]
						title := vm.Strip[stripIndex].Label()
						if title == "" {
							title = fmt.Sprintf("Strip %v", stripIndex+1)
						}
						gain := vm.Strip[stripIndex].Gain()
						status := &StripOrBusStatus{}
						status.IsStrip = true
						stripStatus, err := getStripStatus(vm, stripIndex)
						if err != nil {
							log.Printf("error getting strip status: %v\n", err)
						}
						status.StripStatus = stripStatus
						renderParam.Levels = &levels
						renderParam.Title = &title
						renderParam.Gain = &gain
						renderParam.Status = status

					case "Bus":
						busIndex := actionProps.Settings.StripOrBusIndex
						busCount := len(vm.Bus)
						if busIndex >= busCount || busIndex < 0 {
							log.Printf("busIndex %v is out of range\n", busIndex)
							return
						}
						levels := vm.Bus[busIndex].Levels().All()
						levels = levels[:2]
						title := vm.Bus[busIndex].Label()
						if title == "" {
							title = fmt.Sprintf("Bus %v", busIndex+1)
						}
						gain := vm.Bus[busIndex].Gain()
						status := &StripOrBusStatus{}
						status.IsStrip = false
						busStatus, err := getBusStatus(vm, busIndex)
						if err != nil {
							log.Printf("error getting bus status: %v\n", err)
						}
						status.BusStatus = busStatus
						renderParam.Levels = &levels
						renderParam.Title = &title
						renderParam.Gain = &gain
						renderParam.Status = status

					default:
						log.Printf("unknown stripOrBusKind: '%v'\n", stripOrBusKind)
						return
					}

					action1RenderCh <- renderParam
				}()
			}
		}
	}()

	return nil
}

func action1Render(client *streamdeck.Client, renderParam *Action1RenderParams) error {
	ctx := context.Background()
	ctx = sdcontext.WithContext(ctx, renderParam.TargetContext)

	instProps, ok := action1InstanceMap.Get(renderParam.TargetContext)
	if !ok {
		return fmt.Errorf("action1InstanceMap has no key '%v'", renderParam.TargetContext)
	}

	levelMeter, ok := action1LevelMeterMap.Get(renderParam.TargetContext)
	if !ok {
		levelMeter = graphics.NewLevelMeter(2)
		action1LevelMeterMap.Set(renderParam.TargetContext, levelMeter)
	}

	switch instProps.Controller {
	case "Encoder":
		payload := struct {
			Title      *string `json:"title,omitempty"`
			Icon       *string `json:"icon,omitempty"`
			LevelMeter *string `json:"levelMeter,omitempty"`
			GainValue  *string `json:"gainValue,omitempty"`
			GainSlider *string `json:"gainSlider,omitempty"`
			Status     *string `json:"status,omitempty"`
		}{}

		if renderParam.Title != nil {
			payload.Title = renderParam.Title
		}
		if renderParam.Settings != nil {
			fontParams := renderParam.Settings.IconFontParams
			fontParams.FillEmptyWithDefault()
			if err := fontParams.Assert(); err != nil {
				log.Printf("invalid iconFontParams: %v\n", err)
				fontParams = graphics.MaterialSymbolsFontParams{}
				fontParams.FillEmptyWithDefault()
			}
			iconCodePoint := renderParam.Settings.IconCodePoint
			if iconCodePoint == "" {
				switch renderParam.Settings.StripOrBusKind {
				case "Strip", "":
					iconCodePoint = "f71a" // input_circle
				case "Bus":
					iconCodePoint = "f70e" // output_circle
				}
			}
			img, err := fontParams.RenderIcon(iconCodePoint, 48)
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
			levelMeter.Image.Width = 108
			levelMeter.Image.Height = 5
			levelMeter.Image.Padding.Left = 2
			levelMeter.Image.Padding.Right = 3
			levelMeter.Cell.Length = 1
			levelMeter.PeakHold = graphics.LevelMeterPeakHoldFillPeakShowCurrent
			img, err := levelMeter.RenderHorizontal(*renderParam.Levels)
			if err != nil {
				log.Printf("error creating image: %v\n", err)
				return err
			}
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

			gainFader := graphics.NewGainFader()
			gainFader.Width = 108
			gainFader.Height = 12
			img := gainFader.RenderHorizontal(*renderParam.Gain)
			imgBase64, err := streamdeck.Image(img)
			if err != nil {
				log.Printf("error creating image: %v\n", err)
				return err
			}
			payload.GainSlider = &imgBase64
		}
		if renderParam.Status != nil {
			s := renderParam.Status
			img, err := s.RenderIndicator()
			if err != nil {
				log.Printf("error creating image: %v\n", err)
			}
			imgBase64, err := streamdeck.Image(img)
			if err != nil {
				log.Printf("error creating image: %v\n", err)
			}
			payload.Status = &imgBase64
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

func adjustStripGain(vm *voicemeeter.Remote, stripIndex int, delta float64) error {
	if vm == nil {
		log.Printf("vm is nil\n")
		return fmt.Errorf("vm is nil")
	}

	if stripIndex >= len(vm.Strip) || stripIndex < 0 {
		log.Printf("stripIndex %v is out of range\n", stripIndex)
		return fmt.Errorf("stripIndex %v is out of range", stripIndex)
	}

	strip := vm.Strip[stripIndex]
	gain := strip.Gain()
	gain += delta
	if gain > 12.0 {
		gain = 12.0
	}
	if gain < -60.0 {
		gain = -60.0
	}
	strip.SetGain(gain)

	return nil
}

func adjustBusGain(vm *voicemeeter.Remote, busIndex int, delta float64) error {
	if vm == nil {
		log.Printf("vm is nil\n")
		return fmt.Errorf("vm is nil")
	}

	if busIndex >= len(vm.Bus) || busIndex < 0 {
		log.Printf("busIndex %v is out of range\n", busIndex)
		return fmt.Errorf("busIndex %v is out of range", busIndex)
	}

	bus := vm.Bus[busIndex]
	gain := bus.Gain()
	gain += delta
	if gain > 12.0 {
		gain = 12.0
	}
	if gain < -60.0 {
		gain = -60.0
	}
	bus.SetGain(gain)

	return nil
}
