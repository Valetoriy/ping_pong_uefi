rm -rf esp
mkdir -p esp/efi/boot
cp target/x86_64-unknown-uefi/debug/uefi_ping_pong.efi esp/efi/boot/bootx64.efi
qemu-system-x86_64 -enable-kvm \
    -device virtio-rng-pci \
    -drive if=pflash,format=raw,readonly=on,file=/usr/share/ovmf/x64/OVMF_CODE.fd \
    -drive if=pflash,format=raw,readonly=on,file=/usr/share/ovmf/x64/OVMF_VARS.fd \
    -drive format=raw,file=fat:rw:esp
