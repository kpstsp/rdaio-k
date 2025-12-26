# Third Party Notices

This project uses the following third-party libraries and frameworks:

## Direct Dependencies

| Library | Version | License | Source |
|---------|---------|---------|--------|
| ratatui | 0.26 | MIT | https://github.com/ratatui-org/ratatui |
| crossterm | 0.27 | MIT | https://github.com/crossterm-rs/crossterm |
| symphonia | 0.5.5 | MPL-2.0 | https://github.com/pdeljanov/symphonia |
| rodio | 0.17 | MIT/Apache-2.0 | https://github.com/RustAudio/rodio |
| id3 | 1.x | MIT/Apache-2.0 | https://github.com/jameshurst/rust-id3 |

## License Details

### MIT License
The MIT License applies to: ratatui, crossterm, rodio (dual), id3 (dual)

See LICENSE file in the root directory.

### MPL-2.0 License
The Mozilla Public License 2.0 applies to: symphonia

Full text available in: licenses/MPL-2.0.txt

### Apache-2.0 License
The Apache License 2.0 applies to: rodio (dual), id3 (dual)

See licenses/Apache-2.0.txt

## Transitive Dependencies

This project also depends on numerous transitive dependencies. A complete list can be generated with:

```bash
cargo tree
```

Or check with:

```bash
cargo deny check licenses
```

All transitive dependencies are licensed under permissive open-source licenses compatible with this project's MIT license.

## Compliance

- All dependencies are properly licensed and compatible with this project
- No GPL or copyleft licenses are used
- Full license texts are provided in the `licenses/` directory
- `Cargo.lock` is committed to ensure reproducible builds
