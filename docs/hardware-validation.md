# Hardware acceptance record

Status: pending. Automated fake-backend and loopback tests do not constitute
hardware acceptance. Do not mark a row passed without observing its result.

## Available environment

- User-confirmed: Windows 11, Stream Deck +, VoiceMeeter Potato.
- Installed Stream Deck executable: 7.0.3.22071 (file metadata).
- Installed Potato executable: 3.1.2.2 (file metadata).
- Exact Windows build, connected device firmware and running application
  versions: not yet recorded.
- Basic and Banana executables exist locally, but neither edition has been
  confirmed as an available test environment. Windows 10 is also unverified.

## Before testing

Record the Git commit, package SHA-256, Windows build, Stream Deck version,
device firmware and VoiceMeeter edition/version below. Save the current mixer
configuration and use a dedicated test page with targets that will not interrupt
live audio. Install the package following usage.md, or use `mise run dev` for
the development link. The new plugin ID does not migrate existing settings.

| Field | Observed value |
| --- | --- |
| Commit / package SHA-256 | Pending |
| Windows build | Pending |
| Stream Deck / firmware | Pending |
| VoiceMeeter edition/version | Pending |
| Install or development link | Pending |
| Date / tester | Pending |

## Single-session procedure

For each row record pass/fail, observations and relevant logs. Restore mixer
settings after testing. In particular, an interrupted Push is deliberately not
released into a new VoiceMeeter session; inspect and reset its MacroButton state
manually when necessary.

| Check | Procedure and expected behavior | Result |
| --- | --- | --- |
| Installation | Install or dev-link; all three actions appear without replacing the old plugin. | Pending |
| Offline PI | Open PI with VoiceMeeter stopped; forms and saved settings remain usable and status reports disconnected. | Pending |
| Startup recovery | Start Potato; status and displays recover without reloading the plugin. Offline operations are not replayed. | Pending |
| Gain | Rotate slowly and rapidly with a 3 dB step; values accumulate and stop at -60/+12 dB. Press and tap toggle mute. | Pending |
| Combo | Set left step 3 and right step 0.5. Rotate normally and while pressed; each side uses its own target/step. Tap either half and the center boundary; only the expected target toggles. | Pending |
| Shared targets | Put the same target on multiple actions. Operate either and change gain/mute in VoiceMeeter; all displays converge. | Pending |
| Toggle | Press a Macro Toggle repeatedly, then change its state externally; state, icon and color synchronize. | Pending |
| Push | Hold/release a Macro Push. Repeat while changing its target/settings, changing page and normally closing the plugin; original target is released when communication remains available. | Pending |
| Overlapping Push | Hold two actions pointing at one MacroButton; releasing the first keeps it ON, releasing the last turns it OFF. | Pending |
| Restart during Push | Restart VoiceMeeter during a hold. The old press/release must not be replayed in the new session; a fresh press/release works. | Pending |
| Invalid target | Configure an out-of-range Strip/Bus or Macro ID; show an error and never operate on target zero. | Pending |
| State and meters | Compare stereo meters, mute, solo, routing, Mono/MC and EQ against VoiceMeeter on representative physical/virtual strips and buses. | Pending |
| Rendering | Use long and non-ASCII labels, three icon styles and extreme supported axes. No cross-channel overlap; missing codepoints show a warning/fallback. | Pending |
| PI | Restore settings, edit independent Combo values and Macro colors/icons, expand details, enter invalid numbers and navigate by keyboard. Invalid intermediate values are not saved. | Pending |
| PI appearance | Compare with a built-in PI at the same window size: colors, font size, label width, control height, spacing and focus. Record differences. | Pending |
| Localization | Switch host language between Japanese and English and restart the plugin. Confirm translated PI/action labels and unchanged saved targets. | Pending |
| Lifecycle/performance | Change pages repeatedly and operate multiple actions. No updates to disappeared actions, stale meter backlog or sustained excessive CPU usage. Record observed CPU and responsiveness. | Pending |
| Diagnostics | Confirm startup/errors/shutdown in the documented rotating log directory; attach relevant excerpts to failures. | Pending |

## Coverage outside this machine

| Environment | Status |
| --- | --- |
| Windows 11 / Stream Deck + / Potato | Available; tests pending |
| Windows 10 x64 | Not tested |
| VoiceMeeter Basic | Not tested |
| VoiceMeeter Banana | Not tested |
| Stream Deck 6.4 minimum version | Not tested; manifest declares this minimum |

Do not infer compatibility verification for these environments from a successful
Potato test. Record separate results when those environments become available.
