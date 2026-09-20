# Klor - RMK firware

This keyboard is from [BeeKeeb](https://shop.beekeeb.com/products/klor-split-keyboard-diy-kit?srsltid=AU7gw4WGGXsUuwuvzYfDMVgfDrchJ68C2mfAmpMadAbflrpDxo0v2YZf)

To flash this, you'll need another keyboard to operate your computer while flashing (obviously).

## Flash

1. Reset (hold reset button for >1s for the Sea Picro)
2. Make sure your micro [mounts appropriately](https://wiki.nixos.org/wiki/USB_storage_devices)
3. `nix develop` or use `direnv`
4. `cargo run --release <central|peripheral>`
5. open vial.app and configure as you desire
