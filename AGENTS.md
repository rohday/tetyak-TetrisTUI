# Project Instructions for AI Agents

## Mandatory Build & Installation Rule
**ALWAYS update the binary installation whenever code is modified**:
After making any code changes in this repository, you **MUST** run:
```bash
make install
# or
cargo build --release && cp target/release/tetyak ~/.local/bin/tetyak && cp target/release/tetris ~/.local/bin/tetris && chmod +x ~/.local/bin/tetyak ~/.local/bin/tetris
```
Never leave the repository in a state where source changes have not been compiled and copied to `~/.local/bin/tetyak` and `~/.local/bin/tetris`.
