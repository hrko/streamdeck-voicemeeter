package main

import (
	"context"
	"encoding/json"
	"fmt"
	"log"
	"os"
	"path/filepath"
	"time"

	"github.com/FlowingSPDG/streamdeck"
	sdcontext "github.com/FlowingSPDG/streamdeck/context"
	"github.com/onyx-and-iris/voicemeeter/v2"

	"github.com/hrko/streamdeck-voicemeeter/pkg/graphics"
)

var (
	chGlobalSettings chan *GlobalSettings
)

type GlobalSettings struct {
	VoiceMeeterKind string `json:"voiceMeeterKind"`
}

func main() {
	log.SetPrefix("package main: ")
	streamdeck.Log().SetOutput(os.Stderr)
	streamdeck.Log().SetPrefix("package streamdeck: ")

	cacheDir := setupPluginCacheDir()
	graphics.SetMaterialSymbolsCacheDir(cacheDir)

	ctx := context.Background()
	log.Println("Starting voicemeeter-streamdeck-plugin")
	if err := run(ctx); err != nil {
		panic(err)
	}
}

func setupPluginCacheDir() string {
	userCacheDir := ""
	userCacheDir, err := os.UserCacheDir()
	if err != nil {
		log.Printf("error getting user cache dir: %v, fallback to temp dir\n", err)
		userCacheDir = os.TempDir()
	}
	cacheDir := filepath.Join(userCacheDir, "streamdeck-voicemeeter")
	if err := os.MkdirAll(cacheDir, 0755); err != nil {
		log.Printf("error creating cache dir: %v\n", err)
	}
	return cacheDir
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
