# ADR-013: Input Method (IME) Setup for Rustile in TTY Environment

**Date**: 2025-09-30
**Status**: Accepted
**Context**: Setting up Japanese input (fcitx5) in rustile when running on TTY3 alongside desktop environment

## Problem

When running rustile on TTY3 (via `startx -- :10`) alongside GNOME/Wayland on TTY1/2, input methods (IME) like fcitx5 don't work properly in applications, particularly Chrome.

**Symptoms observed:**
- Terminal (alacritty) and Emacs: IME works ✓
- Chrome: IME doesn't work ✗
- Chrome logs showed DBus connection errors: `Failed to connect to the bus: Could not parse server address`

## Investigation Process

### Step 1: Basic IME Configuration
Initial `.xinitrc` setup attempted:
```bash
export GTK_IM_MODULE=fcitx
export XMODIFIERS=@im=fcitx
fcitx5 -d -r &
```

**Result**: Terminal and Emacs worked, but Chrome failed.

### Step 2: Debugging Chrome
Created diagnostic scripts to check:
- fcitx5 process status: Running correctly (PID 83196, active)
- Environment variables: Set correctly (`GTK_IM_MODULE=fcitx`, etc.)
- Chrome process environment: Variables inherited correctly

**Discovery**: Chrome logs revealed the root cause:
```
[ERROR:dbus/bus.cc:408] Failed to connect to the bus:
Could not parse server address: Unknown address type
```

### Step 3: DBus Session Bus Issue
Key findings:
- DBus session daemon was already running (systemd-managed, PID 2263) from GNOME session
- `DBUS_SESSION_BUS_ADDRESS` environment variable was **not set** in TTY3 X session
- fcitx5 uses DBus for inter-process communication with applications
- Without the DBus address, Chrome couldn't communicate with fcitx5

## Solution

Add `DBUS_SESSION_BUS_ADDRESS` environment variable to `.xinitrc` to point applications to the existing systemd-managed DBus session.

**Result**: Chrome can now connect to DBus and communicate with fcitx5. See "Configuration Files Modified" section below for complete `.xinitrc` setup.

## Technical Background

### What is DBus?
DBus (Desktop Bus) is a Linux inter-process communication (IPC) system that allows processes to exchange messages. Many desktop services use DBus, including:
- Input method frameworks (fcitx5, ibus)
- Desktop notifications
- Media players
- System services

### How IME Works - Complete Flow

```
Keyboard → X11 Server → Application (Chrome) ⇄ fcitx5 (IME Framework) → Mozc/SKK (Conversion Engine)
```

**Detailed flow when typing Japanese:**

1. **Key press**: User types 'a' on keyboard
2. **X11 captures**: X Window System receives hardware event
3. **X11 forwards**: Sends KeyPress event to focused application (Chrome)
4. **Chrome asks fcitx5**: "Should I handle this key, or do you want it?" (via DBus)
5. **fcitx5 decides**:
   - If IME is OFF (ASCII mode): "You handle it" → Chrome gets 'a'
   - If IME is ON (Japanese mode): "I'll handle it" → fcitx5 takes the key
6. **fcitx5 → Mozc/SKK**: fcitx5 sends 'a' to conversion engine
7. **Mozc/SKK converts**: 'a' → 'あ' (hiragana) → suggests kanji candidates
8. **fcitx5 → Chrome**: Sends conversion candidates via DBus
9. **Chrome displays**: Shows 'あ' with candidate list

**Component roles:**

- **X11**: Keyboard event capture and window management (NOT involved in IME logic)
- **fcitx5**: IME framework - manages input methods, handles protocol communication
- **Mozc**: Google's conversion engine (kana→kanji, predictive input)
- **SKK**: Alternative conversion engine (simpler, manual kana-kanji conversion with 'l' key)
- **DBus**: Communication channel between Chrome and fcitx5

**Why Terminal/Emacs worked without DBus:**
They use **XIM (X Input Method)**, an older protocol built into X11:
- fcitx5 provides both XIM (via `XMODIFIERS`) and DBus interfaces
- XIM: Simple, part of X11 protocol, works everywhere
- DBus: Modern, feature-rich, required by Chrome/Firefox

**fcitx5 vs Mozc vs SKK:**
```
┌─────────────────────────────────────┐
│  fcitx5 (IME Framework)             │
│  ┌────────────────────────────────┐ │
│  │ Input Method Plugins:          │ │
│  │  - Mozc    (Google IME style)  │ │
│  │  - SKK     (manual conversion) │ │
│  │  - Anthy   (another engine)    │ │
│  └────────────────────────────────┘ │
└─────────────────────────────────────┘
```

