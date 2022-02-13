# Mini mining game

## Run
```
cargo run --release
```

## Controls

Hold right click to camera look

- W - move forward
- S - move backward
- A - move left
- D - move right
- Q - move down
- E - move up
- SHIFT + Click - shoot projectile at mouse location
- M - change camera view to MOBA style camera
- F - change camera view to FPS

## Moving the inverse kinematics arm

- Click - to select the hand (The last box of the kinematics set-up)
- ALT + Click - to move the hand around the 3D space.
- X / SHIFT + X - to move the hand around X axis
- Y / SHIFT + Y - to move the hand around Y axis
- Z / SHIFT + Z - to move the hand around Z axis
- R - reset the hand position
    - do the reset if the IK errored, won't move anymore.
