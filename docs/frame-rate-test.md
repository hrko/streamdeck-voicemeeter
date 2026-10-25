# Frame-rate camera test

Development builds provide `Frame Test (Key)` and `Frame Test (Dial)` actions.
They do not control audio and need no channel configuration. Place both from
VoiceMeeter's action list. They use the same worker refresh notifications as
normal actions, including when VoiceMeeter is disconnected.

- Large five-digit number: source update sequence, wrapping after 99999.
- Small number: milliseconds since the worker started (shared across actions).
- Top-left square: alternates light/dark every source update.

Record at 120 or 240 fps, with a short enough exposure to distinguish digits.
At a nominal 30 fps, one number should last about 4 or 8 camera frames respectively.
Camera and device clocks are not synchronized, so occasional differences in
repeat counts and partially scanned digits are not alone proof of a dropped frame.
A skipped sequence indicates an update not captured in the recording (including
application, device, or camera stages). Consecutive numbers with unusually large
millisecond increments indicate a slower source update. This is a source counter,
not an acknowledgement that the device displayed a frame. Compare normal meters
alongside the test actions to exercise a representative load.

Sequence and elapsed time reset when the plugin restarts. These actions are
excluded from release manifests and release executables.