### Environment Variables Required
- `GTK_IM_MODULE=fcitx`: Tells GTK applications to use fcitx as input method
- `XMODIFIERS=@im=fcitx`: Tells X11 applications about the input method (enables XIM protocol)
- `DBUS_SESSION_BUS_ADDRESS`: Tells applications where to find the DBus session bus

## Different Deployment Scenarios

### Scenario 1: TTY3 Alongside Desktop Environment (Current Setup)
**Environment**: GNOME running on TTY1/2, rustile on TTY3

**DBus Setup**:
```bash
# Use existing systemd-managed DBus session
export DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/$(id -u)/bus
```

**Rationale**:
- Desktop environment already started a DBus session
- Multiple X sessions can share the same DBus session bus
- No need to start another DBus daemon (avoids conflicts)

### Scenario 2: Rustile as Primary Window Manager (No Desktop Environment)
**Environment**: Only rustile, no desktop environment

**DBus Setup Option A** (dbus-launch):
```bash
# Start DBus session if not already running
if [ -z "$DBUS_SESSION_BUS_ADDRESS" ]; then
    eval $(dbus-launch --sh-syntax --exit-with-session)
fi
```

**DBus Setup Option B** (dbus-run-session - recommended):
```bash
# Wrap entire session in DBus
exec dbus-run-session -- rustile > ~/.rustile.log 2>&1
```

**Rationale**:
- No existing DBus session, must create one
- `dbus-run-session` is cleaner (automatic cleanup on exit)
- `--exit-with-session` ensures DBus daemon stops when X session ends

### Scenario 3: Xephyr (Nested X Server for Testing)
**Environment**: Testing rustile in Xephyr window

**DBus Setup**:
```bash
# Use parent session's DBus
DISPLAY=:10 DBUS_SESSION_BUS_ADDRESS=$DBUS_SESSION_BUS_ADDRESS rustile &
```

**Rationale**: Inherit DBus from parent desktop session

## Configuration Files Modified

### ~/.xinitrc (Final Version for Scenario 1)
```bash
xsetroot -solid gray

# Set DBus session address (systemd-managed)
export DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/$(id -u)/bus

# Input method configuration for fcitx5
export GTK_IM_MODULE=fcitx      # For GTK apps (Firefox, GIMP, etc.)
export XMODIFIERS=@im=fcitx     # For X11 apps in general
fcitx5 -d -r &                  # Start fcitx5 daemon in background (-r = replace if running)

exec rustile > ~/.rustile.log 2>&1
```

## Testing & Verification

### Quick Diagnostic Commands
```bash
# 1. Check fcitx5 is running (watch for zombie/defunct processes)
ps aux | grep fcitx5 | grep -v grep

# 2. Verify environment variables are set
echo "GTK_IM_MODULE=$GTK_IM_MODULE"
echo "XMODIFIERS=$XMODIFIERS"
echo "DBUS_SESSION_BUS_ADDRESS=$DBUS_SESSION_BUS_ADDRESS"

# 3. Check fcitx5 status (should return 1=active or 2=inactive)
fcitx5-remote -s
echo $?  # Should be 0 (success), not error

# 4. Check Chrome logs for DBus errors (this was the key to finding the problem)
google-chrome 2>&1 | tee /tmp/chrome.log &
sleep 3
grep -i "dbus" /tmp/chrome.log | head -5
```

### Test Procedure
1. Terminal/Emacs: Ctrl-j activates Japanese input, 'l' switches to ASCII
2. Chrome: Same Ctrl-j/l shortcuts work
3. Verify no DBus errors in Chrome logs

## Lessons Learned

### Why This Was Difficult
1. **Multiple layers**: X11, DBus, IME framework, application toolkit (GTK in this case)
2. **Implicit dependencies**: DBus requirement not documented in fcitx5 basic setup
3. **Partial success misleading**: Terminal/Emacs working suggested configuration was correct
4. **Different app behaviors**: Each application uses different IME protocols (XIM vs DBus)

### Key Debugging Technique
When IME works in some apps but not others:
1. Check application logs (e.g., `google-chrome 2>&1 | tee chrome.log`)
2. Look for DBus/communication errors
3. Verify `DBUS_SESSION_BUS_ADDRESS` is set
4. Test with `dbus-send --session --print-reply --dest=org.freedesktop.DBus /org/freedesktop/DBus org.freedesktop.DBus.GetId`

## References

- **fcitx5 Setup Guide**: https://fcitx-im.org/wiki/Setup_Fcitx_5/en - Environment variables (GTK_IM_MODULE, XMODIFIERS) and dbus-launch usage
- **Arch Wiki - Fcitx5**: https://wiki.archlinux.org/title/Fcitx5 - Basic setup and troubleshooting

## Future Considerations

- Document this setup in README.md for users
- Test with other IME frameworks (ibus, etc.)
