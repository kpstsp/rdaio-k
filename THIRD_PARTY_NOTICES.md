# Third-Party Notices

This project uses third-party software components that are licensed under their own terms.
See the `licenses/` directory for license texts referenced below.

## Direct dependencies

| Library   | Version | License         | Source |
|----------|---------|-----------------|--------|
| ratatui  | 0.26    | MIT             | https://github.com/ratatui-org/ratatui |
| crossterm| 0.27    | MIT             | https://github.com/crossterm-rs/crossterm |
| symphonia| 0.5.5   | MPL-2.0         | https://github.com/pdeljanov/Symphonia |
| rodio    | 0.17    | MIT OR Apache-2.0 | https://github.com/RustAudio/rodio |
| id3      | 1.x     | MIT OR Apache-2.0 | https://github.com/jameshurst/rust-id3 |

## License details

### MIT License
Applies to: ratatui, crossterm.  
Text: see `LICENSE` in the repository root.

Note: Some dependencies are dual-licensed (e.g. rodio, id3) and may be used under either MIT or Apache-2.0.

### MPL-2.0 License (Mozilla Public License 2.0)
Applies to: Symphonia.  
Text: `licenses/MPL-2.0.txt`.  
Upstream license reference: https://github.com/pdeljanov/Symphonia/blob/master/LICENSE

### Apache-2.0 License
Applies to: dual-licensed dependencies when used under Apache-2.0 (e.g. rodio, id3).  
Text: `licenses/Apache-2.0.txt`.

## Transitive dependencies

This project depends on additional transitive Rust crates pulled in via the direct dependencies.
A complete list of dependencies and their license expressions should be generated from `Cargo.lock`, for example with:
- `cargo deny check licenses` (recommended), or
- a license-reporting tool such as `cargo license` / `cargo 3pl` (if you use them in your workflow).

## Source availability (for binary releases)

Binary releases include MPL-2.0 components (via Symphonia).  
When distributing executables, recipients must be informed how to obtain the corresponding source code for the MPL-2.0 covered software (see MPL-2.0 Section 3.2(a)). 

For GitHub Releases, the corresponding source code is provided via the release tag (“Source code (zip/tar.gz)” assets) or an equivalent link in the release description.

## Compliance notes

- This project is licensed under MIT; it also includes Symphonia under MPL-2.0 (weak copyleft).
- `Cargo.lock` is committed to support reproducible builds.
- Before distributing binaries, run a license scan on the exact dependency graph (from `Cargo.lock`) and include the required license texts/notices in the release artifacts.
