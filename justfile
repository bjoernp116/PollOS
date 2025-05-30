_default:
    @just --choose

generate_disk:
    dd if=/dev/zero of=disk.img bs=512 count=32768
    mkfs.fat -F 16 disk.img
    sudo mount -o loop,uid=$(id -u),gid=$(id -g) disk.img /mnt
    cp -rf userspace/* /mnt
    sudo umount /mnt

build:
    @cargo fix --allow-dirty
    @cargo fmt --all
    @cargo build && cargo bootimage

bochs: build
    bochs -f .bochsrc

run args="" display="sdl": build
    qemu-system-x86_64 \
        -drive id=boot,\
            format=raw,\
            file=target/x86_64-pollos/debug/bootimage-pollos.bin,\
            if=ide,\
            index=0 \
        -drive id=disk,\
            format=raw,\
            file=disk.img,\
            if=ide,\
            index=1 \
        -display {{display}} \
        -serial stdio \
        -device isa-debug-exit,iobase=0xf4,iosize=0x04 \
        -vga std \
        {{args}}

test:
    @cargo test

edit:
    nvim ./justfile
