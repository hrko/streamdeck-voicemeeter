BINNAME = streamdeck-voicemeeter.exe
PLUGINNAME = jp.hrko.streamdeck.voicemeeter.sdPlugin
GO = go
GOFLAGS =
INSTALLDIR = "$(APPDATA)\Elgato\StreamDeck\Plugins\$(PLUGINNAME)"

.PHONY: kill-streamdeck start-streamdeck restart-streamdeck build layouts test install logs logs-full clear-logs run

kill-streamdeck:
	taskkill -t -f -im StreamDeck.exe || true

start-streamdeck:
	start "" "C:\Program Files\Elgato\StreamDeck\StreamDeck.exe"

restart-streamdeck: kill-streamdeck start-streamdeck

build:
	$(GO) build $(GOFLAGS) -o $(BINNAME) main.go

layouts:
	rm -f layouts/*.json
	$(GO) run ./cmd/layout-gen/ layouts.drawio layouts/

test:
	$(GO) run $(GOFLAGS) main.go -port 12345 -pluginUUID 213 -registerEvent test -info "{\"application\":{\"language\":\"en\",\"platform\":\"mac\",\"version\":\"4.1.0\"},\"plugin\":{\"version\":\"1.1\"},\"devicePixelRatio\":2,\"devices\":[{\"id\":\"55F16B35884A859CCE4FFA1FC8D3DE5B\",\"name\":\"Device Name\",\"size\":{\"columns\":5,\"rows\":3},\"type\":0},{\"id\":\"B8F04425B95855CF417199BCB97CD2BB\",\"name\":\"Another Device\",\"size\":{\"columns\":3,\"rows\":2},\"type\":1}]}"

install: kill-streamdeck build layouts
	rm -rf $(INSTALLDIR)
	mkdir $(INSTALLDIR)
	cp *.json $(INSTALLDIR)
	cp *.html $(INSTALLDIR)
	cp *.woff2 $(INSTALLDIR)
	cp *.exe $(INSTALLDIR)
	cp -r layouts $(INSTALLDIR)
	# ldd $(BINNAME) | sed 's/^.*=> \([^ ]\+\).*/\1/' | grep -v /c/ | xargs -i{} cp {} $(INSTALLDIR)

logs:
	ls -t "$(TMP)"/voicemeeter-streamdeck-plugin.*.log | head -n 1 | xargs tail +1f

logs-full:
	ls -t "$(TMP)"/voicemeeter-streamdeck-plugin.*.log | head -n 1 | xargs less

clear-logs:
	rm -f "$(TMP)"/voicemeeter-streamdeck-plugin.*.log

run: install start-streamdeck clear-logs
