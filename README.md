# streamdeck-voicemeeter
*Work in progress.*

## Actions
### Key
- [ ] Toggle Mute
- [ ] Gain Control (Set, Increment, Decrement)
- [x] VoiceMeeter Macro
- [ ] Restart VoiceMeeter

### Dial and Touchpad
- [x] Gain Control
- [x] Gain Control Combo
- [ ] Strip/Bus Parameter Control

## Screenshots
### Actions
![Actions](./screenshots/actions.png)

### Configuration
| Macro | Gain Control | Gain Control Combo |
| --- | --- | --- |
| ![Macro](./screenshots/macro.png) | ![Gain Control](./screenshots/gain_control.png) | ![Gain Control Combo](./screenshots/gain_control_combo.png) |

## Build Requirements
- Go 1.23.0 or later
- [Task](https://taskfile.dev/installation/)

## Build and Run
You can build and run the plugin using the following command:

```bash
cd <project-root>
task dev
```

This command will build the plugin, kill the running Stream Deck app, install the plugin, and run the Stream Deck app.

## Generate Layouts
Layouts describe how information is shown on the Stream Deck + touch display. Visit [Stream Deck SDK](https://docs.elgato.com/sdk/plugins/layouts-sd+) for more information.

First, create layouts using Draw.io and save it as a drawio file at `<project-root>/layout.drawio`.
Then, convert the layout to JSON files using the following command:
```bash
cd <project-root>
task layouts
```

Output files will be generated in `<project-root>/layouts/`.
