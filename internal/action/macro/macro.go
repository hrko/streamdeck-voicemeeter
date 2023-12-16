package macro

import (
	"context"
	"encoding/json"
	"image"
	"image/color"
	"image/draw"
	"log"
	// "time"

	"github.com/FlowingSPDG/streamdeck"
	sdcontext "github.com/FlowingSPDG/streamdeck/context"
	"github.com/fufuok/cmap"
	"github.com/onyx-and-iris/voicemeeter/v2"

	"github.com/hrko/streamdeck-voicemeeter/internal/action"
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

type instanceProperty struct {
	action.ActionInstanceCommonProperty
	Settings instanceSettings `json:"settings,omitempty"`
}

type instanceSettings struct {
	LogicalId  int    `json:"logicalId,omitempty"`
	ButtonType string `json:"buttonType,omitempty"`
}

type renderParams struct {
	targetContext string
	state         bool
}

type keyDownPayload struct {
	action.KeyDownCommonPayload
	Settings instanceSettings `json:"settings,omitempty"`
}

type keyUpPayload struct {
	action.KeyUpCommonPayload
	Settings instanceSettings `json:"settings,omitempty"`
}

func defaultInstanceSettings() instanceSettings {
	return instanceSettings{
		LogicalId:  0,
		ButtonType: ButtonTypePush,
	}
}

func SetupPreClientRun(client *streamdeck.Client) {
	action := client.Action(ActionUUID)
	shownInstances = cmap.NewOf[string, instanceProperty]() // key: context of action instance
	renderCh = make(chan *renderParams, 32)

	action.RegisterHandler(streamdeck.DidReceiveSettings, func(ctx context.Context, client *streamdeck.Client, event streamdeck.Event) error {
		var prop instanceProperty
		prop.Settings = defaultInstanceSettings()
		err := json.Unmarshal(event.Payload, &prop)
		if err != nil {
			log.Printf("error unmarshaling payload: %v\n", err)
			return err
		}
		shownInstances.Set(event.Context, prop)
		return nil
	})

	action.RegisterHandler(streamdeck.WillAppear, func(ctx context.Context, client *streamdeck.Client, event streamdeck.Event) error {
		var prop instanceProperty
		prop.Settings = defaultInstanceSettings()
		err := json.Unmarshal(event.Payload, &prop)
		if err != nil {
			log.Printf("error unmarshaling payload: %v\n", err)
			return err
		}
		shownInstances.Set(event.Context, prop)
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
		var payload keyDownPayload
		payload.Settings = defaultInstanceSettings()
		err := json.Unmarshal(event.Payload, &payload)
		if err != nil {
			log.Printf("error unmarshaling payload: %v\n", err)
			return err
		}

		if payload.Settings.LogicalId < 0 || payload.Settings.LogicalId > len(vm.Button) {
			log.Printf("invalid logicalId: %v\n", payload.Settings.LogicalId)
			return nil
		}
		button := vm.Button[payload.Settings.LogicalId]

		if payload.Settings.ButtonType == ButtonTypeToggle {
			currentState := button.State()
			button.SetState(!currentState)
			renderCh <- &renderParams{
				targetContext: event.Context,
				state:         !currentState,
			}
		} else if payload.Settings.ButtonType == ButtonTypePush {
			button.SetState(true)
			renderCh <- &renderParams{
				targetContext: event.Context,
				state:         true,
			}
		}

		return nil
	})

	action.RegisterHandler(streamdeck.KeyUp, func(ctx context.Context, client *streamdeck.Client, event streamdeck.Event) error {
		var payload keyUpPayload
		payload.Settings = defaultInstanceSettings()
		err := json.Unmarshal(event.Payload, &payload)
		if err != nil {
			log.Printf("error unmarshaling payload: %v\n", err)
			return err
		}

		if payload.Settings.LogicalId < 0 || payload.Settings.LogicalId > len(vm.Button) {
			log.Printf("invalid logicalId: %v\n", payload.Settings.LogicalId)
			return nil
		}
		button := vm.Button[payload.Settings.LogicalId]

		if payload.Settings.ButtonType == ButtonTypePush {
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
				log.Println("macro dirty")
				for item := range shownInstances.IterBuffered() {
					actionContext := item.Key
					actionProps := item.Val
					go func() {
						if actionProps.Settings.LogicalId < 0 || actionProps.Settings.LogicalId > len(vm.Button) {
							log.Printf("invalid logicalId: %v\n", actionProps.Settings.LogicalId)
							return
						}
						button := vm.Button[actionProps.Settings.LogicalId]
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

	// go func() {
	// 	const refreshInterval = time.Second / 15
	// 	for range time.Tick(refreshInterval) {
	// 		for item := range shownInstances.IterBuffered() {
	// 			actionContext := item.Key
	// 			actionProps := item.Val
	// 			go func() {
	// 				renderParam := &renderParams{
	// 					targetContext: actionContext,
	// 				}

	// 				renderCh <- renderParam
	// 			}()
	// 		}
	// 	}
	// }()

	return nil
}

func render(client *streamdeck.Client, renderParam *renderParams) {
	ctx := context.Background()
	ctx = sdcontext.WithContext(ctx, renderParam.targetContext)

	if renderParam.state {
		white72x72 := image.NewRGBA(image.Rect(0, 0, 72, 72))
		draw.Draw(white72x72, white72x72.Bounds(), &image.Uniform{color.White}, image.Point{}, draw.Src)
		imgBase64, err := streamdeck.Image(white72x72)
		if err != nil {
			log.Printf("error encoding image: %v\n", err)
			return
		}
		client.SetImage(ctx, imgBase64, streamdeck.HardwareAndSoftware)
	} else {
		black72x72 := image.NewRGBA(image.Rect(0, 0, 72, 72))
		draw.Draw(black72x72, black72x72.Bounds(), &image.Uniform{color.Black}, image.Point{}, draw.Src)
		imgBase64, err := streamdeck.Image(black72x72)
		if err != nil {
			log.Printf("error encoding image: %v\n", err)
			return
		}
		client.SetImage(ctx, imgBase64, streamdeck.HardwareAndSoftware)
	}
}
