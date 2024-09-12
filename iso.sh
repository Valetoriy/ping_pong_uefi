# Тут происходит небольшой trolling, нормисам привычно загружать .iso файлы
dd if=/dev/zero of=ping_pong.iso bs=512 count=1000
mkfs.fat -F 12 -n 'PING_PONG' ping_pong.iso
mmd -i ping_pong.iso ::efi
mmd -i ping_pong.iso ::efi/boot
mcopy -i ping_pong.iso \
    target/x86_64-unknown-uefi/release/uefi_ping_pong.efi ::efi/boot/bootx64.efi
